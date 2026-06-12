use serde::Deserialize;

// this struct is used to deserialize the data recieved from frontend and get a hold out of it
#[derive(Deserialize,Debug)]
pub struct User{
    pub first_name:String,
    pub last_name:String,
    pub username:String,
    pub email:String,
    pub password:String,
}

#[derive(Debug,Deserialize)]
pub struct Login{
    pub username:String,
    pub password:String,
}