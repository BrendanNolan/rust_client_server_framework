use client::{command::Command, tasks};
use connection_utils::ServerError;
use std::time::SystemTime;
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
    let begin_time = SystemTime::now();
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
            Ok(Some(response)) => match process_response_from_server(&response, &begin_time) {
                ClientAction::Continue => {}
                ClientAction::Stop => return,
            },
            Ok(None) => println!("Failed to read response."),
            Err(_) => panic!("Client unexpectedly failed to receive a response"),
        }
    }
}

enum ClientAction {
    Continue,
    Stop,
}

fn process_response_from_server(
    response: &Result<String, ServerError>,
    begin_time: &SystemTime,
) -> ClientAction {
    match response {
        Ok(response) => {
            println!(
                "Received a response: {:?} after {} seconds.",
                response,
                begin_time.elapsed().unwrap().as_secs()
            );
            ClientAction::Continue
        }
        Err(error_message) => {
            println!(
                "Received a server error: {:?} after {} seconds.",
                error_message,
                begin_time.elapsed().unwrap().as_secs()
            );
            ClientAction::Stop
        }
    }
}
