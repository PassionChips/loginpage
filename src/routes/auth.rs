use crate::model::auth::{User,Login};
use axum::response::Json;
use serde_json::{json,Value};
use crate::dbutils::write_users::create_users;
use crate::dbutils::dbconfig::user_verifier;

pub async fn signup(Json(user): Json<User>)->Json<Value>{
    println!("{}" , user.username);
    println!("{}" , user.password);

    if user.username.len()!=0 && user.password.len()>=7{
        create_users(&user);
        Json(json!({"result":true})) 
    }
    else{
        Json(json!({"result":false}))
    }
} 

pub async fn login(Json(login): Json<Login>)->Json<Value>{
    println!("{}" , login.username);
    println!("{}" , login.password);

    if login.username.len()==0 && login.password.len()==0{
        Json(json!({"result":false}))
    } else {
        user_verifier(&login)
    }
}