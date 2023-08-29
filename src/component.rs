use maud::{html, Markup};
use crate::{theme, icon};

pub fn theme_preference(color_scheme: theme::ColorScheme) -> Markup {
    let color_mode = color_scheme.color_mode();
    let is_system = color_mode == theme::ColorMode::System;
    let is_select = color_mode == theme::ColorMode::Select;
    let selected_color = color_scheme.selected_color();
    let is_dark = selected_color == theme::Theme::Dark;

    html! {
        div class="flex gap-3" {
            select #system-or-select {
                option value="system" selected[is_system] { "Sync with system" }
                option value="select" selected[is_select] { "Single theme" }
            }
            label class="flex justify-between items-center relative gap-1" {
                span #light-span .select-none."opacity-50"[is_system] { "Light" }

                input #theme-toggle type="checkbox" checked[is_dark] disabled[is_system]
                    class="m-0 left-0 bottom-0 w-4 h-2 rounded-2 duration-18 cursor-pointer 
                        appearance-none disabled:cursor-not-allowed bg-gold
                        checked:bg-blue-night disabled:bg-opacity-50 checked:disabled:bg-opacity-22
                        dark:disabled:bg-opacity-35 dark:checked:disabled:bg-opacity-50 partner";

                span class="absolute content-empty cursor-pointer w-1.6 h-1.6 right-1/2+0.1 rounded-full
                            bg-gold-dark duration-30 partner:disabled:bg-opacity-50 
                            partner:disabled:cursor-not-allowed partner:checked:bg-blue-twilight
                            transform partner:checked:translate-x-2 partner:checked:disabled:bg-opacity-30
                            dark:partner:disabled:bg-opacity-30 dark:partner:checked:disabled:bg-opacity:35
                            parent"
                {
                    span class="absolute rounded-full top-1/2-0.4 left-1/2-0.4 w-0.8 h-0.8 transition
                                duration-30 transform scale-50 partner:checked:parent:scale-100  
                                shadow-inner offset-x-0.4 -offset-y-0.4 blur-radius-0 spread-radius-0.5
                                shadow-color-white shadow-opacity-100 
                                partner:disabled:parent:shadow-opacity-50
                                partner:checked:parent:offset-x-0.2 partner:checked:parent:-offset-y-0.2
                                partner:checked:parent:spread-radius-0.2" {}
    
                    @for n in 0..8 {
                        @let deg1 = n * 45;
                        @let deg2 = (n + 1) * 45;
                        span class={ "absolute rounded-0.05 top-1/2 left-1/2-0.05 w-0.1 h-0.2 bg-white transition 
                                duration-30 origin-center-top transform-rotate-first translate-y-0.4 
                                partner:disabled:parent:bg-opacity-50 partner:checked:parent:opacity-0
                                partner:checked:parent:translate-y-0.8 
                                rotate-"(deg1) " partner:checked:parent:rotate-"(deg2) } {}
                    }
                }
                span #light-span .select-none."opacity-50"[is_system] { "Dark" }
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