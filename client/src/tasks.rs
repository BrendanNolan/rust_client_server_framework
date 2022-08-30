use connection_utils::command::Command;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use tokio::{io, net::TcpStream, sync::mpsc::Receiver};

pub async fn create_connection_manager<S, R>(stream: TcpStream, mut rx: Receiver<Command<S, R>>)
where
    S: Serialize + Debug,
    R: DeserializeOwned + Debug,
{
    let (mut reader, mut writer) = io::split(stream);
    while let Some(command) = rx.recv().await {
        connection_utils::read_write::send(&command.to_send, &mut writer).await;
        let received = connection_utils::read_write::receive::<R>(&mut reader).await;
        command.responder.send(received.ok()).unwrap();
    }
}
