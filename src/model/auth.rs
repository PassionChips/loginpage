use serde::Deserialize;

#[derive(Deserialize,Debug)]
pub struct User{
    pub username:String,
    pub password:String,
}