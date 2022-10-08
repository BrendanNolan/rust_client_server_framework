use server::{request_processing::RequestProcessor, server_runner};

fn put_int_in_string(i: &u32) -> String {
    std::thread::sleep(std::time::Duration::from_secs(1));
    format!("The number is: {}", i)
}

struct StringIntPlacer {}

impl StringIntPlacer {
    fn new() -> Self {
        StringIntPlacer {}
    }
}

impl RequestProcessor<u32, String> for StringIntPlacer {
    fn process(&self, request: &u32) -> String {
        put_int_in_string(request)
    }
}

#[tokio::main]
async fn main() {
    server_runner::run_server("127.0.0.1:6379", StringIntPlacer::new(), 10, 10).await;
}
