use connection_utils::Communicable;
use std::fmt::{Debug, Error, Formatter};
use tokio::sync::oneshot::Sender;

pub struct Command<S, R>
where
    S: Communicable,
    R: Communicable,
{
    pub data: S,
    pub responder: Sender<R>, // TODO: Is this always how we want to communicate?
}

pub fn process<F, S, R>(command: Command<S, R>, f: F)
where
    S: Communicable,
    R: Communicable,
    F: FnOnce(&S) -> R,
{
    let response = f(&command.data);
    command.responder.send(response).unwrap();
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
