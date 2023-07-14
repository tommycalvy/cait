use axum::{
    extract,
    response::Html,
    routing::get,
    Router,
};
use sailfish::TemplateOnce;
use tower_http::services::ServeDir;
use std::fs;
use walkdir::WalkDir;
use lightningcss::stylesheet::{
    StyleSheet, ParserOptions, MinifyOptions, PrinterOptions
};

#[derive(TemplateOnce)]  // automatically implement `TemplateOnce` trait
#[template(path = "hello.stpl")]  // specify the path to template
struct NavbarTemplate {
    // data to be passed to the template
    messages: Vec<String>,
    pathname: String,
}

#[tokio::main]
async fn main() {
    let mut css_string_combo = String::new();
    for entry in WalkDir::new("./templates").into_iter()
            .filter_map(|e| e.ok()) {
        let f_name = entry.file_name().to_string_lossy();
        if f_name.ends_with(".css") {
            let css_string = fs::read_to_string(entry.path())
                .expect("Should have been able to read string from css file");
            css_string_combo.insert_str(0, &css_string);
        }
    }
    let mut stylesheet = StyleSheet::parse(&css_string_combo, ParserOptions::default()).unwrap();
    stylesheet.minify(MinifyOptions::default()).unwrap();

    // Serialize it to a string.
    let res = stylesheet.to_css(PrinterOptions::default()).unwrap();
    fs::write("./assets/styles.css", res.code).expect("Should be able to write minified css string to file");

    let app = Router::new()
        .route("/", get(home))
        .route("/:pathname", get(navbar))
        .nest_service("/assets", ServeDir::new("./assets"));

    
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn home() -> Html<String> {
    let ctx = NavbarTemplate {
        messages: vec![String::from("foo"), String::from("bar")],
        pathname: String::from(""),
    };
    Html(ctx.render_once().unwrap())
}

async fn navbar(extract::Path(pathname): extract::Path<String>) -> Html<String> {
    let ctx = NavbarTemplate {
        messages: vec![String::from("foo"), String::from("bar")],
        pathname,
    };
    Html(ctx.render_once().unwrap())
}

