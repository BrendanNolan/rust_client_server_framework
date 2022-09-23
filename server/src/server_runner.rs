use crate::{jobs, manage_connection};
use connection_utils::{Communicable, TriviallyThreadable};
use tokio::net::{TcpListener, ToSocketAddrs};

pub async fn run_server<ReceiveType, SendType, AddressType, Operation>(
    address: AddressType,
    f: Operation,
    jobs_buffer_size: usize,
) where
    ReceiveType: Communicable,
    SendType: Communicable,
    AddressType: ToSocketAddrs,
    Operation: FnOnce(&ReceiveType) -> SendType + TriviallyThreadable + Copy,
{
    let listener = TcpListener::bind(address).await.unwrap();
    let (job_dispatcher, jobs_task) = jobs::spawn_jobs_task(f, jobs_buffer_size);
    let mut connection_tasks = Vec::new();
    while let Ok((stream, _)) = listener.accept().await {
        let connection_task = tokio::spawn(manage_connection::create_connection_manager(
            stream,
            job_dispatcher.clone(),
        ));
        connection_tasks.push(connection_task);
    }
    for connection_task in connection_tasks {
        connection_task.await.unwrap();
    }
    jobs_task.await.unwrap();
}
