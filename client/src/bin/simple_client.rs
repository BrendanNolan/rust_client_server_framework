use client::{command::Command, tasks};
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
};

type IntStringCommand = Command<u32, String>;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel::<IntStringCommand>(32);
    let stream = TcpStream::connect("127.0.0.1:6379").await.unwrap();
    let manager = tokio::spawn(tasks::create_cyclic_connection_manager(stream, rx));
    let client = tokio::spawn(create_client_task(tx));
    client.await.unwrap();
    manager.await.unwrap();
}

async fn create_client_task(tx: mpsc::Sender<IntStringCommand>) {
    for i in 0..10 {
        let (response_tx, response_rx) = oneshot::channel();
        tx.send(IntStringCommand {
            data: i,
            responder: response_tx,
        })
        .await
        .unwrap();
        let response = response_rx.await;
        match response {
            Ok(Some(response)) => println!("Received a response: {:?}", response),
            Ok(None) => println!("Failed to read response."),
            Err(_) => panic!("Client unexpectedly failed to receive a response"),
        }
    }
}
