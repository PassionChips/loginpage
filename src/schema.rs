// @generated automatically by Diesel CLI.

diesel::table! {
    client_profiles (id) {
        id -> Int4,
        user_id -> Int4,
        #[max_length = 255]
        company_name -> Varchar,
        #[max_length = 50]
        business_type -> Varchar,
        #[max_length = 15]
        gst_number -> Varchar,
        #[max_length = 10]
        pan_number -> Varchar,
        #[max_length = 20]
        business_phone -> Varchar,
        #[max_length = 255]
        business_email -> Varchar,
        #[max_length = 255]
        address_line1 -> Varchar,
        #[max_length = 255]
        address_line2 -> Nullable<Varchar>,
        #[max_length = 100]
        city -> Varchar,
        #[max_length = 100]
        state -> Varchar,
        #[max_length = 6]
        pincode -> Varchar,
        #[max_length = 100]
        country -> Varchar,
        is_tamil_nadu -> Bool,
        tamil_nadu_compliance -> Nullable<Bool>,
        terms_accepted -> Bool,
        terms_accepted_at -> Nullable<Timestamp>,
        #[max_length = 20]
        verification_status -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    customer_profiles (id) {
        id -> Int4,
        user_id -> Int4,
        #[max_length = 20]
        phone -> Varchar,
        #[max_length = 20]
        alternate_phone -> Nullable<Varchar>,
        #[max_length = 255]
        address_line1 -> Varchar,
        #[max_length = 255]
        address_line2 -> Nullable<Varchar>,
        #[max_length = 100]
        city -> Varchar,
        #[max_length = 100]
        state -> Varchar,
        #[max_length = 6]
        pincode -> Varchar,
        #[max_length = 100]
        country -> Varchar,
        terms_accepted -> Bool,
        terms_accepted_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        first_name -> Varchar,
        last_name -> Varchar,
        username -> Varchar,
        email -> Varchar,
        password -> Varchar,
        #[max_length = 20]
        role -> Varchar,
        profile_completed -> Bool,
        profile_completed_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(client_profiles -> users (user_id));
diesel::joinable!(customer_profiles -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(client_profiles, customer_profiles, users,);
