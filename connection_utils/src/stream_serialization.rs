use super::Communicable;

use serde::de::DeserializeOwned;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
};

type ByteArray = Vec<u8>;

struct ByteMessage {
    size: u64,
    data: ByteArray,
}

pub enum ReadError {
    IoError(tokio::io::Error),
    DeserializationError(bincode::Error),
}

impl From<bincode::Error> for ReadError {
    fn from(error: bincode::Error) -> Self {
        ReadError::DeserializationError(error)
    }
}

impl From<tokio::io::Error> for ReadError {
    fn from(error: tokio::io::Error) -> Self {
        ReadError::IoError(error)
    }
}

fn convert_to_byte_message(item: &impl Communicable) -> ByteMessage {
    let data = bincode::serialize(&item).unwrap();
    let size = data.len() as u64;
    ByteMessage { size, data }
}

pub async fn send(data: &impl Communicable, writer: &mut WriteHalf<TcpStream>) {
    let byte_message = convert_to_byte_message(data);
    writer.write_u64(byte_message.size).await.unwrap();
    writer.write_all(&byte_message.data).await.unwrap();
}

pub async fn receive<R: DeserializeOwned>(
    reader: &mut ReadHalf<TcpStream>,
) -> Result<R, ReadError> {
    let num_bytes_to_read = reader.read_u64().await.unwrap();
    let mut raw_bytes_received = ByteArray::new();
    raw_bytes_received.resize(num_bytes_to_read as usize, 0_u8);
    reader.read_exact(&mut raw_bytes_received).await?;
    let deserialized_r = bincode::deserialize::<R>(&raw_bytes_received)?;
    Ok(deserialized_r)
}
