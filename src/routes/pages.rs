use axum::response::Html;

pub async fn index()->Html<&'static str>{
    let html_content = include_str!("../static/templates/index.html");
    Html(html_content)
}
pub async fn success()->Html<&'static str>{
    let html_content = include_str!("../static/templates/success.html");
    Html(html_content)
}

pub async fn error()-> Html<&'static str>{
    let html_content = include_str!("../static/templates/error.html");
    Html(html_content)
}
