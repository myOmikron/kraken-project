// This annotation exists as the code for queries was unused at the time of writing this.
// It may be useful in the future, so I don't delete it
#![allow(dead_code)]

use std::any::TypeId;

use kraken::api::handler::common::schema::{ApiErrorResponse, ApiStatusCode};
use reqwest::Url;
use serde::de::value::UnitDeserializer;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::KrakenError;
use crate::{KrakenClient, KrakenResult};

pub(crate) enum Method {
    Get,
    Post,
    Put,
    Delete,
}

pub(crate) struct KrakenRequest<BOD, QUE>
where
    BOD: Serialize,
    QUE: Serialize,
{
    pub method: Method,
    pub url: Url,
    pub query: Option<QUE>,
    pub body: Option<BOD>,
}

impl KrakenRequest<(), ()> {
    pub(crate) fn get(url: Url) -> KrakenRequestBuilder {
        KrakenRequestBuilder::new(Method::Get, url)
    }

    pub(crate) fn post(url: Url) -> KrakenRequestBuilder {
        KrakenRequestBuilder::new(Method::Post, url)
    }

    pub(crate) fn put(url: Url) -> KrakenRequestBuilder {
        KrakenRequestBuilder::new(Method::Put, url)
    }

    pub(crate) fn delete(url: Url) -> KrakenRequestBuilder {
        KrakenRequestBuilder::new(Method::Delete, url)
    }
}

pub(crate) struct KrakenRequestBuilder {
    method: Method,
    url: Url,
}

impl KrakenRequestBuilder {
    pub(crate) fn new(method: Method, url: Url) -> Self {
        Self { method, url }
    }

    pub(crate) fn body<BOD>(self, body: BOD) -> KrakenRequestBodyBuilder<BOD>
    where
        BOD: Serialize,
    {
        KrakenRequestBodyBuilder::new(self, body)
    }

    pub(crate) fn query<QUE>(self, query: QUE) -> KrakenRequestQueryBuilder<QUE>
    where
        QUE: Serialize,
    {
        KrakenRequestQueryBuilder::new(self, query)
    }

    pub(crate) fn build(self) -> KrakenRequest<(), ()> {
        KrakenRequest {
            method: self.method,
            url: self.url,
            query: None,
            body: None,
        }
    }
}

pub(crate) struct KrakenRequestBodyBuilder<BOD>
where
    BOD: Serialize,
{
    method: Method,
    url: Url,
    body: BOD,
}

impl<BOD> KrakenRequestBodyBuilder<BOD>
where
    BOD: Serialize,
{
    fn new(prev: KrakenRequestBuilder, body: BOD) -> Self {
        Self {
            method: prev.method,
            url: prev.url,
            body,
        }
    }

    pub(crate) fn query<QUE>(self, query: QUE) -> KrakenRequest<BOD, QUE>
    where
        QUE: Serialize,
    {
        KrakenRequest {
            method: self.method,
            url: self.url,
            query: Some(query),
            body: Some(self.body),
        }
    }

    pub fn build(self) -> KrakenRequest<BOD, ()> {
        KrakenRequest {
            method: self.method,
            url: self.url,
            query: None,
            body: Some(self.body),
        }
    }
}

pub(crate) struct KrakenRequestQueryBuilder<QUE>
where
    QUE: Serialize,
{
    method: Method,
    url: Url,
    query: QUE,
}

impl<QUE> KrakenRequestQueryBuilder<QUE>
where
    QUE: Serialize,
{
    fn new(prev: KrakenRequestBuilder, query: QUE) -> Self {
        Self {
            method: prev.method,
            url: prev.url,
            query,
        }
    }

    pub(crate) fn body<BOD>(self, body: BOD) -> KrakenRequest<BOD, QUE>
    where
        BOD: Serialize,
    {
        KrakenRequest {
            method: self.method,
            url: self.url,
            query: Some(self.query),
            body: Some(body),
        }
    }

    pub(crate) fn build(self) -> KrakenRequest<(), QUE> {
        KrakenRequest {
            method: self.method,
            url: self.url,
            query: Some(self.query),
            body: None,
        }
    }
}

impl KrakenClient {
    pub(crate) async fn make_request<QUE, BOD, RES>(
        &self,
        req: KrakenRequest<QUE, BOD>,
    ) -> KrakenResult<RES>
    where
        QUE: Serialize,
        BOD: Serialize,
        RES: DeserializeOwned + 'static,
    {
        let KrakenRequest {
            query,
            url,
            method,
            body,
        } = req;

        let mut rb = match method {
            Method::Get => self.client.get(url),
            Method::Post => self.client.post(url),
            Method::Put => self.client.put(url),
            Method::Delete => self.client.delete(url),
        };

        if let Some(query) = query {
            rb = rb.query(&query);
        }

        if let Some(req) = body {
            rb = rb.json(&req);
        }

        let res = rb.send().await?;

        let status = res.status();
        let txt = res.text().await?;
        if !status.is_success() {
            return if status == 400 || status == 500 {
                let Ok(err) = serde_json::from_str(&txt) else {
                    return Err(KrakenError::DeserializeError(txt));
                };
                let err: ApiErrorResponse = err;

                if err.status_code == ApiStatusCode::Unauthenticated {
                    return Err(KrakenError::AuthenticationFailed);
                }

                Err(KrakenError::ApiError(err))
            } else {
                Err(KrakenError::DeserializeError(txt))
            };
        }

        if TypeId::of::<RES>() == TypeId::of::<()>() {
            // check above guarantees that RES is ()
            #[allow(clippy::unwrap_used)]
            return Ok(RES::deserialize(UnitDeserializer::<serde_json::Error>::new()).unwrap());
        }

        let Ok(deserialized) = serde_json::from_str(&txt) else {
            return Err(KrakenError::DeserializeError(txt));
        };

        Ok(deserialized)
    }
}
