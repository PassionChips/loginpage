use std::sync::Arc;

use crate::model::auth::{User,Login};
use axum::{extract::{Json,State}, response::Json as ResponseJson,response::Html};
use serde_json::{json,Value};
use axum::{Router,routing::{get,post}};
use crate::dbutils::write_users::create_users;
use crate::dbutils::dbconfig::{user_verifier,email_finder,user_finder};
use crate::dbutils::dbconfig::{Appstate};

pub async fn signup(State(state): State<Arc<Appstate>>, Json(user): Json<User>)->ResponseJson<Value>{
    println!("{}" , user.username);
    println!("{}" , user.password);

    let mut conn = state.pool.get().expect("Unable to get connection");

    if email_finder(&user.email,&mut conn).is_some(){
        return ResponseJson(json!({
            "result":false
        }));
    }

    if user_finder(&user.username,&mut conn).is_some(){
        return ResponseJson(json!({
            "result":false
        }));
    }
    let user = create_users(&user,&mut conn);

    ResponseJson(json!({
        "result":true,
        "user_id":user.id.clone(),
        "username":user.username.clone(),
        "role":user.role.clone()
    }))
}

pub async fn login(State(state): State<Arc<Appstate>>,Json(login): Json<Login>)->ResponseJson<Value>{
    println!("{}" , login.username);
    println!("{}" , login.password);

    let mut conn = state.pool.get().expect("unable to get the db pool");

    let user = user_finder(&login.username,&mut conn);
    let email = email_finder(&login.username,&mut conn);

    if user.is_some(){
        let found_user = user.unwrap();
        let verification = user_verifier(&login, &found_user);
        // If verification succeeds, add user_id and username
        if let Some(result) = verification.0.get("result") {
            if result.as_bool() == Some(true) {
                return ResponseJson(json!({
                    "result": true,
                    "user_id": found_user.id,
                    "username": found_user.username,
                    "role": found_user.role
                }));
            }
        }
        return verification;
    }
    if email.is_some(){
        let found_user = email.unwrap();
        let verification = user_verifier(&login, &found_user);
        // If verification succeeds, add user_id and username
        if let Some(result) = verification.0.get("result") {
            if result.as_bool() == Some(true) {
                return ResponseJson(json!({
                    "result": true,
                    "user_id": found_user.id,
                    "username": found_user.username,
                    "role": found_user.role
                }));
            }
        }
        return verification;
    }

    ResponseJson(json!({
        "result":false
    }))
    
}


pub fn auth_router()->Router<Arc<Appstate>>{
    Router::new()
        .route("/login", get(login_page))
        .route("/login", post(login))
        .route("/signup", get(signup_page))
        .route("/signup", post(signup))
}
async fn login_page()->Html<&'static str>{
    let html_content = include_str!("../static/templates/login.html");
    Html(html_content)
}
async fn signup_page()->Html<&'static str>{
    let html_content = include_str!("../static/templates/signup.html");
    Html(html_content)
}
