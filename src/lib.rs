pub mod routes;
pub mod model;
pub mod schema;
pub mod dbutils;

use tokio;
use axum::{Router, response::Html, routing::{get,post}};
use crate::routes::auth::{signup,login};
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::ServeDir;

pub async fn run(){

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(index))
        .route("/login", get(login_page))
        .route("/signup", get(signup_page))
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/success", get(success))
        .route("/error", get(error))
        .nest_service("/static", ServeDir::new("src/static")) //when the html requests for /static/auth.js you don't need to create a separate route instead you use nest_service("/static" , ServeDir::new("src/static")) it will take whatever it needs
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

async fn login_page()->Html<&'static str>{
    let html_content = include_str!("static/templates/login.html");
    Html(html_content)
}

async fn signup_page()->Html<&'static str>{
    let html_content = include_str!("static/templates/signup.html");
    Html(html_content)
}