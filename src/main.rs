use axum::{
    extract,
    response::Html,
    routing::get,
    Router,
};
use sailfish::TemplateOnce;
use tower_http::services::ServeDir;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::fs;
use std::sync::Arc;

#[derive(TemplateOnce)]  // automatically implement `TemplateOnce` trait
#[template(path = "app.stpl")]  // specify the path to template
struct NavbarTemplate {
    // data to be passed to the template
    messages: Vec<String>,
    pathname: String,
}

#[derive(TemplateOnce)]  // automatically implement `TemplateOnce` trait
#[template(path = "chats/chats.stpl")]  // specify the path to template
struct ChatsTemplate<'a> {
    messages: &'a Vec<FakeMessage>,
    pathname: &'a str,
}

#[tokio::main]
async fn main() {
    let out_path = env!("OUT_DIR");
    let assets_path = format!("{out_path}/assets");

    let fake_messages = fs::read_to_string("./src/fake-messages.json")
        .expect("Should be able to read fake-messages.json to string");
    let fm_list: Vec<FakeMessage> = serde_json::from_str(&fake_messages)
        .expect("Should be able to parse fake-message json from string");
    let shared_fm_list = Arc::new(fm_list);
    

    
    let app = Router::new()
        .route("/", get(home))
        .route(
            "/chats", 
            get({
                let shared_fm_list = Arc::clone(&shared_fm_list);
                move || chats(shared_fm_list)
            }),
        )
        .route("/:pathname", get(navbar))
        .nest_service("/assets", ServeDir::new(&assets_path));

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

async fn chats(fm_list: Arc<Vec<FakeMessage>>) -> Html<String> {
    let ctx = ChatsTemplate {
        messages: fm_list.as_ref(),
        pathname: "chats",
    };
    Html(ctx.render_once().unwrap())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FakeMessage {
    from: String,
    content: String,
}
