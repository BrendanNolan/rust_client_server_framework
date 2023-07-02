use crate::command::Command;
pub use connection_utils::ServerError;
use connection_utils::{stream_serialization, Communicable};
use tokio::{
    io,
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
};

pub async fn create_connection_manager<Req: Communicable, Resp: Communicable>(
    stream: TcpStream,
    rx: Receiver<Req>,
    tx: Sender<Result<Resp, ServerError>>,
) {
    let (reader, writer) = io::split(stream);
    let request_loop_handle = tokio::spawn(run_request_loop(rx, writer));
    let response_loop_handle = tokio::spawn(run_response_loop(reader, tx));
    let _ = request_loop_handle.await;
    let _ = response_loop_handle.await;
}

async fn run_response_loop<Resp: Communicable>(
    mut reader: io::ReadHalf<TcpStream>,
    tx: Sender<Result<Resp, ServerError>>,
) {
    while let Ok(response) =
        stream_serialization::receive::<Result<Resp, ServerError>>(&mut reader).await
    {
        let _ = tx.send(response).await;
    }
}

async fn run_request_loop<Req: Communicable>(
    mut rx: Receiver<Req>,
    mut writer: io::WriteHalf<TcpStream>,
) {
    while let Some(request) = rx.recv().await {
        let _ = stream_serialization::send(&request, &mut writer).await;
    }
}

// Will not send a new request until it receives a response from the previous request.
// Responds with None if it couldn't read the response from the socket or deserlialize
// the response.
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
