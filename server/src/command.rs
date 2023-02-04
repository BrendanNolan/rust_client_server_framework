use crate::request_processing::RequestProcessor;
use connection_utils::Communicable;
use std::fmt::{Debug, Error, Formatter};
use tokio::sync::oneshot;

pub struct Command<Req: Communicable, Resp: Communicable> {
    pub data: Req,
    pub responder: oneshot::Sender<Resp>,
}

impl<Req: Communicable, Resp: Communicable> Debug for Command<Req, Resp> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.data.fmt(f)
    }
}

pub fn process<Req: Communicable, Resp: Communicable, P: RequestProcessor<Req, Resp>>(
    command: Command<Req, Resp>,
    p: &P,
) {
    let response = p.process(&command.data);
    command.responder.send(response).unwrap();
}
