use connection_utils::Communicable;
use std::fmt::{Debug, Error, Formatter};
use tokio::sync::oneshot;

pub struct Command<Request, Response>
where
    Request: Communicable,
    Response: Communicable,
{
    pub data: Request,
    pub responder: oneshot::Sender<Response>,
}

pub fn process<Operation, Request, Response>(command: Command<Request, Response>, f: Operation)
where
    Request: Communicable,
    Response: Communicable,
    Operation: FnOnce(&Request) -> Response,
{
    let response = f(&command.data);
    command.responder.send(response).unwrap();
}

impl<Request, Response> Debug for Command<Request, Response>
where
    Request: Communicable,
    Response: Communicable,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.data.fmt(f)
    }
}
