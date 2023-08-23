use maud::{DOCTYPE, html, Markup};
use templates::head;

pub fn settings(theme: &str) -> Markup {
    html! {
        (head("Cait - Settings", theme))
        header class="flex justify-center items-center fixed top-0 left-0 right-0 min-h-4 
                        bg-white bg-opacity-65 dark:bg-dark dark:bg-opacity-50" 
        {
            h3 { "Settings" }
        }
        main {
            h3 { "Theme Preferences" }
        }
    }
}