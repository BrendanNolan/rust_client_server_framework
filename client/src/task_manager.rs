use std::fmt::Debug;

use bincode::{self};
use tokio::{
    io::{AsyncWriteExt, AsyncReadExt},
    net::{ToSocketAddrs,TcpStream},
    sync::mpsc::Receiver,
};
use serde::{Serialize, de::{DeserializeOwned}};
use crate::command::Command;

pub async fn create_task_manager<A, S, R>(
    socket_address: A, mut rx: Receiver<Command<S, R>>)
where
    A: ToSocketAddrs,
    S: Serialize + Debug,
    R: DeserializeOwned + Debug,
{
    let mut stream = TcpStream::connect(socket_address).await.unwrap();
    while let Some(command) = rx.recv().await {
        let raw_bytes_to_send = bincode::serialize(&command.to_send).unwrap();
        stream.write(&raw_bytes_to_send).await.unwrap();
        let mut raw_bytes_received = Vec::new();
        stream.read_to_end(&mut raw_bytes_received).await.unwrap();
        let received: R = bincode::deserialize(&raw_bytes_received).unwrap();
        command.responder.send(Some(received)).unwrap();
    }
}
