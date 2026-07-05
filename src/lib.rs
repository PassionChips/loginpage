pub mod routes;
pub mod model;
pub mod schema;
pub mod dbutils;
pub mod middleware;

use crate::routes::{auth::auth_router,pages::{index,success,error},profile::profile_router};
use tokio;
use axum::{Router, routing::get};
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::ServeDir;
use crate::dbutils::dbconfig::{create_pool,Appstate};
use std::net::SocketAddr;
use tower_governor::GovernorLayer;
use crate::middleware::rate_limit::Ratelimiters;

pub async fn run(){

    let limiters = Ratelimiters::new();
    limiters.cleanup();
    let auth_rate_limiter = limiters.auth;
    let profile_rate_limiter=limiters.profile;
    

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
        .route("/success", get(success))
        .route("/error", get(error))
        .nest("/auth", auth_router())
        .layer(GovernorLayer::new(auth_rate_limiter))
        .nest("/profile", profile_router())
        .layer(GovernorLayer::new(profile_rate_limiter))
        .nest_service("/static", ServeDir::new("src/static")) //when the html requests for /static/auth.js you don't need to create a separate route instead you use nest_service("/static" , ServeDir::new("src/static")) it will take whatever it needs
        .layer(cors)
        .with_state(appstate);
    let addr = SocketAddr::from(([127,0,0,1],3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("listening on {}",listener.local_addr().unwrap());

    axum::serve(listener,app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();

}




