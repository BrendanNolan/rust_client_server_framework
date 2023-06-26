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
    tx: Sender<Option<Result<Resp, ServerError>>>,
) {
    let (mut reader, mut writer) = io::split(stream);
    let request_loop_handle = tokio::spawn(async move {
        while let Some(request) = rx.recv().await {
            let _ = stream_serialization::send(&request, &mut writer).await;
        }
    });
    let response_loop_handle = tokio::spawn(async move {
        while let Ok(response) =
            stream_serialization::receive::<Result<Resp, ServerError>>(&mut reader).await
        {
            let _ = tx.send(Some(response)).await;
        }
    });
    let _ = request_loop_handle.await;
    let _ = response_loop_handle.await;
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
