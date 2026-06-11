use diesel::prelude::*;
use auth::*;
use auth::dbutils::dbconfig::establish_connection;
use auth::model::userdb::Users;



pub fn main(){
    use self::schema::users::dsl::*;

    let mut conn = establish_connection();

    let results = users
        .limit(5)
        .select(Users::as_select())
        .load(&mut conn)
        .expect("cannot load the data properly");

    println!("Displaying {} Posts: ",results.len());

    for res in results{
        println!("user id: {}", res.id);
        println!("-----------\n");
        println!("username : {}", res.username);
    }
    
}