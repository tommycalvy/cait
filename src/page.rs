use maud::{html, Markup};
use crate::template;
use crate::component;
use crate::color_scheme::ColorScheme;

#[derive(PartialEq)]
pub enum Pathname { Home, Admin, Conversations, Settings }

pub fn settings(color_scheme: ColorScheme) -> Markup {
    
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