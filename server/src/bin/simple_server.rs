use server::server_runner;

fn put_int_in_string(i: &u32) -> Option<String> {
    Some(format!("The number is: {}", i))
}

#[tokio::main]
async fn main() {
    server_runner::run_server("127.0.0.1:6379", put_int_in_string).await;
}
