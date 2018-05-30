use discord::gateway::websocket::Error as WError;
use discord::gateway::websocket::{Client, ClientDecoder, ClientEncoder};
use error::{DResult, Error};

use std::fmt::{self, Write};
use std::io::BufReader;
use std::net::{SocketAddr, ToSocketAddrs};
use std::str;

use base64::display::Base64Display;
use either::Either;
use futures::prelude::*;
use http::uri::{Scheme, Uri};
use httparse::{Response, Status, EMPTY_HEADER};
use native_tls::TlsConnector;
use rand::{thread_rng, Rng};
use sha1::Sha1;
use tokio::io::{read_until, write_all, AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_io::codec::{FramedRead, FramedWrite};
use tokio_io::io::{ReadHalf, WriteHalf};
use tokio_tls::{TlsConnectorExt, TlsStream};

pub struct ConnectSettings {
    /// The largest frame that will be accepted from the server.
    /// Default is 1_000_000 bytes
    pub max_websocket_frame: usize,
    pub extra_headers: Vec<String>,
}

#[async]
pub fn connect_with_settings(uri: Uri, settings: ConnectSettings) -> DResult<Client> {
    let connection = await!(establish_connection(uri.clone()))?;

    match connection {
        Either::Left((r, w)) => await!(do_connect(uri, settings, r, w)),
        Either::Right((r, w)) => await!(do_connect(uri, settings, r, w)),
    }
}

#[async]
fn do_connect<R, W>(
    uri: Uri,
    settings: ConnectSettings,
    mut reader: BufReader<R>,
    writer: W,
) -> DResult<Client>
where
    R: AsyncRead + Send + 'static,
    W: AsyncWrite + Send + 'static,
{
    let sec_key = thread_rng().gen::<[u8; 16]>();
    let hashed_key = hash_sec_key(&sec_key);

    let headers = format!(
        "GET {path} HTTP/1.1\r\n\
         Host: {host}\r\n\
         Connection: Upgrade\r\n\
         Upgrade: WebSocket\r\n\
         Sec-WebSocket-Key: {key}\r\n\
         Sec-WebSocket-Version: 13\r\n\
         {extra}\
         \r\n",
        path = uri.path_and_query().map(|p| p.as_str()).unwrap_or("/"),
        host = host_port(&uri),
        key = Base64Display::standard(&sec_key),
        extra = DisplayHeaders(&settings.extra_headers),
    );
    let (mut writer, _) = await!(write_all(writer, headers))?;

    let mut buf = Vec::with_capacity(512);
    loop {
        let prev_len = buf.len();
        let (nreader, nbuf) = await!(read_until(reader, b'\n', buf))?;
        buf = nbuf;
        reader = nreader;

        let read = buf.len() - prev_len;
        if read <= 2 {
            break;
        }
    }

    let mut headers = [EMPTY_HEADER; 20];
    let mut resp = Response::new(&mut headers);
    match resp.parse(&buf) {
        Ok(Status::Complete(_)) => (),
        Ok(Status::Partial) => return Err(WError::IncompleteHeaders.into()),
        Err(e) => return Err(WError::Headers(e).into()),
    };

    // RFC 6455 Page 19, Point 1
    if resp.code != Some(101) {
        return Err(WError::InvalidResponseCode(resp.code).into());
    }

    let mut has_upgrade = false;
    let mut has_connection = false;
    let mut sec_accept = None;
    let mut sec_extensions = false;
    let mut sec_protocols = false;

    for header in resp.headers {
        if header.name.eq_ignore_ascii_case("Upgrade") {
            if header.value.eq_ignore_ascii_case(b"websocket") {
                has_upgrade = true;
            }
        } else if header.name.eq_ignore_ascii_case("Connection") {
            for tok in header.value.split(|&b| b == b' ') {
                if tok.eq_ignore_ascii_case(b"upgrade") {
                    has_connection = true;
                }
            }
        } else if header.name.eq_ignore_ascii_case("Sec-WebSocket-Accept") {
            sec_accept = str::from_utf8(header.value).map(str::trim).ok();
        } else if header.name.eq_ignore_ascii_case("Sec-WebSocket-Extensions") {
            if !header.value.iter().all(|b| b.is_ascii_whitespace()) {
                sec_extensions = true;
            }
        } else if header.name.eq_ignore_ascii_case("Sec-WebSocket-Protocol") {
            if !header.value.iter().all(|b| b.is_ascii_whitespace()) {
                sec_protocols = true;
            }
        }
    }

    // RFC 6455 Page 19, Point 2/3
    if !has_upgrade || !has_connection {
        return Err(WError::BadUpgrade.into());
    }

    // RFC 6455 Page 19, Point 4
    if !sec_accept
        .map(|a| b64_equal(a, &hashed_key))
        .unwrap_or(false)
    {
        return Err(WError::BadSecretKey.into());
    }

    // RFC6455 Page 19, Point 5/6. I'm not using any extensions/protocols,
    // and the server isn't allowed to turn on ones I didn't request.
    if sec_extensions || sec_protocols {
        return Err(WError::UnexpectedExtensions.into());
    }

    let decoder = ClientDecoder::with_limit(settings.max_websocket_frame);
    let reader = FramedRead::new(reader, decoder);
    let writer = FramedWrite::new(writer, ClientEncoder);

    let reader = Box::new(reader);
    let writer = Box::new(writer);

    Ok(Client { reader, writer })
}

pub fn connect(uri: Uri) -> impl Future<Item = Client, Error = Error> {
    connect_with_settings(
        uri,
        ConnectSettings {
            max_websocket_frame: 1_000_000,
            extra_headers: vec![],
        },
    )
}

pub fn connect_with_auth<S>(uri: Uri, header: S) -> impl Future<Item = Client, Error = Error>
where
    S: Into<String>,
{
    connect_with_settings(
        uri,
        ConnectSettings {
            max_websocket_frame: 1_000_000,
            extra_headers: vec![header.into()],
        },
    )
}

#[async]
fn establish_connection(uri: Uri) -> DResult<Connection> {
    let secure = match uri.scheme_part().map(Scheme::as_str) {
        Some("ws") => false,
        Some("wss") => true,
        _ => return Err(WError::InvalidUri.into()),
    };

    if secure {
        await!(establish_tls(uri))
    } else {
        await!(establish_plain(uri))
    }
}

#[async]
fn establish_plain(uri: Uri) -> DResult<Connection> {
    let addr = get_addr(&uri, 80)?;
    let stream = await!(TcpStream::connect(&addr))?;
    let (r, w) = stream.split();
    Ok(Either::Left((BufReader::new(r), w)))
}

#[async]
fn establish_tls(uri: Uri) -> DResult<Connection> {
    let addr = get_addr(&uri, 443)?;
    let stream = await!(TcpStream::connect(&addr))?;
    let tls = TlsConnector::builder()?.build()?;
    let tls_stream = await!(tls.connect_async(domain(&uri), stream))?;
    let (r, w) = tls_stream.split();
    Ok(Either::Right((BufReader::new(r), w)))
}

fn b64_equal(s: &str, b: &[u8]) -> bool {
    let mut eq = B64Eq(s, true);
    write!(&mut eq, "{}", Base64Display::standard(b)).unwrap();
    eq.1 && eq.0 == ""
}

// only use this on URIs that are already verified to have an authority segment
fn domain(uri: &Uri) -> &str {
    uri.authority_part().unwrap().host()
}

fn host_port(uri: &Uri) -> &str {
    let authority = uri.authority_part().unwrap().as_str();
    if let Some(i) = authority.find("@") {
        &authority[i + 1..]
    } else {
        authority
    }
}

fn get_addr(uri: &Uri, default_port: u16) -> DResult<SocketAddr> {
    let authority = uri.authority_part().ok_or(WError::InvalidUri)?;
    let addr = (authority.host(), authority.port().unwrap_or(default_port))
        .to_socket_addrs()?
        .nth(0)
        .ok_or(WError::ServerNotFound)?;

    Ok(addr)
}

fn hash_sec_key(key: &[u8; 16]) -> [u8; 20] {
    let mut key_hasher = Sha1::new();
    write!(
        Sha1Write(&mut key_hasher),
        "{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11",
        Base64Display::standard(key)
    ).unwrap();
    key_hasher.digest().bytes()
}

type PlainRead = BufReader<ReadHalf<TcpStream>>;
type TlsRead = BufReader<ReadHalf<TlsStream<TcpStream>>>;
type PlainWrite = WriteHalf<TcpStream>;
type TlsWrite = WriteHalf<TlsStream<TcpStream>>;

type Connection = Either<(PlainRead, PlainWrite), (TlsRead, TlsWrite)>;

struct DisplayHeaders<'a>(&'a [String]);
impl<'a> fmt::Display for DisplayHeaders<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for h in self.0.iter() {
            fmt.write_str(h)?;
            fmt.write_str("\r\n")?;
        }
        Ok(())
    }
}

struct Sha1Write<'a>(&'a mut Sha1);
impl<'a> fmt::Write for Sha1Write<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.update(s.as_bytes());
        Ok(())
    }
}

struct B64Eq<'a>(&'a str, bool);
impl<'a> fmt::Write for B64Eq<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if s.len() > self.0.len() {
            self.1 = false;
        } else if !self.0.starts_with(s) {
            self.0 = "";
            self.1 = false;
        } else {
            self.0 = &self.0[s.len()..];
        }
        Ok(())
    }
}
