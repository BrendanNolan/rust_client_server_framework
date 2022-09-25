use super::command::{self, Command};
use connection_utils::{Communicable, TriviallyThreadable};
use threadpool::ThreadPool;
use tokio::{
    sync::mpsc,
    sync::oneshot,
    task::{self, JoinHandle},
};

pub struct JobDispatcher<Req, Resp>
where
    Req: Communicable,
    Resp: Communicable,
{
    tx: mpsc::Sender<Command<Req, Resp>>,
}

impl<Req, Resp> JobDispatcher<Req, Resp>
where
    Req: Communicable,
    Resp: Communicable,
{
    pub async fn dispatch_job(&self, data: Req) -> oneshot::Receiver<Resp> {
        let (responder, response_receiver) = oneshot::channel::<Resp>();
        let command = Command { data, responder };
        self.tx.send(command).await.unwrap();
        response_receiver
    }
}

// #[derive(Clone)] does not work - it requires that both generic parameters implement Clone
impl<Req, Resp> Clone for JobDispatcher<Req, Resp>
where
    Req: Communicable,
    Resp: Communicable,
{
    fn clone(&self) -> Self {
        JobDispatcher {
            tx: self.tx.clone(),
        }
    }
}

pub fn spawn_jobs_task<Req, Resp, Op>(
    f: Op,
    buffer_size: usize,
) -> (JobDispatcher<Req, Resp>, JoinHandle<()>)
where
    Req: Communicable,
    Resp: Communicable,
    Op: FnOnce(&Req) -> Resp + TriviallyThreadable + Copy,
{
    let (tx, rx) = mpsc::channel(buffer_size);
    let join_handle = tokio::spawn(create_jobs_task(rx, f));
    (JobDispatcher { tx }, join_handle)
}

async fn create_jobs_task<Req, Resp, Op>(mut rx: mpsc::Receiver<Command<Req, Resp>>, f: Op)
where
    Req: Communicable,
    Resp: Communicable,
    Op: FnOnce(&Req) -> Resp + TriviallyThreadable + Copy,
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
