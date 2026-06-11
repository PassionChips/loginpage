use diesel::prelude::*;
use crate::dbutils::dbconfig::establish_connection;
use crate::schema::users;
use crate::model::auth::User;
use crate::model::userdb::{NewUsers,Users};

pub fn create_users(data: &User)->Users{
    let mut conn = establish_connection();
    let new_user = NewUsers{
        username : data.username.clone(),
        password : data.password.clone()

    };

    diesel::insert_into(users::table)
        .values(new_user)
        .returning(Users::as_returning())
        .get_result(&mut conn)
        .expect("unable to insert users")
    
}
