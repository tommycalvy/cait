use axum::{
    middleware,
    http::{Request, header, HeaderValue},
    extract,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use maud::Markup;
use maud::html;
use sailfish::TemplateOnce;
use tower_http::services::ServeDir;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::env;
use std::fs;
use std::sync::Arc;
use axum_extra::extract::{CookieJar, cookie::Cookie};

mod templates;
mod pages;

/*
#[derive(TemplateOnce)]  // automatically implement `TemplateOnce` trait
#[template(path = "app.stpl")]  // specify the path to template
struct NavbarTemplate {
    // data to be passed to the template
    messages: Vec<String>,
    pathname: String,
    color_scheme: ColorScheme,
}

#[derive(TemplateOnce)]  
#[template(path = "chats/chats.stpl")]  
struct ChatsTemplate<'a> {
    messages: &'a Vec<FakeMessage>,
    pathname: &'a str,
    color_scheme: ColorScheme,
}


#[derive(TemplateOnce)]  
#[template(path = "settings/settings.stpl")]  
struct SettingsTemplate<'a> {
    pathname: &'a str,
    color_scheme: ColorScheme,
}

#[derive(TemplateOnce)]  
#[template(path = "chats/conversation/conversation.stpl")]  
struct ConversationTemplate<'a> {
    messages: &'a Vec<FakeMessage>,
    id: &'a str,
    color_scheme: ColorScheme,
}
*/


#[tokio::main]
async fn main() {
    let out_path = env!("OUT_DIR");
    let assets_path = format!("{out_path}/assets");

    // Will eventually remove and store actual message in postgres
    let fake_messages = fs::read_to_string("./src/fake-messages.json")
        .expect("Should be able to read fake-messages.json to string");
    let fm_list: Vec<FakeMessage> = serde_json::from_str(&fake_messages)
        .expect("Should be able to parse fake-message json from string");
    let shared_fm_list = Arc::new(fm_list);
    
    let app = Router::new()
        //.route("/", get(home))
        //.route("/chats", get(chats))
        //.route("/chats/:id", get(conversation))
        .layer(axum::Extension(shared_fm_list))
        .route("/settings", get(settings))
        //.route("/:pathname", get(navbar))
        .layer(middleware::from_fn(theme))
        .nest_service("/assets", ServeDir::new(&assets_path));

    println!("\n\tServing on localhost:3000\n");
    
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/*
async fn home(axum::Extension(color_scheme): axum::Extension<ColorScheme>) -> Html<String> {
    let ctx = NavbarTemplate {
        messages: vec![String::from("foo"), String::from("bar")],
        pathname: String::from(""),
        color_scheme,
    };
    Html(ctx.render_once().unwrap())
}

async fn navbar(
    extract::Path(pathname): extract::Path<String>, 
    axum::Extension(color_scheme): axum::Extension<ColorScheme>
) -> Html<String> {
    let ctx = NavbarTemplate {
        messages: vec![String::from("foo"), String::from("bar")],
        pathname,
        color_scheme,
    };
    Html(ctx.render_once().unwrap())
}

async fn chats(
    axum::Extension(fm_list): axum::Extension<Arc<Vec<FakeMessage>>>,
    axum::Extension(color_scheme): axum::Extension<ColorScheme>,
) -> Html<String> {
    let ctx = ChatsTemplate {
        messages: fm_list.as_ref(),
        pathname: "chats",
        color_scheme,
    };
    Html(ctx.render_once().unwrap())
}

async fn conversation(
    extract::Path(id): extract::Path<String>, 
    axum::Extension(fm_list): axum::Extension<Arc<Vec<FakeMessage>>>,
    axum::Extension(color_scheme): axum::Extension<ColorScheme>,
) -> Html<String> {
    let ctx = ConversationTemplate {
        messages: fm_list.as_ref(),
        id: &id,
        color_scheme,
    };
    Html(ctx.render_once().unwrap())
}
*/

async fn settings(axum::Extension(color_scheme): axum::Extension<ColorScheme>) -> Markup {
    html! {
        (pages::settings(&color_scheme.class))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FakeMessage {
    from: String,
    content: String,
}

#[derive(Clone)]
enum ColorMode {
    Select,
    System,
}

#[derive(Clone)]
enum Theme {
    Light,
    Dark,
}

#[derive(Clone)]
struct ColorScheme {
    color_mode: ColorMode,
    selected_color: Theme,
    class: String,
}

impl ColorScheme {
    fn new() -> ColorScheme {
        ColorScheme { 
            color_mode: ColorMode::System,
            selected_color: Theme::Light,
            class: String::from("system"),
        }
    }
    fn from_json(s: &str) -> Result< ColorScheme, serde_json::Error> {
        let v: Value = serde_json::from_str(s)?;
        let color_mode: ColorMode = match &v["color_mode"] {
            Value::String(mode) => match mode.as_str() {
                "select" => ColorMode::Select,
                _ => ColorMode::System
            },
            _ => ColorMode::System,
        };
        let selected_color: Theme = match &v["selected_color"] {
            Value::String(theme) => match theme.as_str() {
                "dark" => Theme::Dark,
                _ => Theme::Light
            },
            _ => Theme::Light,
        };
        let class = match color_mode {
            ColorMode::System => String::from(""),
            ColorMode::Select => match selected_color {
                Theme::Dark => String::from("dark"),
                Theme::Light => String::from("light"),
            }
        };
        Ok(ColorScheme { color_mode, selected_color, class })
    }

    fn color_mode(&self) -> &str {
        match self.color_mode {
            ColorMode::Select => "select",
            ColorMode::System => "system",
        }
    }

    fn selected_color(&self) -> &str {
        match self.selected_color {
            Theme::Dark => "dark",
            Theme::Light => "light",
        }
    }

    fn to_cookie_value(&self) -> String {
        let color_mode = self.color_mode();
        let selected_color = self.selected_color();
        format!("{{\"color_mode\":\"{color_mode}\", \"selected_color\":\"{selected_color}\"}}")
    }
}

async fn theme<B>(
    mut req: Request<B>,
    next: middleware::Next<B>,
) -> impl IntoResponse {
    // transform the request...
    let mut jar = CookieJar::from_headers(req.headers());
    
    let mut color_scheme = ColorScheme::new();
    if let Some(cookie) = jar.get("color_scheme") {
        if let Ok(cs) = ColorScheme::from_json(cookie.value()) {
            color_scheme = cs;
        };
        let theme_cookie = Cookie::build("color_scheme", color_scheme.to_cookie_value())
            .path("/").secure(true).same_site(cookie::SameSite::Lax).finish();
        jar = jar
            .remove(Cookie::named("color_scheme"))
            .add(theme_cookie);
    } else {
        let theme_cookie = Cookie::build("color_scheme", color_scheme.to_cookie_value())
                .path("/").secure(true).same_site(cookie::SameSite::Lax).finish();
        jar = jar
            .add(theme_cookie);
    }

    req.extensions_mut().insert(color_scheme);
    let mut res = next.run(req).await;
    // transform the response...
    let cookie_string: String = jar.iter().map(|c| c.to_string()).collect::<Vec<String>>().join("; ");
    if let Ok(header_value) = HeaderValue::from_str(&cookie_string) {
        res.headers_mut().insert(
            header::SET_COOKIE,
            header_value,
        );
    }
    res
}