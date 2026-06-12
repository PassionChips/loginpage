use diesel::prelude::*;
use crate::schema::users;
//this struct is used for querying purpose in future if we want to cross check a particular user
#[derive(Debug,Queryable,Selectable)]
#[diesel(table_name= users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Users{
    pub id:i32,
    pub first_name:String,
    pub last_name:String,
    pub username:String,
    pub email:String,
    pub password:String,

}

//this struct is used to create a new user data structure from the deserialized struct and push it inside the db
#[derive(Debug,Insertable)]
#[diesel(table_name= users)]
pub struct NewUsers{
    pub first_name:String,
    pub last_name:String,
    pub username:String,
    pub email:String,
    pub password:String,

}