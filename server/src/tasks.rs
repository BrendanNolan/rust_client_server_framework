use super::command::{self, Command};
use connection_utils::{stream_handling, Communicable, TriviallyThreadable};
use futures::executor::ThreadPool;
use tokio::{
    io,
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
};

pub async fn create_connection_manager<R, S>(stream: TcpStream, tx: Sender<Command<R, S>>)
where
    S: Communicable,
    R: Communicable,
{
    let (mut reader, mut writer) = io::split(stream);
    let (responder, mut response_receiver) = mpsc::channel::<S>(10);
    while let Ok(data) = stream_handling::receive::<R>(&mut reader).await {
        let responder = responder.clone();
        let command = Command { data, responder };
        tx.send(command).await.unwrap();
        let response = response_receiver.recv().await.unwrap();
        stream_handling::send(&response, &mut writer).await;
    }
    println!("Shutting down connection on server side.");
}

pub async fn create_job_handler<R, S, F>(mut rx: Receiver<Command<R, S>>, f: F)
where
    R: Communicable,
    S: Communicable,
    F: FnOnce(&R) -> S + TriviallyThreadable + Copy,
{
    let pool = ThreadPool::new().unwrap();
    while let Some(command) = rx.recv().await {
        pool.spawn_ok(async move {
            command::process(command, f).await;
        });
    }
}
