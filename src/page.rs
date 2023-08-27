use maud::{html, Markup};
use serde::{Deserialize, Serialize};

use crate::template;
use crate::component;
use crate::theme;

#[derive(PartialEq)]
pub enum Pathname { Home, Admin, Conversations, Settings }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FakeMessage {
    pub from: String,
    pub content: String,
}

pub fn settings(color_scheme: theme::ColorScheme) -> Markup {
    html! {
        (template::head("Cait - Settings", color_scheme.class()))
        body {
            header class="flex justify-center items-center fixed top-0 left-0 right-0 min-h-4 
                          bg-white bg-opacity-65 dark:bg-dark dark:bg-opacity-50 blur-0.2" 
            {
                h3 { "Settings" }
            }
            main class="mt-6 mb-4 px-2" {
                h3 { "Theme Preferences" }
                (component::theme_preference(color_scheme))
            }
            (template::bottom_navbar(Pathname::Settings))
        }
    }
}

pub fn conversations(color_scheme_class: &str, messages: &Vec<FakeMessage>) -> Markup {
    html! {
        (template::head("Cait - Conversations", color_scheme_class))
        body {
            (component::search_bar())
            (template::messages(messages))
            (template::bottom_navbar(Pathname::Conversations))
        }
    }
}