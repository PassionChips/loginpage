use diesel::prelude::*;
use crate::schema::users;

#[derive(Debug,Queryable,Selectable)]
#[diesel(table_name= users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Users{
    pub id:i32,
    pub username:String,
    pub password:String,

}

#[derive(Debug,Insertable)]
#[diesel(table_name= users)]
pub struct NewUsers{
    pub username:String,
    pub password:String,

}