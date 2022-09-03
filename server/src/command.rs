use connection_utils::Communicable;
use std::fmt::{Debug, Error, Formatter};
use tokio::sync::mpsc;

pub struct Command<S, R>
where
    S: Communicable,
    R: Communicable,
{
    pub data: S,
    pub responder: mpsc::Sender<R>, // TODO: Is this always how we want to communicate?
}

pub async fn process<F, S, R>(command: Command<S, R>, f: F)
where
    S: Communicable,
    R: Communicable,
    F: FnOnce(&S) -> R,
{
    let response = f(&command.data);
    command.responder.send(response).await.unwrap();
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
