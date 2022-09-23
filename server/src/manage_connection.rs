use crate::jobs::JobDispatcher;
use connection_utils::{
    incremental_read::IncrementalReadStorage, stream_serialization, Communicable,
};
use futures::stream::{FuturesUnordered, StreamExt};
use tokio::{io, net::TcpStream};

pub async fn create_connection_manager<ReceiveType, SendType>(
    stream: TcpStream,
    job_dispatcher: JobDispatcher<ReceiveType, SendType>,
) where
    ReceiveType: Communicable,
    SendType: Communicable,
{
    let (mut reader, mut writer) = io::split(stream);
    let mut response_receivers = FuturesUnordered::new();
    let mut stream_read_storage = IncrementalReadStorage::default();
    loop {
        tokio::select! {
            fill_result = stream_read_storage.progress_filling(&mut reader) => {
                if fill_result.is_err() {
                    println!("Shutting down server connection - client disconnected");
                    break;
                }
                if stream_read_storage.is_full() {
                    let data = bincode::deserialize::<ReceiveType>(&stream_read_storage.buffer).unwrap();
                    let response_receiver = job_dispatcher.dispatch_job(data).await;
                    response_receivers.push(response_receiver);
                    stream_read_storage.reset();
                }
            },
            Some(Ok(response)) = response_receivers.next() => {
                stream_serialization::send(&response, &mut writer).await;
            },
            else => break,
        }
    }
}
