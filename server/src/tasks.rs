use connection_utils::{command::Command, command::Communicable, stream_handling};
use tokio::{io, net::TcpStream, sync::mpsc::Sender, sync::oneshot};

pub async fn create_connection_manager<R, S>(stream: TcpStream, tx: Sender<Command<R, S>>)
where
    S: Communicable,
    R: Communicable,
{
    let (mut reader, mut writer) = io::split(stream);
    loop {
        let data = stream_handling::receive::<R>(&mut reader).await.unwrap();
        let (responder, response_receiver) = oneshot::channel::<Option<S>>();
        let command = Command { data, responder };
        tx.send(command).await.unwrap(); // The receiver should use command.responder to let us know the result of the job.
        if let Ok(response) = response_receiver.await {
            stream_handling::send(&response, &mut writer).await;
        }
    }
}
