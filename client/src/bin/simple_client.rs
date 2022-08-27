use tokio::sync::{mpsc, oneshot};

use client::command::Command;
use client::task_manager;

type StringCommand = Command<u32, String>;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel::<StringCommand>(32);
    let manager = tokio::spawn(task_manager::create_task_manager("127.0.0.1:6379", rx));
    let client = tokio::spawn(create_client_task(tx));
    client.await.unwrap();
    manager.await.unwrap();
}

async fn create_client_task(tx: mpsc::Sender<StringCommand>) {
    let (response_tx, response_rx) = oneshot::channel();
    tx.send(StringCommand {
        to_send: 147,
        responder: response_tx,
    })
    .await
    .unwrap();
    let response = response_rx.await;
    println!("Got a response {:?}", response);
}
