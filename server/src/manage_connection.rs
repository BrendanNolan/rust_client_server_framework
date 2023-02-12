use crate::{jobs::JobDispatcher, shutdown::ShutdownListener};
use connection_utils::{stream_serialization, Communicable};
use futures::{stream::FuturesUnordered, StreamExt};
use tokio::{
    io::{self, ReadHalf, WriteHalf},
    net::TcpStream,
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

pub async fn create_connection_manager<Req: Communicable, Resp: Communicable>(
    stream: TcpStream,
    job_dispatcher: JobDispatcher<Req, Resp>,
    mut shutdown_listener: ShutdownListener,
    read_write_buffer_size: usize,
) {
    let (reader, writer) = io::split(stream);
    let (tx, rx) = mpsc::channel(read_write_buffer_size);
    let read_task = tokio::spawn(create_read_task(reader, job_dispatcher, tx));
    let (tx_misc, write_task) = spawn_write_task(writer, rx);
    tokio::select! {
        _ = shutdown_listener.listen_for_shutdown_signal() => {
            let _ = tx_misc.send(make_shutdown_response()).await;
            return;
        },
        _ = {
            let _ = read_task.await;
            write_task
        } => {
            println!("Client disconnected; shutting down server connection.")
        },
    }
}

fn make_shutdown_response<Resp: Communicable>() -> Resp {
    !todo!();
}

async fn create_read_task<Req: Communicable, Resp: Communicable>(
    mut reader: ReadHalf<TcpStream>,
    job_dispatcher: JobDispatcher<Req, Resp>,
    tx: mpsc::Sender<oneshot::Receiver<Resp>>,
) {
    while let Ok(data) = stream_serialization::receive::<Req>(&mut reader).await {
        let response_receiver = job_dispatcher.dispatch_job(data).await;
        let _ = tx.send(response_receiver).await;
    }
}

fn spawn_write_task<Resp: Communicable>(writer: WriteHalf<TcpStream>,
    rx: mpsc::Receiver<oneshot::Receiver<Resp>>,) -> (mpsc::Sender<Resp>, JoinHandle<()>) {
    let (tx_misc, rx_misc) = mpsc::channel(147);
    let write_task = tokio::spawn(create_write_task(writer, rx, rx_misc));
    (tx_misc, write_task)
}

async fn create_write_task<Resp: Communicable>(
    mut writer: WriteHalf<TcpStream>,
    mut rx: mpsc::Receiver<oneshot::Receiver<Resp>>,
    mut rx_misc: mpsc::Receiver<Resp>,
) {
    let mut response_receivers = FuturesUnordered::new();
    loop {
        tokio::select! {
            Some(receiver) = rx.recv() => {
                response_receivers.push(receiver);
            },
            Some(Ok(response)) = response_receivers.next() => {
                let _ = stream_serialization::send(&response, &mut writer).await;
            },
            Some(response) = rx_misc.recv() => {
                let _ = stream_serialization::send(&response, &mut writer).await;
            }
            else => break,
        }
    }
}
