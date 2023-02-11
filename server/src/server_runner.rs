use crate::{
    jobs, manage_connection, request_processing::RequestProcessor, shutdown::ShutdownListener,
};
use connection_utils::{Communicable, TriviallyThreadable};
use futures::future::join_all;
use tokio::net::{TcpListener, ToSocketAddrs};

pub async fn run_server<
    Req: Communicable,
    Resp: Communicable,
    Address: ToSocketAddrs,
    Processor: RequestProcessor<Req, Resp> + TriviallyThreadable,
>(
    address: Address,
    request_processor: Processor,
    mut shutdown_listener: ShutdownListener,
    jobs_buffer_size: usize,
    read_write_buffer_size: usize,
) {
    let connection_listener = TcpListener::bind(address).await.unwrap();
    let (job_dispatcher, jobs_task) = jobs::spawn_jobs_task(
        request_processor,
        jobs_buffer_size,
        shutdown_listener.clone(),
    );
    let mut connection_tasks = vec![];
    while !shutdown_listener.is_shutting_down() {
        tokio::select! {
            connection_result = connection_listener.accept() => {
                match connection_result {
                    Ok((stream, _)) => {
                        println!("Accetped Connection.");
                        let connection_task = tokio::spawn(manage_connection::create_connection_manager(
                            stream,
                            job_dispatcher.clone(),
                            shutdown_listener.clone(),
                            read_write_buffer_size,
                        ));
                        connection_tasks.push(connection_task);
                    },
                    Err(_) => { println!("Failed to accept client connection."); },
                }
            },
            _ = shutdown_listener.listen_for_shutdown_signal() => {},
        }
    }
    join_all(connection_tasks).await;
    let _ = jobs_task.await;
}
