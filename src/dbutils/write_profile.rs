use diesel::prelude::*;
use crate::model::userdb::{NewClientProfiles,NewCustomerProfiles,CustomerProfiles,ClientProfiles};
use crate::model::profile::{CustomerProfileExtract,ClientProfileExtract};
use crate::schema::{customer_profiles,client_profiles};

pub fn create_customer_profile(data:&CustomerProfileExtract,conn:&mut PgConnection)->CustomerProfiles{

    let customerprofile = NewCustomerProfiles{
        user_id:data.user_id.clone(),
        phone:data.phone.clone(),
        alternate_phone:data.alternate_phone.clone(),
        address_line1:data.address_line1.clone(),
        address_line2:data.address_line2.clone(),
        city:data.city.clone(),
        state:data.state.clone(),
        pincode:data.pincode.clone(),
        country:data.country.clone(),
        terms_accepted:data.terms_accepted.clone(),
        terms_accepted_at:data.terms_accepted_at.clone(),
    };

    diesel::insert_into(customer_profiles::table)
        .values(customerprofile)
        .returning(CustomerProfiles::as_returning())
        .get_result(conn)
        .expect("Unable to push the customer profile data into the databse")
}

pub fn create_client_profile(data:&ClientProfileExtract,conn:&mut PgConnection)-> ClientProfiles{

    let clientprofile = NewClientProfiles{
        user_id:data.user_id.clone(),
        company_name:data.company_name.clone(),
        business_type:data.business_type.clone(),
        gst_number:data.gst_number.clone(),
        pan_number:data.pan_number.clone(),
        business_phone:data.business_phone.clone(),
        business_email:data.business_email.clone(),
        address_line1:data.address_line1.clone(),
        address_line2:data.address_line2.clone(),
        city:data.city.clone(),
        state:data.state.clone(),
        pincode:data.pincode.clone(),
        country:data.country.clone(),
        is_tamil_nadu:data.is_tamil_nadu.clone(),
        tamil_nadu_compliance:data.tamil_nadu_compliance.clone(),
        terms_accepted:data.terms_accepted.clone(),
        terms_accepted_at:data.terms_accepted_at.clone(),
        verification_status:data.verification_status.clone()
    };

    diesel::insert_into(client_profiles::table)
        .values(clientprofile)
        .returning(ClientProfiles::as_returning())
        .get_result(conn)
        .expect("unable to push the client profile data into the database")
    
}