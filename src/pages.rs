use maud::{html, Markup};
use crate::templates::head;
use crate::color_scheme::{ColorScheme, ColorMode, Theme};

pub fn settings(color_scheme: ColorScheme) -> Markup {
    let color_mode = color_scheme.color_mode();
    let is_system = color_mode == ColorMode::System;
    let is_select = color_mode == ColorMode::Select;
    let selected_color = color_scheme.selected_color();
    let is_dark = selected_color == Theme::Dark;
    html! {
        (head("Cait - Settings", color_scheme.class()))
        body {
            header class="flex justify-center items-center fixed top-0 left-0 right-0 min-h-4 
                          bg-white bg-opacity-65 dark:bg-dark dark:bg-opacity-50 blur-0.2" 
            {
                h3 { "Settings" }
            }
            main class="mt-6 mb-4 px-2" {
                h3 { "Theme Preferences" }
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
                                   dark:disabled:bg-opacity-35 dark:checked:disabled:bg-opacity-50 sibling";

                        span class="absolute content-empty cursor-pointer w-1.6 h-1.6 right-1/2+0.1 rounded-full
                                    bg-gold-dark duration-30 sibling:disabled:bg-opacity-50 
                                    sibling:disabled:cursor-not-allowed sibling:checked:bg-blue-twilight
                                    transform sibling:checked:translate-x-2 sibling:checked:disabled:bg-opacity-30
                                    dark:sibling:disabled:bg-opacity-30 dark:sibling:checked:disabled:bg-opacity:35"
                        {
                            span {}
                            span {}
                            span {}
                            span {}
                            span {}
                            span {}
                            span {}
                            span {}
                            span {}
                        }
                        span #light-span .select-none."opacity-50"[is_system] { "Dark" }
                    }
                }
            }
        }
    }
}