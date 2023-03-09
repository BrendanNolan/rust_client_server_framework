use crate::{
    jobs::{self, JobDispatcher},
    manage_connection,
    request_processing::RequestProcessor,
    shutdown::ShutdownListener,
};
use connection_utils::{Communicable, TriviallyThreadable};
use futures::future::join_all;
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
    let (job_dispatcher, jobs_task) = jobs::spawn_jobs_task(
        request_processor,
        jobs_buffer_size,
        shutdown_listener.clone(),
    );
    let connection_tasks = run_server_loop(
        address,
        job_dispatcher,
        read_write_buffer_size,
        shutdown_listener,
    )
    .await;
    join_all(connection_tasks).await;
    let _ = jobs_task.await;
}

async fn run_server_loop<Req: Communicable, Resp: Communicable, Address: ToSocketAddrs>(
    address: Address,
    job_dispatcher: JobDispatcher<Req, Resp>,
    read_write_buffer_size: usize,
    mut shutdown_listener: ShutdownListener,
) -> Vec<JoinHandle<()>> {
    let mut connection_tasks = vec![];
    let connection_listener = TcpListener::bind(address).await.unwrap();
    while !shutdown_listener.is_shutting_down() {
        tokio::select! {
            connection = try_establish_connection(
                &connection_listener,
                job_dispatcher.clone(),
                read_write_buffer_size,
                shutdown_listener.clone())
            => {
                if let Some(connection) = connection {
                    connection_tasks.push(connection);
                } else {
                    println!("Failed to accept client connection.");
                }
            }
            _ = shutdown_listener.listen_for_shutdown_signal() => {},
        }
    }
    connection_tasks
}

async fn try_establish_connection<Req: Communicable, Resp: Communicable>(
    connection_listener: &TcpListener,
    job_dispatcher: JobDispatcher<Req, Resp>,
    read_write_buffer_size: usize,
    shutdown_listener: ShutdownListener,
) -> Option<JoinHandle<()>> {
    let (stream, _) = connection_listener.accept().await.ok()?;
    println!("Accetped Connection.");
    let connection_task = tokio::spawn(manage_connection::create_connection_manager(
        stream,
        job_dispatcher,
        shutdown_listener,
        read_write_buffer_size,
    ));
    Some(connection_task)
}
