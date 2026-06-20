use std::sync::Arc;

use crate::model::auth::{User,Login};
use axum::{extract::{Json,State}, response::Json as ResponseJson};
use serde_json::{json,Value};
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
    create_users(&user,&mut conn);

    ResponseJson(json!({
        "result":true
    }))
}

pub async fn login(State(state): State<Arc<Appstate>>,Json(login): Json<Login>)->ResponseJson<Value>{
    println!("{}" , login.username);
    println!("{}" , login.password);

    let mut conn = state.pool.get().expect("unable to get the db pool");

    let user = user_finder(&login.username,&mut conn);
    let email = email_finder(&login.username,&mut conn);

    if user.is_some(){
        return user_verifier(&login, &user.unwrap());
    }
    if email.is_some(){
        return user_verifier(&login, &email.unwrap());
    }

    ResponseJson(json!({
        "result":false
    }))
    
}