use serde_json::{self, Value};

#[derive(Debug)]
pub enum ColorSchemeError {
    Json(serde_json::Error),
    InvalidColorMode,
    InvalidSelectedColor,
}

impl From<serde_json::Error> for ColorSchemeError {
    fn from(err: serde_json::Error) -> ColorSchemeError {
        ColorSchemeError::Json(err)
    }
}

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
}

impl ColorScheme {
    pub fn new() -> ColorScheme {
        ColorScheme { 
            color_mode: ColorMode::System,
            selected_color: Theme::Light,
        }
    }

    pub fn from_string(color_mode_string: &str, selected_color_string: &str) -> Result<ColorScheme, ColorSchemeError> {
        let color_mode: ColorMode = match color_mode_string {
            "system" => ColorMode::System,
            "select" => ColorMode::Select,
            _ => return Err(ColorSchemeError::InvalidColorMode),
        };
        let selected_color: Theme = match selected_color_string {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => return Err(ColorSchemeError::InvalidSelectedColor),
        };
        Ok(ColorScheme { color_mode, selected_color })
    }

    /* 
    pub fn from_json(color_scheme_json: &str) -> Result<ColorScheme, ColorSchemeError> {
        let v: Value = serde_json::from_str(color_scheme_json)?;
    
        let color_mode: ColorMode = match &v["color_mode"] {
            Value::String(mode) => match mode.as_str() {
                "system" => ColorMode::System,
                "select" => ColorMode::Select,
                _ => return Err(ColorSchemeError::InvalidColorMode),
            },
            _ => return Err(ColorSchemeError::InvalidColorMode),
        };
        
        let selected_color: Theme = match &v["selected_color"] {
            Value::String(theme) => match theme.as_str() {
                "dark" => Theme::Dark,
                "light" => Theme::Light,
                _ => return Err(ColorSchemeError::InvalidSelectedColor),
            },
            _ => return Err(ColorSchemeError::InvalidSelectedColor),
        };
        
        Ok(ColorScheme { color_mode, selected_color })
    }
    */

    pub fn color_mode_string(&self) -> String {
        match self.color_mode {
            ColorMode::System => String::from("system"),
            ColorMode::Select => String::from("select"),
        }
    }

    pub fn selected_color_string(&self) -> String {
        match self.selected_color {
            Theme::Dark => String::from("dark"),
            Theme::Light => String::from("light"),
        }
    }

    pub fn color_mode(&self) -> ColorMode {
        self.color_mode
    }

    pub fn selected_color(&self) -> Theme {
        self.selected_color
    }

    pub fn derive_class(&self) -> String {
        match self.color_mode {
            ColorMode::System => String::from("system"),
            ColorMode::Select => match self.selected_color {
                Theme::Dark => String::from("dark"),
                Theme::Light => String::from("light"),
            }
        }
    }
    
    pub fn color_mode_cookie(&self) -> String {
        format!("color_mode={}; Path=/; Secure; SameSite=Lax; HttpOnly", self.color_mode_string())
    }

    pub fn selected_color_cookie(&self) -> String {
        format!("selected_color={}; Path=/; Secure; SameSite=Lax; HttpOnly", self.selected_color_string())
    }
    
    /*
    pub fn to_cookie(&self) -> String {
        let color_mode = self.color_mode_string();
        let selected_color = self.selected_color_string();
        format!("color_scheme={{\"color_mode\":\"{color_mode}\", \"selected_color\":\"{selected_color}\"}}")
    }
    
    pub fn to_cookie(&self) -> String {
        let color_mode = self.color_mode_string();
        let selected_color = self.selected_color_string();
        format!("color_scheme={{\"color_mode\":\"{color_mode}\", \"selected_color\":\"{selected_color}\"}}; Path=/; Secure; SameSite=Lax; HttpOnly")
    }

    pub fn to_cookie_value(&self) -> String {
        let color_mode = self.color_mode_string();
        let selected_color = self.selected_color_string();
        format!("{{\"color_mode\":\"{color_mode}\", \"selected_color\":\"{selected_color}\"}}")
    }
    */
}

/*
pub async fn extract_theme<B>(
    mut req: Request<B>,
    next: middleware::Next<B>,
) -> impl IntoResponse {
    
    let jar = CookieJar::from_headers(req.headers());
    
    if let Some(cookie) = jar.get("color_scheme") {
        let color_scheme = ColorScheme::from_json(cookie.value())
            .unwrap_or_else(|_| ColorScheme::new());
        req.extensions_mut().insert(color_scheme);
        let res = next.run(req).await;
        return res
    
    }
    let color_scheme = ColorScheme::new();
    req.extensions_mut().insert(color_scheme.clone());
    let mut res = next.run(req).await;
    
    if let Ok(color_scheme_cookie) = HeaderValue::from_str(&color_scheme.to_cookie()) {
        res.headers_mut().insert(
            header::SET_COOKIE,
            color_scheme_cookie,
        );
    }
    res
}
*/