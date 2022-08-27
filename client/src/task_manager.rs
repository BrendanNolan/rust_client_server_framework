use std::fmt::Debug;

use crate::command::Command;
use bincode::{self};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::{TcpStream, ToSocketAddrs},
    sync::mpsc::Receiver,
};

pub async fn create_task_manager<A, S, R>(socket_address: A, mut rx: Receiver<Command<S, R>>)
where
    A: ToSocketAddrs,
    S: Serialize + Debug,
    R: DeserializeOwned + Debug,
{
    let stream = TcpStream::connect(socket_address).await.unwrap();
    let (mut reader, mut writer) = io::split(stream);
    while let Some(command) = rx.recv().await {
        send(&command.to_send, &mut writer).await;
        let received = receive::<R>(&mut reader).await;
        command.responder.send(received.ok()).unwrap();
    }
}

async fn send(message: &(impl Serialize + Debug), writer: &mut WriteHalf<TcpStream>) {
    let raw_bytes_to_send = bincode::serialize(message).unwrap();
    writer.write_all(&raw_bytes_to_send).await.unwrap();
}

async fn receive<R>(reader: &mut ReadHalf<TcpStream>) -> bincode::Result<R>
where
    R: DeserializeOwned + Debug,
{
    let mut raw_bytes_received = Vec::new();
    reader.read_to_end(&mut raw_bytes_received).await.unwrap();
    bincode::deserialize::<R>(&raw_bytes_received)
}
