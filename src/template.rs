use maud::{DOCTYPE, html, Markup};
use crate::{icon, page};

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
            a class="w-5 p-1 text-color-black dark:text-color-white" href="/" aria-label="Home" {
                @if pathname == page::Pathname::Home {
                    (icon::home_filled())
                } @else {
                    (icon::home_oulined())
                }
            }
            a class="w-5 p-1 text-color-black dark:text-color-white" href="/admin" aria-label="Admin" {
                @if pathname == page::Pathname::Admin {
                    (icon::admin_filled())
                } @else {
                    (icon::admin_outlined())
                }
            }
            a class="w-5 p-1 text-color-black dark:text-color-white" href="/conversations" aria-label="Conversations" {
                @if pathname == page::Pathname::Conversations {
                    (icon::conversation_filled())
                } @else {
                    (icon::conversation_outlined())
                }
            }
            a class="w-5 p-1 text-color-black dark:text-color-white" href="/settings" aria-label="Settings" {
                @if pathname == page::Pathname::Settings {
                    (icon::settings_filled())
                } @else {
                    (icon::settings_outlined())
                }
            }
        }
    }
}