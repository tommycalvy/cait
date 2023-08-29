use axum::{
    middleware,
    routing::get,
    Router,
};
use maud::{html, Markup};
use tower_http::services::ServeDir;
use serde_json;
use std::env;
use std::fs;
use std::sync::Arc;

mod icon;
mod component;
mod template;
mod page;
mod theme;

#[tokio::main]
async fn main() {
    let out_path = env!("OUT_DIR");
    let assets_path = format!("{out_path}/assets");

    // Will eventually remove and store actual message in postgres
    let fake_messages = fs::read_to_string("./fake-messages.json")
        .expect("Should be able to read fake-messages.json to string");
    let fm_list: Vec<page::FakeMessage> = serde_json::from_str(&fake_messages)
        .expect("Should be able to parse fake-message json from string");
    let shared_fm_list = Arc::new(fm_list);
    
    let app = Router::new()
        .route("/", get(home))
        .route("/conversations", get(conversations))
        .route("/conversations/:id", get(conversation))
        .layer(axum::Extension(shared_fm_list))
        .route("/settings", get(settings))
        .layer(middleware::from_fn(theme::extract_theme))
        .nest_service("/assets", ServeDir::new(&assets_path));

    println!("\n\tServing on localhost:3000\n");
    
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn home(axum::Extension(color_scheme): axum::Extension<theme::ColorScheme>) -> Markup {
    html! {
        (page::home(color_scheme.class()))
    }
}

async fn admin(axum::Extension(color_scheme): axum::Extension<theme::ColorScheme>) -> Markup {
    html! {
        (page::admin(color_scheme.class()))
    }
}

async fn conversations(
    axum::Extension(fm_list): axum::Extension<Arc<Vec<page::FakeMessage>>>,
    axum::Extension(color_scheme): axum::Extension<theme::ColorScheme>,
) -> Markup {
    html! {
        (page::conversations(color_scheme.class(), fm_list.as_ref()))
    }
}


async fn conversation(
    axum::extract::Path(id): axum::extract::Path<String>, 
    axum::Extension(fm_list): axum::Extension<Arc<Vec<page::FakeMessage>>>,
    axum::Extension(color_scheme): axum::Extension<theme::ColorScheme>,
) -> Markup {
    html! {
        (page::conversation(color_scheme.class(), &id, fm_list.as_ref()))
    }
}


async fn settings(axum::Extension(color_scheme): axum::Extension<theme::ColorScheme>) -> Markup {
    html! {
        (page::settings(color_scheme))
    }
}



