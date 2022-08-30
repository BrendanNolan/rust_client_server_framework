use connection_utils::{command::Command, command::Communicable, stream_handling};
use tokio::{io, net::TcpStream, sync::mpsc::Sender, sync::oneshot};

pub async fn create_connection_manager<R, S>(stream: TcpStream, tx: Sender<Command<R, S>>)
where
    S: Communicable,
    R: Communicable,
{
    let (mut reader, mut writer) = io::split(stream);
    loop {
        let message = stream_handling::receive::<R>(&mut reader).await.unwrap();
        let (tx_oneshot, rx_oneshot) = oneshot::channel::<Option<S>>();
        let command = Command {
            to_send: message,
            responder: tx_oneshot,
        };
        tx.send(command).await.unwrap(); // The receiver should use command.responder to let us know the result of the job.
        if let Ok(response) = rx_oneshot.await {
            stream_handling::send(&response, &mut writer).await;
        }
    }
}
