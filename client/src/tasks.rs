use crate::command::Command;
use connection_utils::{stream_serialization, Communicable, ServerError};
use tokio::{
    io,
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
};

pub async fn create_connection_manager<Req: Communicable, Resp: Communicable>(
    stream: TcpStream,
    mut rx: Receiver<Req>,
    tx: Sender<Resp>,
) {
    let (mut reader, mut writer) = io::split(stream);
    loop {
        tokio::select! {
            request = rx.recv() => {
                let Some(request) = request else {
                    println!("Got a bad request.");
                    break;
                };
                let _ = stream_serialization::send(&request, &mut writer).await;
            },
            response = stream_serialization::receive::<Resp>(&mut reader) => {
                let Ok(response) = response else {
                    println!("Received a bad response");
                    break;
                };
                let _ = tx.send(response).await;
            },
        }
    }
}

// Will not send a new request until it receives a response from the previous request.
pub async fn create_cyclic_connection_manager<S: Communicable, R: Communicable>(
    stream: TcpStream,
    mut rx: Receiver<Command<S, R>>,
) {
    let (mut reader, mut writer) = io::split(stream);
    while let Some(command) = rx.recv().await {
        if stream_serialization::send(&command.data, &mut writer)
            .await
            .is_err()
        {
            break;
        }
        let received = stream_serialization::receive::<Result<R, ServerError>>(&mut reader).await;
        command.responder.send(received.ok()).unwrap();
    }
}
