use tokio::{
    net::ToSocketAddrs,
    sync::{
        mpsc::{self, Receiver},
        oneshot,
    },
};
use serde::{Serialize, Deserialize};

pub async fn create_task_manager<Address, Command>(
    socket_address: Address, mut rx: Receiver<Command>)
where
    Address: ToSocketAddrs,
    Command: Serialize + Deserialize<'static>,
{
}
