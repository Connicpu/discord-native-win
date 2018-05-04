use error::{ApiError, DResult, Error};

use futures::prelude::*;
use http::uri::{Authority, Parts, PathAndQuery, Scheme, Uri};
use hyper::body::Payload;
use hyper::client::{Client, HttpConnector};
use hyper_tls::HttpsConnector;
use serde::de;
use serde_json;

pub mod gateway;

lazy_static! {
    pub static ref CLIENT: Client<HttpsConnector<HttpConnector>> =
        Client::builder().build(HttpsConnector::new(4).unwrap());
    static ref API_AUTHORITY: Authority = "discordapp.com".parse().unwrap();
}

pub fn get_data<T>(endpoint: &str) -> impl Future<Item = T, Error = Error>
where
    T: for<'de> de::Deserialize<'de>,
{
    do_get_data(endpoint.parse().unwrap())
}

#[async]
fn do_get_data<T>(endpoint: PathAndQuery) -> DResult<T>
where
    T: for<'de> de::Deserialize<'de>,
{
    let mut parts = Parts::default();
    parts.scheme = Some(Scheme::HTTPS);
    parts.authority = Some(API_AUTHORITY.clone());
    parts.path_and_query = Some(endpoint);
    let uri = Uri::from_parts(parts).unwrap();

    let response = await!(CLIENT.get(uri))?;

    if !response.status().is_success() {
        //eprintln!("{:?}", response);
        return Err(ApiError::UnknownEndpointError.into());
    }

    let body = response.into_body();
    let len = body.content_length().unwrap_or(128) as usize;
    let mut data = Vec::with_capacity(len);

    #[async]
    for chunk in body {
        data.extend_from_slice(&chunk);
    }

    Ok(serde_json::from_slice(&data)?)
}
