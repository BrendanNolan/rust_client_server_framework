use std::fmt::Debug;

use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
};

pub async fn send(message: &(impl Serialize + Debug), writer: &mut WriteHalf<TcpStream>) {
    let raw_bytes_to_send = bincode::serialize(message).unwrap();
    writer.write_all(&raw_bytes_to_send).await.unwrap();
}

pub async fn receive<R>(reader: &mut ReadHalf<TcpStream>) -> bincode::Result<R>
where
    R: DeserializeOwned,
{
    let mut raw_bytes_received = Vec::new();
    reader.read_to_end(&mut raw_bytes_received).await.unwrap();
    bincode::deserialize::<R>(&raw_bytes_received)
}
