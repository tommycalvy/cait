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
        (template::head("Cait - Settings", color_scheme.class()))
        body {
            (template::top_navbar("Settings", html! { div {} }, html! { div {}}))
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

pub fn conversation(color_scheme_class: &str, title: &str, messages: &Vec<FakeMessage>) -> Markup {
    html! {
        (template::head("Cait - Conversation", color_scheme_class))
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

pub fn home(color_scheme_class: &str) -> Markup {
    html! {
        (template::head("Cait - Home", color_scheme_class))
        body {
            (template::bottom_navbar(Pathname::Home))
        }
    }
}

pub fn admin(color_scheme_class: &str) -> Markup {
    html! {
        (template::head("Cait - Admin", color_scheme_class))
        body {
            (template::bottom_navbar(Pathname::Admin))
        }
    }
}