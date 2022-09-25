use crate::jobs::JobDispatcher;
use connection_utils::{stream_serialization, Communicable};
use futures::{stream::FuturesUnordered, StreamExt};
use tokio::{
    io::{self, ReadHalf, WriteHalf},
    net::TcpStream,
    sync::{mpsc, oneshot},
};

pub async fn create_connection_manager<ReceiveType, SendType>(
    stream: TcpStream,
    job_dispatcher: JobDispatcher<ReceiveType, SendType>,
    read_write_buffer_size: usize,
) where
    ReceiveType: Communicable,
    SendType: Communicable,
{
    let (reader, writer) = io::split(stream);
    let (tx, rx) = mpsc::channel(read_write_buffer_size);
    let read_task = tokio::spawn(create_read_task(reader, job_dispatcher, tx));
    let write_task = tokio::spawn(create_write_task(writer, rx));
    read_task.await.unwrap();
    write_task.await.unwrap();
    println!("Client disconnected; shutting down server connection.")
}

async fn create_read_task<ReceiveType, SendType>(
    mut reader: ReadHalf<TcpStream>,
    job_dispatcher: JobDispatcher<ReceiveType, SendType>,
    tx: mpsc::Sender<oneshot::Receiver<SendType>>,
) where
    ReceiveType: Communicable,
    SendType: Communicable,
{
    while let Ok(data) = stream_serialization::receive::<ReceiveType>(&mut reader).await {
        let response_receiver = job_dispatcher.dispatch_job(data).await;
        let _ = tx.send(response_receiver).await;
    }
}

async fn create_write_task<SendType>(
    mut writer: WriteHalf<TcpStream>,
    mut rx: mpsc::Receiver<oneshot::Receiver<SendType>>,
) where
    SendType: Communicable,
{
    let mut response_receivers = FuturesUnordered::new();
    loop {
        tokio::select! {
            Some(receiver) = rx.recv() => {
                response_receivers.push(receiver);
            },
            Some(Ok(response)) = response_receivers.next() => {
                let _ = stream_serialization::send(&response, &mut writer).await;
            },
            else => break,
        }
    }
}
