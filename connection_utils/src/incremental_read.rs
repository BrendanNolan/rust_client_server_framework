use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, ReadHalf},
    net::TcpStream,
};

#[derive(Default)]
pub struct IncrementalReadStorage {
    size: Option<usize>,
    pub buffer: BytesMut,
}

impl IncrementalReadStorage {
    pub async fn progress_filling(&mut self, reader: &mut ReadHalf<TcpStream>) {
        if self.is_full() {
            panic!("IncrementalReadStorage Already Full");
        }
        if !self.is_initialized() {
            let num_bytes_to_read = reader.read_u64().await.unwrap();
            self.set_up(num_bytes_to_read as usize);
        }
        let _ = reader.read_buf(&mut self.buffer).await;
    }

    pub fn is_full(&self) -> bool {
        self.size.is_some() && (self.buffer.len() == self.buffer.capacity())
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    fn set_up(&mut self, size: usize) {
        self.size = Some(size);
        self.buffer = BytesMut::with_capacity(size);
    }

    fn is_initialized(&self) -> bool {
        self.size.is_some()
    }
}
