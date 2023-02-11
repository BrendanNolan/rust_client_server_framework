use crate::{jobs::JobDispatcher, shutdown::ShutdownListener};
use connection_utils::{stream_serialization, Communicable, ServerError};
use futures::{stream::FuturesUnordered, StreamExt};
use tokio::{
    io::{self, ReadHalf, WriteHalf},
    net::TcpStream,
    sync::{mpsc, oneshot},
};

pub async fn create_connection_manager<Req: Communicable, Resp: Communicable>(
    stream: TcpStream,
    job_dispatcher: JobDispatcher<Req, Resp>,
    shutdown_listener: ShutdownListener,
    read_write_buffer_size: usize,
) {
    let (reader, writer) = io::split(stream);
    let (tx, rx) = mpsc::channel(read_write_buffer_size);
    let read_task = tokio::spawn(create_read_task(
        reader,
        job_dispatcher,
        tx,
        shutdown_listener.clone(),
    ));
    let write_task = tokio::spawn(create_write_task(writer, rx, shutdown_listener));
    read_task.await.unwrap();
    write_task.await.unwrap();
    println!("Shutting down connection to client.")
}

async fn create_read_task<Req: Communicable, Resp: Communicable>(
    reader: ReadHalf<TcpStream>,
    job_dispatcher: JobDispatcher<Req, Resp>,
    tx: mpsc::Sender<oneshot::Receiver<Resp>>,
    mut shutdown_listener: ShutdownListener,
) {
    tokio::select! {
        _ = shutdown_listener.listen_for_shutdown_signal() => {},
        _ = perform_reading(reader, job_dispatcher, tx) => {},
    }
}

async fn perform_reading<Req: Communicable, Resp: Communicable>(
    mut reader: ReadHalf<TcpStream>,
    job_dispatcher: JobDispatcher<Req, Resp>,
    tx: mpsc::Sender<oneshot::Receiver<Resp>>,
) {
    while let Ok(data) = stream_serialization::receive::<Req>(&mut reader).await {
        let response_receiver = job_dispatcher.dispatch_job(data).await;
        let _ = tx.send(response_receiver).await;
    }
}

async fn create_write_task<Resp: Communicable>(
    mut writer: WriteHalf<TcpStream>,
    mut rx: mpsc::Receiver<oneshot::Receiver<Resp>>,
    mut shutdown_listener: ShutdownListener,
) {
    let mut response_receivers = FuturesUnordered::new();
    loop {
        tokio::select! {
            receiver = rx.recv() => {
                let Some(receiver) = receiver else { return; };
                response_receivers.push(receiver);
            },
            Some(Ok(response)) = response_receivers.next() => {
                let response: Result<Resp, ServerError> = Ok(response);
                let _ = stream_serialization::send(&response, &mut writer).await;
            },
            _ = shutdown_listener.listen_for_shutdown_signal() => {
                let server_error = ServerError { error_message: "Server Has Been Shut Down.".to_string() };
                let server_error: Result<Resp, ServerError> = Err(server_error);
                let _ = stream_serialization::send(&server_error, &mut writer).await;
                return;
            },
            else => return,
        }
    }
}
