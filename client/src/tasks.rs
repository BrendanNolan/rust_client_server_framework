use super::command::Command;
use connection_utils::{stream_handling, Communicable};
use tokio::{io, net::TcpStream, sync::mpsc::Receiver};

pub async fn create_connection_manager<S, R>(stream: TcpStream, mut rx: Receiver<Command<S, R>>)
where
    S: Communicable,
    R: Communicable,
{
    let (mut reader, mut writer) = io::split(stream);
    while let Some(command) = rx.recv().await {
        stream_handling::send(&command.data, &mut writer).await;
        let received = stream_handling::receive::<R>(&mut reader).await;
        command.responder.send(received.ok()).unwrap();
    }
}
