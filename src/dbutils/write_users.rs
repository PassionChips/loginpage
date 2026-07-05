use diesel::prelude::*;
use crate::schema::users;
use crate::model::auth::User;
use crate::model::userdb::{NewUsers,Users};
use argon2::{
    password_hash::{
        PasswordHasher, SaltString
    },
    Argon2
};
use rand::rngs::OsRng;

//here we write the users into db
pub fn create_users(data: &User, conn:&mut PgConnection)->Users{

    let password = data.password.as_bytes();
    let hashed_password = password_hasher(password).unwrap();

    let new_user = NewUsers{
        first_name: data.first_name.clone(),
        last_name:data.last_name.clone(),
        username : data.username.clone(),
        email: data.email.clone(),
        password : hashed_password,
        role: data.role.clone(),
    };

    diesel::insert_into(users::table)
        .values(new_user)
        .returning(Users::as_returning())
        .get_result(conn)
        .expect("unable to insert users")
    
}

fn password_hasher(pw:&[u8])->Result<String,argon2::password_hash::Error>{

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash= argon2.hash_password(pw, &salt)?.to_string();
    Ok(password_hash)
}
