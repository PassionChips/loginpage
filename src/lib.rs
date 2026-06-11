pub mod routes;
pub mod model;
pub mod schema;
pub mod dbutils;

use tokio;
use axum::{Router, response::Html, routing::{get,post}}; 
use crate::routes::auth::login;
use tower_http::cors::{CorsLayer, Any};

pub async fn run(){

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(index))
        .route("/login", post(login))
        .route("/success", get(success))
        .route("/error", get(error))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("listening on {}",listener.local_addr().unwrap());

    axum::serve(listener,app).await.unwrap();

}

async fn index()->Html<&'static str>{
    let html_content = include_str!("static/templates/index.html");
    Html(html_content)
}
async fn success()->Html<&'static str>{
    let html_content = include_str!("static/templates/success.html");
    Html(html_content)
}

async fn error()-> Html<&'static str>{
    let html_content = include_str!("static/templates/error.html");
    Html(html_content)
}