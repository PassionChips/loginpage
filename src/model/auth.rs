use regex::Regex;
use serde::Deserialize;
use validator::{Validate, ValidationError};

lazy_static::lazy_static!{
    static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
}


// this struct is used to deserialize the data recieved from frontend and get a hold out of it
#[derive(Deserialize,Debug,Validate)]
pub struct User{
    #[validate(length(min = 2, max = 50, message = "First name must be 2-50 characters"))]
    pub first_name:String,
    #[validate(length(min = 2, max = 50, message = "Last name must be 2-50 characters"))]
    pub last_name:String,
    #[validate(length(min = 3, max = 30, message = "Username must be 3-30 characters"),
        regex(path = "*USERNAME_REGEX", message = "Username can only contain letters, numbers, and underscores"))]
    pub username:String,
    #[validate(email(message = "Invalid email format"))]
    pub email:String,
    #[validate(custom(function ="validate_password"))]
    pub password:String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Login {
    #[validate(custom(function = "validate_username_or_email"))]
    pub username: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

fn validate_password(password:&str)->Result<(),validator::ValidationError>{
    if password.len()<=6{
        return Err(ValidationError::new("Password must be more than 6 characters"));
    }
    if !password.chars().any(|c| {c.is_uppercase()}){
        return Err(ValidationError::new("Password must contain at least one uppercase letter"))
    }
    if !password.chars().any(|c| {c.is_lowercase()}){
        return Err(ValidationError::new("Password must contain at least one lowercase letter"))
    }
    if !password.chars().any(|c| {c.is_numeric()}){
        return Err(ValidationError::new("Password must contain at least one neumeric"))
    }
    Ok(())
}   

fn validate_username_or_email(username_or_email:&str)->Result<(),validator::ValidationError>{
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if email_regex.is_match(username_or_email){
        return Ok(());
    }

    if username_or_email.len()>=3 && username_or_email.len()<=10{
        return Ok(());
    }

    Err(ValidationError::new("doesn't match both username and email"))
}