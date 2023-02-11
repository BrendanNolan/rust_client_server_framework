use crate::{
    command::{self, Command},
    request_processing::RequestProcessor,
    shutdown::ShutdownListener,
};
use connection_utils::{Communicable, TriviallyThreadable};
use std::sync::Arc;
use threadpool::ThreadPool;
use tokio::{
    sync::mpsc,
    sync::oneshot,
    task::{self, JoinHandle},
};

pub struct JobDispatcher<Req: Communicable, Resp: Communicable> {
    tx: mpsc::Sender<Command<Req, Resp>>,
}

impl<Req: Communicable, Resp: Communicable> JobDispatcher<Req, Resp> {
    pub async fn dispatch_job(&self, data: Req) -> oneshot::Receiver<Resp> {
        let (responder, response_receiver) = oneshot::channel::<Resp>();
        let command = Command { data, responder };
        self.tx.send(command).await.unwrap();
        response_receiver
    }
}

// #[derive(Clone)] does not work - it requires that both generic parameters implement Clone
impl<Req: Communicable, Resp: Communicable> Clone for JobDispatcher<Req, Resp> {
    fn clone(&self) -> Self {
        JobDispatcher {
            tx: self.tx.clone(),
        }
    }
}

pub fn spawn_jobs_task<
    Req: Communicable,
    Resp: Communicable,
    P: RequestProcessor<Req, Resp> + TriviallyThreadable,
>(
    p: P,
    buffer_size: usize,
    shutdown_listener: ShutdownListener,
) -> (JobDispatcher<Req, Resp>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel(buffer_size);
    let join_handle = tokio::spawn(create_jobs_task(rx, p, shutdown_listener));
    (JobDispatcher { tx }, join_handle)
}

async fn create_jobs_task<
    Req: Communicable,
    Resp: Communicable,
    P: RequestProcessor<Req, Resp> + TriviallyThreadable,
>(
    mut rx: mpsc::Receiver<Command<Req, Resp>>,
    p: P,
    mut shutdown_listener: ShutdownListener,
) {
    let p = Arc::new(p);
    let pool = ThreadPool::new(8);
    while !shutdown_listener.is_shutting_down() {
        tokio::select! {
            Some(command) = rx.recv() => {
                let p = p.clone();
                pool.execute(move || {
                    command::process(command, &p);
                });
            },
            _ = shutdown_listener.listen_for_shutdown_signal() => {},
        }
    }
    task::spawn_blocking(move || {
        pool.join();
    });
}
