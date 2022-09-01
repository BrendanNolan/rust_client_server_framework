use super::command::Communicable;
use serde::de::DeserializeOwned;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
};

type ByteArray = Vec<u8>;

struct ByteMessage {
    size: ByteArray,
    data: ByteArray,
}

fn convert_to_byte_message(item: &impl Communicable) -> ByteMessage {
    let data = bincode::serialize(&item).unwrap();
    let size = bincode::serialize(&(data.len() as u64)).unwrap();
    ByteMessage { size, data }
}

pub async fn send(data: &impl Communicable, writer: &mut WriteHalf<TcpStream>) {
    let byte_message = convert_to_byte_message(data);
    writer.write_all(&byte_message.size).await.unwrap();
    writer.write_all(&byte_message.data).await.unwrap();
}

async fn receive_first_u64(reader: &mut ReadHalf<TcpStream>) -> bincode::Result<u64> {
    let mut raw_bytes_received: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    reader.read_exact(&mut raw_bytes_received).await.unwrap();
    bincode::deserialize::<u64>(&raw_bytes_received)
}

pub async fn receive<R: DeserializeOwned>(reader: &mut ReadHalf<TcpStream>) -> bincode::Result<R> {
    let num_bytes_to_read = receive_first_u64(reader).await.unwrap();
    let mut raw_bytes_received = ByteArray::new();
    raw_bytes_received.resize(num_bytes_to_read as usize, 0 as u8);
    reader.read_exact(&mut raw_bytes_received).await.unwrap();
    bincode::deserialize::<R>(&raw_bytes_received)
}
