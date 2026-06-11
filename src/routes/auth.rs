use crate::model::auth::User;
use axum::response::Json;
use serde_json::{json,Value};
use crate::dbutils::write_users::create_users;

pub async fn login(Json(user): Json<User>)->Json<Value>{
    println!("{}" , user.username);
    println!("{}" , user.password);

    if user.username.len()!=0 && user.password.len()!=0{
        create_users(&user); 
        Json(json!({"result":true})) 
    }
    else{
        Json(json!({"result":false}))
    }
} 