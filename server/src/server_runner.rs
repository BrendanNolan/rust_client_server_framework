use crate::{
    jobs, manage_connection, request_processing::RequestProcessor, shutdown::ShutdownListener,
};
use connection_utils::{Communicable, TriviallyThreadable};
use tokio::{
    net::{TcpListener, ToSocketAddrs},
    task::JoinHandle,
};

pub async fn run_server<
    Req: Communicable,
    Resp: Communicable,
    Address: ToSocketAddrs,
    Processor: RequestProcessor<Req, Resp> + TriviallyThreadable,
>(
    address: Address,
    request_processor: Processor,
    shutdown_listener: ShutdownListener,
    jobs_buffer_size: usize,
    read_write_buffer_size: usize,
) {
    let connection_listener = TcpListener::bind(address).await.unwrap();
    let (job_dispatcher, jobs_task) = jobs::spawn_jobs_task(
        request_processor,
        jobs_buffer_size,
    );
    let mut connection_tasks = Vec::new();
    while let Ok((stream, _)) = connection_listener.accept().await {
        let connection_task = tokio::spawn(manage_connection::create_connection_manager(
            stream,
            job_dispatcher.clone(),
            shutdown_listener.clone(),
            read_write_buffer_size,
        ));
        connection_tasks.push(connection_task);
    }
    join_tasks(connection_tasks).await;
    jobs_task.await.unwrap();
}

async fn join_tasks(tasks: Vec<JoinHandle<()>>) {
    for task in tasks {
        task.await.unwrap();
    }
}
