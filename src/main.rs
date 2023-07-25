use axum::{
    extract,
    response::Html,
    routing::get,
    Router,
};
use sailfish::TemplateOnce;
use tower_http::services::ServeDir;

#[derive(TemplateOnce)]  // automatically implement `TemplateOnce` trait
#[template(path = "app.stpl")]  // specify the path to template
struct NavbarTemplate {
    // data to be passed to the template
    messages: Vec<String>,
    pathname: String,
}

#[tokio::main]
async fn main() {
    
    let app = Router::new()
        .route("/", get(home))
        .route("/:pathname", get(navbar))
        .nest_service("/assets", ServeDir::new("./assets"));

    println!("\n\tServing on localhost:3000\n");
    
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn home() -> Html<String> {
    let ctx = NavbarTemplate {
        messages: vec![String::from("foo"), String::from("bar")],
        pathname: String::from(""),
    };
    Html(ctx.render_once().unwrap())
}

async fn navbar(extract::Path(pathname): extract::Path<String>) -> Html<String> {
    let ctx = NavbarTemplate {
        messages: vec![String::from("foo"), String::from("bar")],
        pathname,
    };
    Html(ctx.render_once().unwrap())
}

