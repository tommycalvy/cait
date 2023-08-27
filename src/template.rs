use maud::{DOCTYPE, html, Markup};
use crate::{icon, page::{self, FakeMessage}};

/// A basic header with a dynamic `page_title`.
pub fn head(page_title: &str, theme: &str) -> Markup {
    html! {
        (DOCTYPE)
        html class=(theme) lang="en-US" {
            head {
                meta charset="utf-8";
                title { (page_title) }
                link rel="stylesheet" type="text/css" href="/assets/utils.css";
                script src="/assets/htmx.min.js" defer {};
            }
        }
    }
}

pub fn bottom_navbar(pathname: page::Pathname) -> Markup {
    html! {
        nav class="fixed bottom-0 left-0 right-0 flex justify-around items-center blur-0.5" {
            a class="w-5 p-1 text-black dark:text-white" href="/" aria-label="Home" {
                @if pathname == page::Pathname::Home {
                    (icon::home_filled())
                } @else {
                    (icon::home_oulined())
                }
            }
            a class="w-5 p-1 text-black dark:text-white" href="/admin" aria-label="Admin" {
                @if pathname == page::Pathname::Admin {
                    (icon::admin_filled())
                } @else {
                    (icon::admin_outlined())
                }
            }
            a class="w-5 p-1 text-black dark:text-white" href="/conversations" aria-label="Conversations" {
                @if pathname == page::Pathname::Conversations {
                    (icon::conversation_filled())
                } @else {
                    (icon::conversation_outlined())
                }
            }
            a class="w-5 p-1 text-black dark:text-white" href="/settings" aria-label="Settings" {
                @if pathname == page::Pathname::Settings {
                    (icon::settings_filled())
                } @else {
                    (icon::settings_outlined())
                }
            }
        }
    }
}

pub fn messages(messages: &Vec<FakeMessage>) -> Markup {
    html! {
        main class="flex flex-col items-center w-full" {
            @for msg in messages {
                @let is_user = msg.from == "user";
                @let is_chatbot = msg.from == "chatbot";
                div .flex.justify-center.w-full."pt-2"."pr-2"."pb-3"."pl-1"
                    ."bg-gray-100"[is_chatbot]."dark:bg-gray-100"[is_chatbot] {
                    div class="flex w-50" {
                        div class="w-5" {
                            div ."w-3"."h-3".rounded-full."mx-1".bg-dark-cyan[is_user].bg-dark-magenta[is_chatbot] {}
                        }
                        p {
                            (msg.content)
                        }
                    }
                }
            }
        }
    }
}