use argon2::{Argon2, PasswordHash, PasswordVerifier};
use dotenvy::dotenv;
use std::env;
use diesel::prelude::*;
use axum::response::Json;
use serde_json::{json,Value};
use crate::model::{auth::{Login}, userdb::Users};
use crate::schema::users;
//here we establish connection for database 
pub fn establish_connection()->PgConnection{
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("unable to fetch the url");
    PgConnection::establish(&database_url).unwrap_or_else(|_| panic!("Error connecting to database: {}" , database_url))
}

pub fn user_verifier(data:&Login , user:&Users)->Json<Value>{
    let pass = data.password.as_bytes();
    let parsed_hash = PasswordHash::new(&user.password).unwrap();
    if Argon2::default().verify_password(pass,&parsed_hash).is_ok(){
        Json(json!({
            "result":true
        }))
    } else {
        Json(json!({
            "result":false
        }))
    }

}

pub fn user_finder(username:&str)->Option<Users>{
    let mut conn = establish_connection();
    users::table
        .filter(users::username.eq(username))
        .first::<Users>(&mut conn)
        .ok()


}

pub fn email_finder(email:&str)->Option<Users>{
    let mut conn = establish_connection();
    users::table
        .filter(users::email.eq(email))
        .first::<Users>(&mut conn)
        .ok()

}