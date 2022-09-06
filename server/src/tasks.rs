use super::command::{self, Command};
use connection_utils::{
    incremental_read::IncrementalReadStorage, stream_serialization, Communicable,
    TriviallyThreadable,
};
use futures::stream::{FuturesUnordered, StreamExt};
use threadpool::ThreadPool;
use tokio::{
    io,
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
    sync::oneshot,
    task,
};

pub async fn create_connection_manager<R, S>(stream: TcpStream, tx: Sender<Command<R, S>>)
where
    S: Communicable,
    R: Communicable,
{
    let (mut reader, mut writer) = io::split(stream);
    let mut response_receivers = FuturesUnordered::new();
    let mut stream_read_storage = IncrementalReadStorage::default();
    loop {
        tokio::select! {
            _ = stream_read_storage.progress_filling(&mut reader) => {
                if stream_read_storage.is_full() {
                    dispatch_job_from_stored_data(&stream_read_storage, &mut response_receivers, &tx).await;
                    stream_read_storage.reset();
                }
            },
            Some(Ok(response)) = response_receivers.next() => {
                stream_serialization::send(&response, &mut writer).await;
            },
            else => break,
        }
    }
}

async fn dispatch_job_from_stored_data<S, R>(
    stream_read_storage: &IncrementalReadStorage,
    response_receivers: &mut FuturesUnordered<oneshot::Receiver<R>>,
    tx: &Sender<Command<S, R>>,
) where
    S: Communicable,
    R: Communicable,
{
    let data = bincode::deserialize::<S>(&stream_read_storage.buffer).unwrap();
    let response_receiver = dispatch_job(data, tx).await;
    response_receivers.push(response_receiver);
}

async fn dispatch_job<S, R>(data: S, tx: &Sender<Command<S, R>>) -> oneshot::Receiver<R>
where
    S: Communicable,
    R: Communicable,
{
    let (responder, response_receiver) = oneshot::channel::<R>();
    let command = Command { data, responder };
    tx.send(command).await.unwrap();
    response_receiver
}

pub async fn create_job_handler<R, S, F>(mut rx: Receiver<Command<R, S>>, f: F)
where
    R: Communicable,
    S: Communicable,
    F: FnOnce(&R) -> S + TriviallyThreadable + Copy,
{
    let pool = ThreadPool::new(8);
    while let Some(command) = rx.recv().await {
        pool.execute(move || {
            command::process(command, f);
        });
    }
    task::spawn_blocking(move || {
        pool.join();
    });
}
