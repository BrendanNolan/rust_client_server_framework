use super::command::{self, Command};
use connection_utils::{Communicable, TriviallyThreadable};
use threadpool::ThreadPool;
use tokio::{
    sync::mpsc,
    sync::oneshot,
    task::{self, JoinHandle},
};

pub struct JobDispatcher<S, R>
where
    S: Communicable,
    R: Communicable,
{
    tx: mpsc::Sender<Command<S, R>>,
}

impl<S, R> JobDispatcher<S, R>
where
    S: Communicable,
    R: Communicable,
{
    pub async fn dispatch_job(&self, data: S) -> oneshot::Receiver<R> {
        let (responder, response_receiver) = oneshot::channel::<R>();
        let command = Command { data, responder };
        self.tx.send(command).await.unwrap();
        response_receiver
    }
}

// #[derive(Clone)] does not work - it requires both S and R to implement Clone
impl<S, R> Clone for JobDispatcher<S, R>
where
    S: Communicable,
    R: Communicable,
{
    fn clone(&self) -> Self {
        JobDispatcher {
            tx: self.tx.clone(),
        }
    }
}

pub fn spawn_jobs_task<S, R, F>(f: F, buffer_size: usize) -> (JobDispatcher<S, R>, JoinHandle<()>)
where
    S: Communicable,
    R: Communicable,
    F: FnOnce(&S) -> R + TriviallyThreadable + Copy,
{
    let (tx, rx) = mpsc::channel(buffer_size);
    let join_handle = tokio::spawn(create_jobs_task(rx, f));
    (JobDispatcher { tx }, join_handle)
}

async fn create_jobs_task<S, R, F>(mut rx: mpsc::Receiver<Command<S, R>>, f: F)
where
    S: Communicable,
    R: Communicable,
    F: FnOnce(&S) -> R + TriviallyThreadable + Copy,
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
