use axum::response::Html;
use axum::{extract::{Json,State},response::Json as ResponseJson};
use serde_json::{Value, json};
use crate::dbutils::dbconfig::Appstate;
use crate::model::profile::{ClientProfileExtract,CustomerProfileExtract};
use crate::dbutils::write_profile::{create_client_profile,create_customer_profile};
use axum::{Router,routing::{get,post}};
use std::sync::Arc;

pub async fn customerprofile(State(state): State<Arc<Appstate>>,Json(data): Json<CustomerProfileExtract>)->ResponseJson<Value>{

    let mut conn = state.pool.get().expect("unable to get pooled connection");
    
    create_customer_profile(&data, &mut conn);

    ResponseJson(json!({
        "result":true
    }))

}

pub async fn clientprofile(State(state): State<Arc<Appstate>>,Json(data): Json<ClientProfileExtract>)->ResponseJson<Value>{

    let mut conn = state.pool.get().expect("unable to get pooled connection from the dbpool");
    create_client_profile(&data, &mut conn);

    ResponseJson(json!({
        "result":true
    }))
}

pub async fn client_info()->Html<&'static str>{
    let html_content = include_str!("../static/templates/client_info.html");
    Html(html_content)
}

pub async fn customer_info()->Html<&'static str>{
    let html_content= include_str!("../static/templates/customer_info.html");
    Html(html_content)
}

pub fn  profile_router()->Router<Arc<Appstate>>{
    Router::new()
        .route("/customerprofilecreation", get(customer_info))
        .route("/customerprofilecreation", post(customerprofile))
        .route("/clientprofileextraction", get(client_info))
        .route("/clientprofileextraction", post(clientprofile))
}
