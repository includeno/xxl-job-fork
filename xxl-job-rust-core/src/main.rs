mod models;
mod db;
mod web;

use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    println!("Starting XXL-JOB in Rust!");

    let db_pool = match db::init_db_pool().await {
        Ok(pool) => {
            println!("Database pool initialized successfully.");
            pool
        }
        Err(e) => {
            eprintln!("Failed to initialize database pool: {}", e);
            return;
        }
    };

    let app = web::create_router(db_pool);

    let listener = match TcpListener::bind("0.0.0.0:3000").await {
        Ok(listener) => {
            println!("Server listening on 0.0.0.0:3000");
            listener
        }
        Err(e) => {
            eprintln!("Failed to bind to port 3000: {}", e);
            return;
        }
    };

    axum::serve(listener, app).await.unwrap();
}