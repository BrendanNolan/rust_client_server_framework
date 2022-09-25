use crate::jobs::JobDispatcher;
use connection_utils::{stream_serialization, Communicable};
use futures::{stream::FuturesUnordered, StreamExt};
use tokio::{
    io::{self, ReadHalf, WriteHalf},
    net::TcpStream,
    sync::{mpsc, oneshot},
};

pub async fn create_connection_manager<Req, Resp>(
    stream: TcpStream,
    job_dispatcher: JobDispatcher<Req, Resp>,
    read_write_buffer_size: usize,
) where
    Req: Communicable,
    Resp: Communicable,
{
    let (reader, writer) = io::split(stream);
    let (tx, rx) = mpsc::channel(read_write_buffer_size);
    let read_task = tokio::spawn(create_read_task(reader, job_dispatcher, tx));
    let write_task = tokio::spawn(create_write_task(writer, rx));
    read_task.await.unwrap();
    write_task.await.unwrap();
    println!("Client disconnected; shutting down server connection.")
}

async fn create_read_task<Req, Resp>(
    mut reader: ReadHalf<TcpStream>,
    job_dispatcher: JobDispatcher<Req, Resp>,
    tx: mpsc::Sender<oneshot::Receiver<Resp>>,
) where
    Req: Communicable,
    Resp: Communicable,
{
    while let Ok(data) = stream_serialization::receive::<Req>(&mut reader).await {
        let response_receiver = job_dispatcher.dispatch_job(data).await;
        let _ = tx.send(response_receiver).await;
    }
}

async fn create_write_task<Resp>(
    mut writer: WriteHalf<TcpStream>,
    mut rx: mpsc::Receiver<oneshot::Receiver<Resp>>,
) where
    Resp: Communicable,
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
