use client::tasks;
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
};

use connection_utils::command::Command;

type StringCommand = Command<u32, String>;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel::<StringCommand>(32);
    let stream = TcpStream::connect("127.0.0.1:6379").await.unwrap();
    let manager = tokio::spawn(tasks::create_connection_manager(stream, rx));
    let client = tokio::spawn(create_client_task(tx));
    client.await.unwrap();
    manager.await.unwrap();
}

async fn create_client_task(tx: mpsc::Sender<StringCommand>) {
    let (response_tx, response_rx) = oneshot::channel();
    tx.send(StringCommand {
        data: 147,
        responder: response_tx,
    })
    .await
    .unwrap();
    let response = response_rx.await;
    println!("Got a response {:?}", response);
}
