use connection_utils::Communicable;
use std::fmt::{Debug, Error, Formatter};
use tokio::sync::oneshot;

pub struct Command<Req, Resp>
where
    Req: Communicable,
    Resp: Communicable,
{
    pub data: Req,
    pub responder: oneshot::Sender<Resp>,
}

pub fn process<Req, Resp, Op>(command: Command<Req, Resp>, f: Op)
where
    Req: Communicable,
    Resp: Communicable,
    Op: FnOnce(&Req) -> Resp,
{
    let response = f(&command.data);
    command.responder.send(response).unwrap();
}

impl<Req, Resp> Debug for Command<Req, Resp>
where
    Req: Communicable,
    Resp: Communicable,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.data.fmt(f)
    }
}
