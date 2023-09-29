use axum::{
    routing::{get, put},
    Router,
    extract::{self, Query},
    Extension,
    Form,
    response::{
        AppendHeaders, 
        IntoResponse,
        sse::{Sse, Event},
    }, 
    http::{StatusCode, header::SET_COOKIE},
};
use futures_core::stream::Stream;
use axum_extra::extract::{cookie::Cookie, CookieJar};
use maud::html;
use serde_json;
use serde::Deserialize;
use theme::ColorScheme;
use tower_http::{
    trace::{self, TraceLayer},
    services::ServeDir,
};
use tracing::{Level, log::error};

use std::{
    convert::Infallible, 
    time::Duration,
    env,
    fs,
    sync::Arc,
    net::SocketAddr,
    sync::Mutex,
};

mod icon;
mod component;
mod template;
mod page;
mod theme;
mod llama;

#[tokio::main]
async fn main() {
    let out_path = env!("OUT_DIR");
    let assets_path = format!("{out_path}/assets");

    let llama = match llama::Llama::new(
        "models/llama-2-7b.Q2_K.gguf",
        "models/tokenizer.json",
        llama::Config { ..Default::default()},
    ) {
        Ok(llama) => llama,
        Err(e) => {
            panic!("Failed to load llama: {:?}", e);
        },
    };
    let shared_llama_mutex = Arc::new(Mutex::new(llama));

    // Will eventually remove and store actual message in postgres
    let fake_messages = fs::read_to_string("./fake-messages.json")
        .expect("Should be able to read fake-messages.json to string");
    let fm_list: Vec<page::FakeMessage> = serde_json::from_str(&fake_messages)
        .expect("Should be able to parse fake-message json from string");
    let shared_fm_list = Arc::new(fm_list);

    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
    tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::INFO)
        .with_writer(non_blocking)
        .init();
    
    let app = Router::new()
        .route("/", get(home))
        .route("/admin", get(admin))
        .route("/conversations", get(conversations))
        .route("/conversations/:id", get(conversation).post(message))
        .layer(axum::Extension(shared_fm_list))
        .route("/chatbot", get(chatbot))
        .layer(axum::Extension(shared_llama_mutex))
        .route("/settings", get(settings))
        .route("/settings/theme", put(settings_theme))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new()
                    .level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new()
                    .level(Level::INFO)),
        )
        .nest_service("/assets", ServeDir::new(&assets_path));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn home(jar: CookieJar) -> impl IntoResponse {
    let (color_scheme, jar) = init_and_extract_theme(jar);
    (
        jar,
        html! {
            (template::head("Cait - Home", color_scheme.derive_class()))
            (page::home())
        }
    )
}

async fn admin(jar: CookieJar) -> impl IntoResponse {
    let (color_scheme, jar) = init_and_extract_theme(jar);
    (
        jar,
        html! {
            (template::head("Cait - Admin", color_scheme.derive_class()))
            (page::admin())
        }
    )
}

async fn conversations(
    Extension(fm_list): Extension<Arc<Vec<page::FakeMessage>>>,
    jar: CookieJar,
) -> impl IntoResponse {
    let (color_scheme, jar) = init_and_extract_theme(jar);
    (
        jar,
        html! {
            (template::head("Cait - Conversations", color_scheme.derive_class()))
            (page::conversations(fm_list.as_ref()))
        }
    )
}

async fn conversation(
    extract::Path(id): extract::Path<String>, 
    Extension(fm_list): Extension<Arc<Vec<page::FakeMessage>>>,
    jar: CookieJar
) -> impl IntoResponse {
    let (color_scheme, jar) = init_and_extract_theme(jar);
    (
        jar,
        html! {
            (template::head(&format!("cait - {id}"), color_scheme.derive_class()))
            (page::conversation(&id, fm_list.as_ref()))
        }
    )
}

#[derive(Deserialize)]
struct Message {
    agent: String,
    content: String,
}

async fn message(m: Form<Message>) -> impl IntoResponse {
    let agent = page::str_to_agent(m.agent.as_str());
    html! {
        (component::message(agent, m.content.as_str(), false))
        (component::message(page::Agent::Chatbot, m.content.as_str(), true))
    }
}

async fn settings(jar: CookieJar) -> impl IntoResponse {
    let (color_scheme, jar) = init_and_extract_theme(jar);
    (
        jar,
        html! {
            (template::head("Cait - Settings", color_scheme.derive_class()))
            (page::settings(color_scheme))
        }
    )
}

async fn settings_theme(Form(theme_form): Form<ThemeForm>) -> impl IntoResponse {
    let color_scheme = extract_theme_from_form(theme_form);
    (
        AppendHeaders([(SET_COOKIE, color_scheme.color_mode_cookie())]),
        AppendHeaders([(SET_COOKIE, color_scheme.selected_color_cookie())]),
        html! {
            (component::theme_preference(color_scheme, true))
        }
    )
}

fn init_and_extract_theme(jar: CookieJar) -> (ColorScheme, CookieJar) {
    if let Some(color_mode_cookie) = jar.get("color_mode") {
        if let Some(selected_color_cookie) = jar.get("selected_color") {
            match ColorScheme::from_string(color_mode_cookie.value(), selected_color_cookie.value()) {
                Ok(color_scheme) => return (color_scheme, jar),
                Err(e) => {
                    error!("Failed to parse color scheme from cookies: {:?}", e);
                },
            }
        }
    }
    
    let color_scheme = ColorScheme::new();
    let color_mode_cookie = Cookie::build("color_mode", color_scheme.color_mode_string())
        .path("/").secure(true).same_site(cookie::SameSite::Lax).http_only(true).finish();
    let selected_color_cookie = Cookie::build("selected_color", color_scheme.selected_color_string())
        .path("/").secure(true).same_site(cookie::SameSite::Lax).http_only(true).finish();
    (color_scheme, jar.add(color_mode_cookie).add(selected_color_cookie))
}

#[derive(Deserialize)]
pub struct ThemeForm {
    color_mode: String,
    selected_color: String,
}

fn extract_theme_from_form(theme: ThemeForm) -> ColorScheme {
    ColorScheme::from_string(&theme.color_mode, &theme.selected_color)
        .unwrap_or_else(|e| {
            error!("Failed to parse color scheme from string: {:?}", e);
            ColorScheme::new()
        })
}

async fn chatbot(
    m: Query<Message>,
    Extension(llama_mutex): Extension<Arc<Mutex<llama::Llama>>>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    tracing::info!("prompt: {}", m.content);
    
    let Ok(llama) = llama_mutex.lock() else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    tracing::info!("Got llama lock");
    let token_stream = llama.run(m.content.clone());
    let event_stream = stream_events(token_stream);

    Ok(Sse::new(event_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    ))
}

fn stream_events<S: Stream<Item = Result<String, String>>>(s: S) -> impl Stream<Item = Result<Event, Infallible>> {
    async_stream::stream! {
        for await message in s {
            match message {
                Ok(message) => {
                    let html_fragment = format!("<span>{}</span>", message);
                    tracing::info!("response: {}", html_fragment);
                    yield Ok(Event::default().event("chatbot").data(html_fragment));
                },
                Err(e) => {
                    error!("Llama error: {:?}", e);
                    yield Ok(Event::default().event("error").data("error with stream"));
                },
            }
        }
    }
}