use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

pub trait TriviallyThreadable: Send + Sync + 'static {}
impl<T> TriviallyThreadable for T where T: Send + Sync + 'static {}

pub trait Communicable: 'static + Serialize + DeserializeOwned + Sync + Send + Debug {}
impl<T> Communicable for T where T: 'static + Serialize + DeserializeOwned + Sync + Send + Debug {}

#[async_trait]
pub trait DataPoster<D> {
    async fn post(&self, data: D);
}
