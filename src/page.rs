use maud::{html, Markup};
use serde::{Deserialize, Serialize};

use crate::template;
use crate::component;
use crate::icon;
use crate::theme;


#[derive(PartialEq)]
pub enum Pathname { Home, Admin, Conversations, Settings }

#[derive(PartialEq, Clone, Copy)]
pub enum Agent { User, Chatbot, Other }

pub fn str_to_agent(s: &str) -> Agent {
    match s {
        "user" => Agent::User,
        "chatbot" => Agent::Chatbot,
        _ => Agent::Other,
    }
}

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

pub fn conversation(id: &str, messages: &Vec<FakeMessage>) -> Markup {
    html! {
        body {
            (template::top_navbar(
                id, 
                component::primary_svg_button("/conversations", icon::arrow_left()), 
                html! { div {} },
            ))
            div #messages class="flex flex-col items-center w-full" {
                @for msg in messages {
                    (component::message(str_to_agent(msg.from.as_str()), msg.content.as_str(), false))
                }
            }
            div #bottom-spacer class="w-full min-h-4" {}
            div class="fixed bottom-0 left-0 right-0 py-2 flex justify-center blur-0.5" {
                (component::prompt_input(id))
            }
            
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