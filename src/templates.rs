use maud::{DOCTYPE, html, Markup};

/// A basic header with a dynamic `page_title`.
pub fn head(page_title: &str, theme: &str) -> Markup {
    html! {
        (DOCTYPE)
        html class=(theme) lang="en-US" {
            head {
                meta charset="utf-8";
                title { (page_title) }
                link rel="stylesheet" type="text/css" href="/assets/utils.css";
                script src="/assets/htmx.min.js" defer;
            }
        }
    }
}