use maud::{html, Markup};
use crate::{theme::{ColorScheme, ColorMode, Theme}, icon};

pub fn theme_preference(color_scheme: ColorScheme, set_theme: bool) -> Markup {
    
    let (is_system, is_select) = match color_scheme.color_mode() {
        ColorMode::System => (true, false),
        ColorMode::Select => (false, true),
    };
    let (is_light, is_dark) = match color_scheme.selected_color() {
        Theme::Dark => (false, true),
        Theme::Light => (true, false),
    };

    html! {
        form #color-scheme hx-target="this" hx-swap="outerHTML" hx-ext="set-theme"
                set-theme=@if set_theme { (color_scheme.derive_class()) } class="flex gap-3" {
            select #color-mode name="color_mode" hx-put="/settings/theme" {
                option value="system" selected[is_system] { "Sync with system" }
                option value="select" selected[is_select] { "Single theme" }
            }
            div #selected-color class="flex justify-between w-12" {
                @if is_system && is_light { input type="hidden" name="selected_color" value="light"; }
                input #light-theme type="radio" name="selected_color" value="light" checked[is_light]
                    disabled[is_system] class="hidden partner" hx-put="/settings/theme" 
                    hx-indicator="#light-theme-label";
                label #light-theme-label for="light-theme" class="bg-white text-black w-5 p-0.4 border-0.1 rounded-0.4 transition
                    border-gray-600 partner:checked:border-green-400 partner:disabled:opacity-50 text-center
                    partner:disabled:cursor-not-allowed select-none cursor-pointer partner:checked:cursor-default
                    flex justify-between items-center" {
                    div class="w-1.4 h-1.4 htmx-request:opacity-0 transition" {
                        (icon::sun())
                    }
                    img src="assets/tail-spin-black.svg" class="w-1.2 h-1.2 htmx-indicator absolute";
                    "Light"
                }
                @if is_system && is_dark { input type="hidden" name="selected_color" value="dark"; }
                input #dark-theme type="radio" name="selected_color" value="dark" checked[is_dark]
                    disabled[is_system] class="hidden partner" hx-put="/settings/theme" 
                    hx-indicator="#dark-theme-label";
                label #dark-theme-label for="dark-theme" class="bg-black text-white w-5 p-0.4 border-0.1 
                    rounded-0.4 transition border-gray-300 partner:checked:border-green-400 
                    partner:disabled:opacity-50 text-center partner:disabled:cursor-not-allowed select-none 
                    cursor-pointer partner:checked:cursor-default flex justify-between items-center" {
                    div class="w-1.4 h-1.4 htmx-request:opacity-0 transition" {
                        (icon::moon())
                    }
                    img src="assets/tail-spin-white.svg" class="w-1.2 h-1.2 htmx-indicator absolute";
                    "Dark"
                }
            }
        }
    }
}

pub fn search_bar() -> Markup {
    html! {
        form id="search-form" action="?/searchChats" method="post" 
            class="flex justify-center w-full mt-0.5 mb-1 px-2 " {
            label for="search" class="flex flex-grow gap-0.5 bg-gray-200 dark:bg-gray-700 rounded-1 
                max-w-25 max-h-2.2 px-1 text-gray-500 dark:text-gray-400 focus-within:outline-terracotta-400
                focus-within:bg-gray-100 dark:focus-within:bg-gray-700" {
                div class="p-0.4 w-2.2" {
                    (icon::magnifying_glass())
                }
		        input id="search" enterkeyhint="search" placeholder="Search" class="w-full";
            }
        }
    }
}

pub fn edit_button(link: &str) -> Markup {
    html! {
        a href=(link) {
            p class="text-xl text-terracotta-400 h-2 m-0" { "edit" }
        }
    }
}

pub fn primary_svg_button(link: &str, svg: Markup) -> Markup {
    html! {
        a href=(link) class="text-terracotta-400 w-2 h-2" {
            (svg)
        }
    }
}

pub fn prompt_input() -> Markup {
    html! {
        form #send-prompt action="?" method="post" 
            class="flex justify-center w-full px-2" {
            label for="prompt-input" class="flex flex-grow gap-0.5 bg-white dark:bg-black max-w-50
                    focus-within:outline-terracotta-400" {
                input id="prompt-input" enterkeyhint="send" placeholder="Send a message" class="w-full";
                div class="p-0.4 w-2.2 text-terracotta-400" {
                    (icon::paper_airplane())
                }
            }
        }
    }
}