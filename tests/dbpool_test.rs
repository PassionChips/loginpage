
use std::env;
use diesel::{PgConnection, r2d2::{ConnectionManager,Pool,PooledConnection}};
use dotenvy::dotenv;
use diesel::prelude::*;
use diesel::sql_query;

pub type Dbpool = Pool<ConnectionManager<PgConnection>>;
pub type Dbconnection = PooledConnection<ConnectionManager<PgConnection>>;

pub fn create_pool_test()->Dbpool{
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("unable to connect to database");
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::builder()
        .max_size(1)
        .build(manager)
        .expect("unable to build the pool")
}

table! {
    test_pool_user{
        id -> Integer,
        name -> Text,
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_1(){
        let _pool = create_pool_test();
        let mut conn = _pool.get().expect("unable to recieve PgConnection");

        sql_query(
            "CREATE TABLE IF NOT EXISTS test_pool_user (id SERIAL PRIMARY KEY, name TEXT NOT NULL)",
        )
        .execute(&mut conn)
        .unwrap();

        diesel::insert_into(test_pool_user::table)
        .values(test_pool_user::name.eq("John"))
        .execute(&mut conn)
        .unwrap();
        
    }
}


