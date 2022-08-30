use std::fmt::{Debug, Error, Formatter};

use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::oneshot::Sender;

pub trait Communicable: Serialize + DeserializeOwned + Debug {}
impl<T> Communicable for T where T: Serialize + DeserializeOwned + Debug {}

pub struct Command<S, R>
where
    S: Communicable,
    R: Communicable,
{
    pub to_send: S,
    pub responder: Sender<Option<R>>, // TODO: Is this always how we want to communicate?
}

impl<S, R> Debug for Command<S, R>
where
    S: Communicable,
    R: Communicable,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.to_send.fmt(f)
    }
}
