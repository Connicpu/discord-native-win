use error::{ApiError, DResult, Error};

use std::sync::RwLock;

use futures::prelude::*;
use http::uri::{Authority, Parts, PathAndQuery, Scheme, Uri};
use hyper::body::Payload;
use hyper::client::{Client, HttpConnector};
use hyper_tls::HttpsConnector;
use serde::de;
use serde_json;

pub mod auth;
pub mod gateway;

lazy_static! {
    pub static ref CLIENT: ClientWrapper = ClientWrapper::new();
    static ref API_AUTHORITY: Authority = "discordapp.com".parse().unwrap();
}

pub fn dispose() {
    CLIENT.dispose();
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

    let response = await!(CLIENT.with(|c| c.get(uri)))?;

    if !response.status().is_success() {
        return Err(ApiError::UnknownEndpoint.into());
    }

    let body = response.into_body();
    let len = body.content_length().unwrap_or(256) as usize;
    let mut data = Vec::with_capacity(len);

    #[async]
    for chunk in body {
        data.extend_from_slice(&chunk);
    }

    Ok(serde_json::from_slice(&data)?)
}

pub struct ClientWrapper {
    client: RwLock<Option<Client<HttpsConnector<HttpConnector>>>>,
}

impl ClientWrapper {
    fn new() -> Self {
        ClientWrapper {
            client: RwLock::new(None),
        }
    }

    pub fn with<F, R>(&self, f: F) -> R where F: FnOnce(&Client<HttpsConnector<HttpConnector>>) -> R {
        if let Some(ref client) = *self.client.read().unwrap() {
            return f(client);
        }
        
        let mut slot = self.client.write().unwrap();
        if let Some(ref client) = *slot {
            return f(client);
        }

        let client = Client::builder().build(HttpsConnector::new(4).unwrap());
        let result = f(&client);
        *slot = Some(client);
        result
    }

    fn dispose(&self) {
        if let Ok(mut client) = self.client.write() {
            *client = None;
        }
    }
}
