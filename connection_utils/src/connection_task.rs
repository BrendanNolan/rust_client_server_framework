use tokio::{io, net::TcpStream, sync::mpsc};

use crate::{
    incremental_read::IncrementalReadStorage, stream_serialization, Communicable, DataPoster,
};

pub async fn create_connection_task<R, S, P>(
    socket: TcpStream,
    poster: P,
    mut rx: mpsc::Receiver<S>,
) where
    P: DataPoster<R>,
    R: Communicable,
    S: Communicable,
{
    let (mut reader, mut writer) = io::split(socket);
    let mut stream_read_storage = IncrementalReadStorage::default();
    loop {
        tokio::select! {
            fill_result = stream_read_storage.progress_filling(&mut reader) => {
                if fill_result.is_err() {
                    println!("Shutting down connection - socket closed.");
                    break;
                }
                if stream_read_storage.is_full() {
                    let data = bincode::deserialize::<R>(&stream_read_storage.buffer).unwrap();
                    poster.post(data).await;
                    stream_read_storage.reset();
                }
            },
            Some(response) = rx.recv() => {
                stream_serialization::send(&response, &mut writer).await;
            },
            else => break,
        }
    }
}
