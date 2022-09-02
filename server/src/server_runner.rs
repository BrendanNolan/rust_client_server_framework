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
    let jobs_task = tokio::spawn(tasks::create_job_handler(rx, f));
    let mut connection_tasks = Vec::new();
    while let Ok((stream, _)) = listener.accept().await {
        let connection_task = tokio::spawn(tasks::create_connection_manager(stream, tx.clone()));
        connection_tasks.push(connection_task);
    }
    for connection_task in connection_tasks {
        connection_task.await.unwrap();
    }
    jobs_task.await.unwrap();
}
