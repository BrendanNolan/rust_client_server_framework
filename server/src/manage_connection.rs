use crate::jobs::JobDispatcher;
use connection_utils::{stream_serialization, Communicable};
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
    while let Some(response_receiver) = rx.recv().await {
        if let Ok(response) = response_receiver.await {
            if stream_serialization::send(&response, &mut writer)
                .await
                .is_err()
            {
                break;
            }
        }
    }
}
