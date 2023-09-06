#[derive(Debug)]
pub enum ColorSchemeError {
    InvalidColorMode,
    InvalidSelectedColor,
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
}