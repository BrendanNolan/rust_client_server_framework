use crate::Communicable;

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

pub enum StreamingError {
    IoError(tokio::io::Error),
    DeserializationError(bincode::Error),
}

impl From<bincode::Error> for StreamingError {
    fn from(error: bincode::Error) -> Self {
        StreamingError::DeserializationError(error)
    }
}

impl From<tokio::io::Error> for StreamingError {
    fn from(error: tokio::io::Error) -> Self {
        StreamingError::IoError(error)
    }
}

fn convert_to_byte_message(item: &impl Communicable) -> ByteMessage {
    let data = bincode::serialize(&item).unwrap();
    let size = data.len() as u64;
    ByteMessage { size, data }
}

pub async fn send(
    data: &impl Communicable,
    writer: &mut WriteHalf<TcpStream>,
) -> Result<(), StreamingError> {
    let byte_message = convert_to_byte_message(data);
    writer.write_u64(byte_message.size).await?;
    writer.write_all(&byte_message.data).await?;
    Ok(())
}

pub async fn receive<R: DeserializeOwned>(
    reader: &mut ReadHalf<TcpStream>,
) -> Result<R, StreamingError> {
    let num_bytes_to_read = reader.read_u64().await?;
    let mut raw_bytes_received = ByteArray::new();
    raw_bytes_received.resize(num_bytes_to_read as usize, 0_u8);
    reader.read_exact(&mut raw_bytes_received).await?;
    let deserialized_r = bincode::deserialize::<R>(&raw_bytes_received)?;
    Ok(deserialized_r)
}
