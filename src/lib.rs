pub mod routes;
pub mod model;
pub mod schema;
pub mod dbutils;
pub mod middleware;

use tokio;
use axum::{Router, response::Html, routing::{get,post}};
use std::sync::Arc;
use crate::routes::auth::{signup,login};
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::ServeDir;
use crate::dbutils::dbconfig::{create_pool,Appstate};
use std::net::SocketAddr;
use tower_governor::GovernorLayer;
use crate::middleware::rate_limit::Ratelimiters;

pub async fn run(){

    let limiters = Ratelimiters::new();
    limiters.cleanup();
    let login_rate_limiter = limiters.login;
    let signup_rate_limiter = limiters.signup;
    let general_rate_limiter = limiters.general;
    

    let pool = create_pool();
    let appstate = Arc::new(Appstate{
        pool:pool
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(index))
        .route("/login", get(login_page))
        .route("/login", post(login))
        .layer(GovernorLayer::new(login_rate_limiter))
        .route("/signup", get(signup_page))
        .route("/signup", post(signup))
        .layer(GovernorLayer::new(signup_rate_limiter))
        .route("/success", get(success))
        .route("/error", get(error))
        .layer(GovernorLayer::new(general_rate_limiter))
        .nest_service("/static", ServeDir::new("src/static")) //when the html requests for /static/auth.js you don't need to create a separate route instead you use nest_service("/static" , ServeDir::new("src/static")) it will take whatever it needs
        .layer(cors)
        .with_state(appstate);
    let addr = SocketAddr::from(([127,0,0,1],3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("listening on {}",listener.local_addr().unwrap());

    axum::serve(listener,app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();

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