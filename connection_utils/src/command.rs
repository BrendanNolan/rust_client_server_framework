use crate::Communicable;
use std::fmt::{Debug, Error, Formatter};
use tokio::sync::oneshot;

pub struct Command<S, R>
where
    S: Communicable,
    R: Communicable,
{
    pub data: S,
    pub responder: oneshot::Sender<R>,
}

impl<S, R> Debug for Command<S, R>
where
    S: Communicable,
    R: Communicable,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.data.fmt(f)
    }
}
