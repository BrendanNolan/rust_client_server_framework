use super::command::{self, Command};
use async_trait::async_trait;
use connection_utils::{
    incremental_read::IncrementalReadStorage, stream_serialization, Communicable, DataPoster,
    TriviallyThreadable,
};
use futures::stream::{FuturesUnordered, StreamExt};
use threadpool::ThreadPool;
use tokio::{io, net::TcpStream, sync::mpsc, sync::oneshot, task};

struct TaggedDataPoster<D, R>
where
    D: Communicable,
    R: Communicable,
{
    tx: mpsc::Sender<Command<D, R>>,
    responder: mpsc::Sender<R>,
}

#[async_trait]
impl<D, R> DataPoster<D> for TaggedDataPoster<D, R>
where
    D: Communicable,
    R: Communicable,
{
    async fn post(&self, data: D) {
        self.tx
            .send(Command {
                data,
                responder: self.responder,
            })
            .await
            .unwrap()
    }
}

pub async fn create_connection_manager<R, S>(stream: TcpStream, tx: mpsc::Sender<Command<R, S>>)
where
    S: Communicable,
    R: Communicable,
{
    let (tx_s, mut rx_s) = mpsc::channel::<S>(10);
    let poster = TaggedDataPoster {
        tx,
        responder: tx_s,
    };
}

async fn dispatch_job<S, R>(data: S, tx: &mpsc::Sender<Command<S, R>>) -> oneshot::Receiver<R>
where
    S: Communicable,
    R: Communicable,
{
    let (responder, response_receiver) = oneshot::channel::<R>();
    let command = Command { data, responder };
    tx.send(command).await.unwrap();
    response_receiver
}

pub async fn create_job_handler<R, S, F>(mut rx: mpsc::Receiver<Command<R, S>>, f: F)
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
