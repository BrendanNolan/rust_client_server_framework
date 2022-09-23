use super::command::{self, Command};
use connection_utils::{Communicable, TriviallyThreadable};
use threadpool::ThreadPool;
use tokio::{
    sync::mpsc,
    sync::oneshot,
    task::{self, JoinHandle},
};

pub struct JobDispatcher<Request, Response>
where
    Request: Communicable,
    Response: Communicable,
{
    tx: mpsc::Sender<Command<Request, Response>>,
}

impl<Request, Response> JobDispatcher<Request, Response>
where
    Request: Communicable,
    Response: Communicable,
{
    pub async fn dispatch_job(&self, data: Request) -> oneshot::Receiver<Response> {
        let (responder, response_receiver) = oneshot::channel::<Response>();
        let command = Command { data, responder };
        self.tx.send(command).await.unwrap();
        response_receiver
    }
}

// #[derive(Clone)] does not work - it requires that both generic parameters implement Clone
impl<Request, Response> Clone for JobDispatcher<Request, Response>
where
    Request: Communicable,
    Response: Communicable,
{
    fn clone(&self) -> Self {
        JobDispatcher {
            tx: self.tx.clone(),
        }
    }
}

pub fn spawn_jobs_task<Request, Response, Operation>(
    f: Operation,
    buffer_size: usize,
) -> (JobDispatcher<Request, Response>, JoinHandle<()>)
where
    Request: Communicable,
    Response: Communicable,
    Operation: FnOnce(&Request) -> Response + TriviallyThreadable + Copy,
{
    let (tx, rx) = mpsc::channel(buffer_size);
    let join_handle = tokio::spawn(create_jobs_task(rx, f));
    (JobDispatcher { tx }, join_handle)
}

async fn create_jobs_task<Request, Response, Operation>(
    mut rx: mpsc::Receiver<Command<Request, Response>>,
    f: Operation,
) where
    Request: Communicable,
    Response: Communicable,
    Operation: FnOnce(&Request) -> Response + TriviallyThreadable + Copy,
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
