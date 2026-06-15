use argon2::{Argon2, PasswordHash, PasswordVerifier};
use dotenvy::dotenv;
use std::env;
use diesel::prelude::*;
use axum::Json;
use serde_json::{json,Value};
use crate::model::{auth::{Login}, userdb::Users};
use crate::schema::users;
use diesel::r2d2::{ConnectionManager,PooledConnection,Pool};


pub type Dbpool = Pool<ConnectionManager<PgConnection>>;
pub type Dbconnection = PooledConnection<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct Appstate{
    pub pool:Dbpool
}

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

pub fn user_finder(username:&str,conn:&mut PgConnection)->Option<Users>{
    users::table
        .filter(users::username.eq(username))
        .first::<Users>(conn)
        .ok()


}

pub fn email_finder(email:&str,conn:&mut PgConnection)->Option<Users>{
    users::table
        .filter(users::email.eq(email))
        .first::<Users>(conn)
        .ok()

}

pub fn create_pool()->Dbpool{
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("unable to get database_url");
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::builder()
        .max_size(10)
        .min_idle(Some(2))
        .connection_timeout(std::time::Duration::from_secs(5))
        .build(manager)
        .expect("unable to build Dbpool")
}

pub fn get_pool(pool:&Dbpool)->Result<Dbconnection,r2d2::Error>{
    pool.get()
}

