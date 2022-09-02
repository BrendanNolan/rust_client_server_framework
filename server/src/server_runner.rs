use super::tasks;
use connection_utils::{command::Command, Communicable, TriviallyThreadable};
use tokio::{
    net::{TcpListener, ToSocketAddrs},
    sync::mpsc::{self},
};

pub async fn run_server<R, S, A, F>(address: A, f: F)
where
    R: Communicable,
    S: Communicable,
    A: ToSocketAddrs,
    F: FnOnce(&R) -> Option<S> + TriviallyThreadable + Copy,
{
    let (tx, rx) = mpsc::channel::<Command<R, S>>(10);
    let listener = TcpListener::bind(address).await.unwrap();
    tokio::spawn(tasks::create_job_handler(rx, f));
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(tasks::create_connection_manager(stream, tx.clone()));
    }
}
