// This annotation exists as the code for queries was unused at the time of writing this.
// It may be useful in the future, so I don't delete it
#![allow(dead_code)]

use std::any::TypeId;
use std::marker::PhantomData;

use kraken::api::handler::common::schema::ApiErrorResponse;
use kraken::api::handler::common::schema::ApiStatusCode;
use reqwest::RequestBuilder;
use reqwest::Url;
use serde::de::value::UnitDeserializer;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::KrakenError;
use crate::KrakenClient;
use crate::KrakenResult;

impl KrakenClient {
    pub(crate) fn get(&self, relative_url: &str) -> KrakenRequest<(), ()> {
        KrakenRequest::new(self.client.get(self.build_url(relative_url.as_ref())))
    }
    pub(crate) fn post(&self, relative_url: &str) -> KrakenRequest<(), ()> {
        KrakenRequest::new(self.client.post(self.build_url(relative_url.as_ref())))
    }
    pub(crate) fn put(&self, relative_url: &str) -> KrakenRequest<(), ()> {
        KrakenRequest::new(self.client.put(self.build_url(relative_url.as_ref())))
    }
    pub(crate) fn delete(&self, relative_url: &str) -> KrakenRequest<(), ()> {
        KrakenRequest::new(self.client.delete(self.build_url(relative_url.as_ref())))
    }
    fn build_url(&self, relative_url: &str) -> Url {
        #[allow(clippy::expect_used)]
        self.base_url
            .join(relative_url)
            .expect("The endpoint url should be valid")
    }
}

pub(crate) struct KrakenRequest<BOD, QUE> {
    inner: RequestBuilder,
    phantoms: PhantomData<(BOD, QUE)>,
}

impl<BOD, QUE> KrakenRequest<BOD, QUE> {
    fn new(inner: RequestBuilder) -> Self {
        Self {
            inner,
            phantoms: PhantomData,
        }
    }
}

impl<BOD> KrakenRequest<BOD, ()> {
    pub(crate) fn query<QUE>(self, query: QUE) -> KrakenRequest<BOD, QUE>
    where
        QUE: Serialize,
    {
        KrakenRequest::new(self.inner.query(&query))
    }
}

impl<QUE> KrakenRequest<(), QUE> {
    pub(crate) fn body<BOD>(self, body: BOD) -> KrakenRequest<BOD, QUE>
    where
        BOD: Serialize,
    {
        KrakenRequest::new(self.inner.json(&body))
    }
}

impl<BOD, QUE> KrakenRequest<BOD, QUE> {
    pub(crate) async fn send<RES>(self) -> KrakenResult<RES>
    where
        RES: DeserializeOwned + 'static,
    {
        let response = self.inner.send().await?;

        let status = response.status();
        let txt = response.text().await?;
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
