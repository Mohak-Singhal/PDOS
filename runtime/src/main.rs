use runtime::start_runtime;

#[tokio::main]
async fn main() {
    start_runtime().await;
}