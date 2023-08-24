use axum::{
    response::IntoResponse,
    http::{Request, header, HeaderValue},
    middleware,
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use serde_json::{self, Value};

#[derive(Clone, Copy, PartialEq)]
pub enum ColorMode {
    Select,
    System,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Theme {
    Light,
    Dark,
}

#[derive(Clone)]
pub struct ColorScheme {
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

    fn color_mode_string(&self) -> &str {
        match self.color_mode {
            ColorMode::Select => "select",
            ColorMode::System => "system",
        }
    }

    fn selected_color_string(&self) -> &str {
        match self.selected_color {
            Theme::Dark => "dark",
            Theme::Light => "light",
        }
    }

    pub fn color_mode(&self) -> ColorMode {
        self.color_mode
    }

    pub fn selected_color(&self) -> Theme {
        self.selected_color
    }

    pub fn class(&self) -> &str {
        self.class.as_str()
    }

    fn to_cookie_value(&self) -> String {
        let color_mode = self.color_mode_string();
        let selected_color = self.selected_color_string();
        format!("{{\"color_mode\":\"{color_mode}\", \"selected_color\":\"{selected_color}\"}}")
    }
}

pub async fn theme<B>(
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