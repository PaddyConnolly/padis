use padis::{Db, run_server};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let db = Db::new();

    println!("Listening on port 6379");
    run_server(listener, db).await;
}
