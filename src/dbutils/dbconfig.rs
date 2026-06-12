use dotenvy::dotenv;
use std::env;
use diesel::prelude::*;

//here we establish connection for database 
pub fn establish_connection()->PgConnection{
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("unable to fetch the url");
    PgConnection::establish(&database_url).unwrap_or_else(|_| panic!("Error connecting to database: {}" , database_url))
}