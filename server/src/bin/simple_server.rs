use server::{request_processing::RequestProcessor, server_runner, shutdown::ShutdownListener};
use tokio::{signal, sync::watch};

fn convert_int_to_string(i: &u32) -> String {
    std::thread::sleep(std::time::Duration::from_secs(1));
    format!("The number is: {}", i)
}

struct IntToStringConverter {}

impl IntToStringConverter {
    fn new() -> Self {
        IntToStringConverter {}
    }
}

impl RequestProcessor<u32, String> for IntToStringConverter {
    fn process(&self, request: &u32) -> String {
        convert_int_to_string(request)
    }
}

#[tokio::main]
async fn main() {
    let (tx_shutdown, rx_shutdown) = watch::channel(());
    let server_task = tokio::spawn(server_runner::run_server(
        "127.0.0.1:6379",
        IntToStringConverter::new(),
        ShutdownListener::new(rx_shutdown),
        10,
        10,
    ));
    handle_shutdown(tx_shutdown).await;
    let _ = server_task.await;
}

async fn handle_shutdown(tx_shutdown: watch::Sender<()>) {
    let _ = signal::ctrl_c().await;
    println!("Server shutting down ...");
    let _ = tx_shutdown.send(());
}
