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
                    disabled[is_system] class="hidden partner" hx-put="/settings/theme";
                label for="light-theme" class="bg-white text-black w-5 p-0.4 border-0.1 rounded-0.4 transition
                    border-gray-600 partner:checked:border-green-400 partner:disabled:opacity-50 text-center
                    partner:disabled:cursor-not-allowed select-none cursor-pointer partner:checked:cursor-default
                    flex justify-between items-center" {
                    div class="w-1.4 h-1.4" {
                        (icon::sun())
                    }
                    "Light"
                    //img class="htmx-indicator transition" {
                    //    (icon::oval_indicator())
                    //}
                }
                @if is_system && is_dark { input type="hidden" name="selected_color" value="dark"; }
                input #dark-theme type="radio" name="selected_color" value="dark" checked[is_dark]
                    disabled[is_system] class="hidden partner" hx-put="/settings/theme";
                label for="dark-theme" class="bg-black text-white w-5 p-0.4 border-0.1 rounded-0.4 transition
                    border-gray-300 partner:checked:border-green-400 partner:disabled:opacity-50 text-center
                    partner:disabled:cursor-not-allowed select-none cursor-pointer partner:checked:cursor-default
                    flex justify-between items-center" {
                    div class="w-1.4 h-1.4" {
                        (icon::moon())
                    }
                    "Dark" 
                    //img class="htmx-indicator transition" {
                    //    (icon::oval_indicator())
                    //}
                }
            }
        }
    }
}

/*
pub fn theme_toggle(checked: bool, disabled: bool) -> Markup {
    html! {
        label #theme-toggle class="flex justify-between items-center relative gap-1" {
            span .select-none."opacity-50"[disabled].transition."duration-30" { "Light" }

            input #selected-color name="selected_color" value="dark" type="checkbox" checked[checked] disabled[disabled]
                class="m-0 left-0 bottom-0 w-4 h-2 rounded-2 transition cursor-pointer 
                    appearance-none disabled:cursor-not-allowed bg-gold duration-30
                    checked:bg-blue-night disabled:bg-opacity-50 checked:disabled:bg-opacity-22
                    dark:disabled:bg-opacity-35 dark:checked:disabled:bg-opacity-50 partner";

            span class="absolute content-empty cursor-pointer w-1.6 h-1.6 right-1/2+0.1 rounded-full
                        bg-gold-dark transition duration-30 partner:disabled:bg-opacity-50 
                        partner:disabled:cursor-not-allowed partner:checked:bg-blue-twilight
                        transform partner:checked:translate-x-2 partner:checked:disabled:bg-opacity-30
                        dark:partner:disabled:bg-opacity-30 dark:partner:checked:disabled:bg-opacity:35
                        parent"
            {
                span class="absolute rounded-full top-1/2-0.4 left-1/2-0.4 w-0.8 h-0.8 transition
                            transform scale-50 partner:checked:parent:scale-100 duration-30
                            shadow-inner offset-x-0.4 -offset-y-0.4 blur-radius-0 spread-radius-0.5
                            shadow-color-white shadow-opacity-100 
                            partner:disabled:parent:shadow-opacity-50
                            partner:checked:parent:offset-x-0.2 partner:checked:parent:-offset-y-0.2
                            partner:checked:parent:spread-radius-0.2" {}

                @for n in 0..8 {
                    @let deg1 = n * 45;
                    @let deg2 = (n + 1) * 45;
                    span class={ "absolute rounded-0.05 top-1/2 left-1/2-0.05 w-0.1 h-0.2 bg-white transition 
                            origin-center-top transform-rotate-first translate-y-0.4 duration-30
                            partner:disabled:parent:bg-opacity-50 partner:checked:parent:opacity-0
                            partner:checked:parent:translate-y-0.8 
                            rotate-"(deg1) " partner:checked:parent:rotate-"(deg2) } {}
                }
            }
            span .select-none."opacity-50"[disabled].transition."duration-30" { "Dark" }
        }
    }
}
*/

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

/*
script {"document.getElementById('color-mode').addEventListener('change', function() { 
    var selectedColor = document.getElementById('selected-color');
    var element = document.documentElement;
    if (this.value === 'system') {
        element.classList.remove('light');
        element.classList.remove('dark');
    } else {
        if (selectedColor.checked) {
            element.classList.add('dark');
        } else {
            element.classList.add('light');
        }
    }
    htmx.trigger('#color-scheme', 'change-theme', {});
 });
 document.getElementById('selected-color').addEventListener('change', function() {
    var colorMode = document.getElementById('color-mode');
    var element = document.documentElement;
    if (colorMode.value === 'select') {
        if (this.checked) {
            element.classList.remove('light');
            element.classList.add('dark');
        } else {
            element.classList.remove('dark');
            element.classList.add('light');
        }
    }
    htmx.trigger('#color-scheme', 'change-theme', {});
 });"}
 */