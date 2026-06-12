use diesel::prelude::*;
use crate::dbutils::dbconfig::establish_connection;
use crate::schema::users;
use crate::model::auth::User;
use crate::model::userdb::{NewUsers,Users};

//here we write the users into db
pub fn create_users(data: &User)->Users{
    let mut conn = establish_connection();
    let new_user = NewUsers{
        first_name: data.first_name.clone(),
        last_name:data.last_name.clone(),
        username : data.username.clone(),
        email: data.email.clone(),
        password : data.password.clone(),
    };

    diesel::insert_into(users::table)
        .values(new_user)
        .returning(Users::as_returning())
        .get_result(&mut conn)
        .expect("unable to insert users")
    
}
