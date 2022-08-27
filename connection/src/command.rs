use std::fmt::{Debug, Error, Formatter};

use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::oneshot::Sender;

pub struct Command<S, R>
where
    S: Serialize + Debug,
    R: DeserializeOwned + Debug,
{
    pub to_send: S,
    pub responder: Sender<Option<R>>,
}

impl<S, R> Debug for Command<S, R>
where
    S: Serialize + Debug,
    R: DeserializeOwned + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.to_send.fmt(f)
    }
}
