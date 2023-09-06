use maud::{html, Markup};
use serde::{Deserialize, Serialize};

use crate::template;
use crate::component;
use crate::icon;
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
        body {
            (template::top_navbar("Settings", html! { div {} }, html! { div {}}))
            main class="mt-6 mb-4 px-2" {
                h3 { "Theme Preferences" }
                (component::theme_preference(color_scheme, false))
            }
            (template::bottom_navbar(Pathname::Settings))
        }
    }
}

pub fn conversations(messages: &Vec<FakeMessage>) -> Markup {
    html! {
        body {
            (template::top_navbar(
                "Conversations", 
                component::edit_button("conversations/edit"), 
                component::primary_svg_button("conversations/tommy", icon::plus()),
            ))
            (component::search_bar())
            (template::messages(messages))
            (template::bottom_navbar(Pathname::Conversations))
        }
    }
}

pub fn conversation(title: &str, messages: &Vec<FakeMessage>) -> Markup {
    html! {
        body {
            (template::top_navbar(
                title, 
                component::primary_svg_button("/conversations", icon::arrow_left()), 
                html! { div {} },
            ))
            (template::messages(messages))
            (template::conversations_input())
        }
    }
}

pub fn home() -> Markup {
    html! {
        body {
            (template::bottom_navbar(Pathname::Home))
        }
    }
}

pub fn admin() -> Markup {
    html! {
        body {
            (template::bottom_navbar(Pathname::Admin))
        }
    }
}