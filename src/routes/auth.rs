use crate::model::auth::{User,Login};
use axum::response::Json;
use serde_json::{json,Value};
use crate::dbutils::write_users::create_users;
use crate::dbutils::dbconfig::{user_verifier,email_finder,user_finder};

pub async fn signup(Json(user): Json<User>)->Json<Value>{
    println!("{}" , user.username);
    println!("{}" , user.password);

    if email_finder(&user.email).is_some(){
        return Json(json!({
            "result":false
        }));
    }

    if user_finder(&user.username).is_some(){
        return Json(json!({
            "result":false
        }));
    }
    create_users(&user);

    Json(json!({
        "result":true
    }))
} 

pub async fn login(Json(login): Json<Login>)->Json<Value>{
    println!("{}" , login.username);
    println!("{}" , login.password);

    let user = user_finder(&login.username);
    let email = email_finder(&login.username);

    if user.is_some(){
        return user_verifier(&login, &user.unwrap());
    }
    if email.is_some(){
        return user_verifier(&login, &email.unwrap());
    }

    Json(json!({
        "result":false
    }))
    
}