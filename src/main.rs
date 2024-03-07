#[tokio::main]
async fn main() {
    let tcp_listen = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    let handler = || async { "Hello World 👋" };
    let app = axum::Router::new().route("/", axum::routing::get(handler));
    axum::serve(tcp_listen, app).await.unwrap()
}
