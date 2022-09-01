use std::fmt::{Debug, Error, Formatter};

use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::oneshot::Sender;

pub trait Communicable: Serialize + DeserializeOwned + Debug {}
impl<T: Serialize + DeserializeOwned + Debug> Communicable for T {}

pub struct Command<S: Communicable, R: Communicable> {
    pub data: S,
    pub responder: Sender<Option<R>>, // TODO: Is this always how we want to communicate?
}

impl<S: Communicable, R: Communicable> Debug for Command<S, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.data.fmt(f)
    }
}
