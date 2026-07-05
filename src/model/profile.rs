use chrono::NaiveDateTime;
use serde::Deserialize;
use validator::{Validate, ValidationError};
use regex::Regex;

lazy_static::lazy_static! {
    static ref PHONE_REGEX: Regex = Regex::new(r"^(\+91)?[6-9]\d{9}$").unwrap();
    static ref PINCODE_REGEX: Regex = Regex::new(r"^\d{6}$").unwrap();
    static ref GST_REGEX: Regex = Regex::new(r"^[0-9]{2}[A-Z]{5}[0-9]{4}[A-Z]{1}[1-9A-Z]{1}Z[0-9A-Z]{1}$").unwrap();
    static ref PAN_REGEX: Regex = Regex::new(r"^[A-Z]{5}[0-9]{4}[A-Z]{1}$").unwrap();
}

fn validate_phone(phone: &str) -> Result<(), ValidationError> {
    let cleaned = phone.replace(&[' ', '-'][..], "");
    if PHONE_REGEX.is_match(&cleaned) {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid phone number format. Must be a valid Indian phone number"))
    }
}

fn validate_pincode(pincode: &str) -> Result<(), ValidationError> {
    if PINCODE_REGEX.is_match(pincode) {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid pincode. Must be 6 digits"))
    }
}

// fn validate_gst(gst: &str) -> Result<(), ValidationError> {
//     if GST_REGEX.is_match(gst) {
//         Ok(())
//     } else {
//         Err(ValidationError::new("Invalid GST number format"))
//     }
// }

// fn validate_pan(pan: &str) -> Result<(), ValidationError> {
//     if PAN_REGEX.is_match(pan) {
//         Ok(())
//     } else {
//         Err(ValidationError::new("Invalid PAN number format"))
//     }
// }

#[derive(Debug, Deserialize, Validate)]
pub struct CustomerProfileExtract {
    pub user_id: i32,
    #[validate(custom(function = "validate_phone"))]
    pub phone: String,
    #[validate(custom(function = "validate_phone"))]
    pub alternate_phone: Option<String>,
    #[validate(length(min = 5, max = 255, message = "Address line 1 must be between 5 and 255 characters"))]
    pub address_line1: String,
    #[validate(length(max = 255, message = "Address line 2 must not exceed 255 characters"))]
    pub address_line2: Option<String>,
    #[validate(length(min = 2, max = 100, message = "City must be between 2 and 100 characters"))]
    pub city: String,
    #[validate(length(min = 2, max = 100, message = "State must be between 2 and 100 characters"))]
    pub state: String,
    #[validate(custom(function = "validate_pincode"))]
    pub pincode: String,
    #[validate(length(min = 2, max = 100, message = "Country must be between 2 and 100 characters"))]
    pub country: String,
    pub terms_accepted: bool,
    pub terms_accepted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ClientProfileExtract {
    pub user_id: i32,
    #[validate(length(min = 2, max = 255, message = "Company name must be between 2 and 255 characters"))]
    pub company_name: String,
    #[validate(length(min = 2, max = 50, message = "Business type must be between 2 and 50 characters"))]
    pub business_type: String,
//    #[validate(custom(function = "validate_gst"))]
    pub gst_number: String,
//    #[validate(custom(function = "validate_pan"))]
    pub pan_number: String,
    #[validate(custom(function = "validate_phone"))]
    pub business_phone: String,
    #[validate(email(message = "Invalid business email format"))]
    pub business_email: String,
    #[validate(length(min = 5, max = 255, message = "Address line 1 must be between 5 and 255 characters"))]
    pub address_line1: String,
    #[validate(length(max = 255, message = "Address line 2 must not exceed 255 characters"))]
    pub address_line2: Option<String>,
    #[validate(length(min = 2, max = 100, message = "City must be between 2 and 100 characters"))]
    pub city: String,
    #[validate(length(min = 2, max = 100, message = "State must be between 2 and 100 characters"))]
    pub state: String,
    #[validate(custom(function = "validate_pincode"))]
    pub pincode: String,
    #[validate(length(min = 2, max = 100, message = "Country must be between 2 and 100 characters"))]
    pub country: String,
    pub is_tamil_nadu: bool,
    pub tamil_nadu_compliance: Option<bool>,
    pub terms_accepted: bool,
    pub terms_accepted_at: Option<NaiveDateTime>,
    #[validate(length(min = 2, max = 20, message = "Verification status must be between 2 and 20 characters"))]
    pub verification_status: String,
}

// Made with Bob
