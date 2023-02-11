use connection_utils::{Communicable, ServerError};
use std::fmt::{Debug, Error, Formatter};
use tokio::sync::oneshot::Sender;

pub struct Command<S: Communicable, R: Communicable> {
    pub data: S,
    pub responder: Sender<Option<Result<R, ServerError>>>,
}

impl<S: Communicable, R: Communicable> Debug for Command<S, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.data.fmt(f)
    }
}
