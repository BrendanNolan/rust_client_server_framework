use super::command::{self, Command};
use connection_utils::{stream_handling, Communicable, TriviallyThreadable};
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
    loop {
        tokio::select! {
            Ok(data) = stream_handling::receive::<R>(&mut reader) => {
                let (responder, response_receiver) = oneshot::channel::<S>();
                let command = Command { data, responder };
                tx.send(command).await.unwrap();
                response_receivers.push(response_receiver);
            },
            Some(Ok(response)) = response_receivers.next() => {
                stream_handling::send(&response, &mut writer).await;
            },
            else => break,
        }
    }
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
