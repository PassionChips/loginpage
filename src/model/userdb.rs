use diesel::prelude::*;
use crate::schema::{users,customer_profiles,client_profiles};
use chrono::NaiveDateTime;
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
    pub role:String,
    pub profile_completed:bool,
    pub profile_completed_at:Option<NaiveDateTime>,

}
#[derive(Debug,Queryable,Selectable)]
#[diesel(table_name= customer_profiles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CustomerProfiles{
    pub id:i32,
    pub user_id:i32,
    pub phone:String,
    pub alternate_phone:Option<String>,
    pub address_line1:String,
    pub address_line2:Option<String>,
    pub city:String,
    pub state:String,
    pub pincode:String,
    pub country:String,
    pub terms_accepted:bool,
    pub terms_accepted_at:Option<NaiveDateTime>,
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
}

#[derive(Debug,Queryable,Selectable)]
#[diesel(table_name= client_profiles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ClientProfiles{
    pub id:i32,
    pub user_id:i32,
    pub company_name:String,
    pub business_type:String,
    pub gst_number:String,
    pub pan_number:String,
    pub business_phone:String,
    pub business_email:String,
    pub address_line1:String,
    pub address_line2:Option<String>,
    pub city:String,
    pub state:String,
    pub pincode:String,
    pub country:String,
    pub is_tamil_nadu:bool,
    pub tamil_nadu_compliance:Option<bool>,
    pub terms_accepted:bool,
    pub terms_accepted_at:Option<NaiveDateTime>,
    pub verification_status: String,
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
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
    pub role:String,
}

#[derive(Debug,Insertable)]
#[diesel(table_name= customer_profiles)]
pub struct NewCustomerProfiles{
    pub user_id:i32,
    pub phone:String,
    pub alternate_phone:Option<String>,
    pub address_line1:String,
    pub address_line2:Option<String>,
    pub city:String,
    pub state:String,
    pub pincode:String,
    pub country:String,
    pub terms_accepted:bool,
    pub terms_accepted_at:Option<NaiveDateTime>,
}

#[derive(Debug,Insertable)]
#[diesel(table_name= client_profiles)]
pub struct NewClientProfiles{
    pub user_id:i32,
    pub company_name:String,
    pub business_type:String,
    pub gst_number:String,
    pub pan_number:String,
    pub business_phone:String,
    pub business_email:String,
    pub address_line1:String,
    pub address_line2:Option<String>,
    pub city:String,
    pub state:String,
    pub pincode:String,
    pub country:String,
    pub is_tamil_nadu:bool,
    pub tamil_nadu_compliance:Option<bool>,
    pub terms_accepted:bool,
    pub terms_accepted_at:Option<NaiveDateTime>,
    pub verification_status: String,
}