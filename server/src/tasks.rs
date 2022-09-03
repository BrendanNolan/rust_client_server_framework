use super::command::{self, Command};
use connection_utils::{stream_handling, Communicable, TriviallyThreadable};
use threadpool::ThreadPool;
use tokio::{
    io,
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
    sync::oneshot,
};

pub async fn create_connection_manager<R, S>(stream: TcpStream, tx: Sender<Command<R, S>>)
where
    S: Communicable,
    R: Communicable,
{
    let (mut reader, mut writer) = io::split(stream);
    while let Some(data) = stream_handling::receive::<R>(&mut reader).await {
        let (responder, response_receiver) = oneshot::channel::<S>();
        let command = Command { data, responder };
        tx.send(command).await.unwrap();
        let response = response_receiver.await.unwrap();
        stream_handling::send(&response, &mut writer).await;
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
}
