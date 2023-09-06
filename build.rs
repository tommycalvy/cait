use std::{
    fs,
    path::Path,
    env,
};
use lightningcss::{
    stylesheet::{ParserOptions, MinifyOptions, PrinterOptions, StyleSheet, ParserFlags},
    targets::{Browsers, Targets},
};
use reqwest;

fn main() {
    let out_path = env::var("OUT_DIR").unwrap();

    let assets_path = format!("{out_path}/assets");
    if !Path::new(&assets_path).is_dir() {
        fs::create_dir_all(&assets_path).expect("Should be able to create assets directory if not there");
    }

    // Download htmx
    let htmx_file_path = format!("{assets_path}/htmx.min.js");
    if !Path::new(&htmx_file_path).is_file() {
        let htmx_body = reqwest::blocking::get("https://unpkg.com/htmx.org@1.9.4/dist/htmx.min.js")
            .expect("Should be able to download htmx source code");
        let htmx_text = htmx_body.text().expect("Should be able to convert htmx body to text");
        fs::write(htmx_file_path, htmx_text).expect("Should be able to write htmx text to file");
    }

    // Download hyperscript
    let hyperscript_file_path = format!("{assets_path}/hyperscript.min.js");
    if !Path::new(&hyperscript_file_path).is_file() {
        let hyperscript_body = reqwest::blocking::get("https://unpkg.com/hyperscript.org@0.9.11")
            .expect("Should be able to download htmx source code");
        let hyperscript_text = hyperscript_body.text().expect("Should be able to convert hyperscript body to text");
        fs::write(hyperscript_file_path, hyperscript_text).expect("Should be able to write hyperscript text to file");
    }

    // Move and TODO: Minify js to the assets path in our_dir
    let set_theme_js = fs::read_to_string("assets/set-theme.js")
        .expect("Couldn't read string from set-theme.js file");
    let set_theme_file_path = format!("{assets_path}/set-theme.js");
    fs::write(set_theme_file_path, set_theme_js).expect("Couldn't write set-theme.js string to file");

    // Move and TODO: Minify js to the assets path in our_dir
    let tail_spin_white_svg = fs::read_to_string("assets/tail-spin-white.svg")
        .expect("Couldn't read string from tail-spin-white.svg file");
    let tail_spin_white_file_path = format!("{assets_path}/tail-spin-white.svg");
    fs::write(tail_spin_white_file_path, tail_spin_white_svg)
        .expect("Couldn't write tail-spin-white.svg string to file");

    // Move and TODO: Minify js to the assets path in our_dir
    let tail_spin_black_svg = fs::read_to_string("assets/tail-spin-black.svg")
        .expect("Couldn't read string from tail-spin-black.svg file");
    let tail_spin_black_file_path = format!("{assets_path}/tail-spin-black.svg");
    fs::write(tail_spin_black_file_path, tail_spin_black_svg)
        .expect("Couldn't write tail-spin-black.svg string to file");


    // lightning css
    let targets: Targets = Targets::from(Browsers {
        safari: Some((9 << 16) | (3 << 8)),
        chrome: Some(69 << 16),
        edge: Some(107 << 16),
        android: Some(50 << 16),
        firefox: Some(102 << 16),
        ie: Some(8 << 16),
        ios_saf: Some((11 << 16) | (3 << 8)),
        opera: Some(94 << 16),
        samsung: Some(4 << 16),
    });

    let css_string = fs::read_to_string("assets/utils.css")
        .expect("Should have been able to read string from css file");


    let styles_file_name = "utils.css";
    let styles_file_path = format!("{assets_path}/{styles_file_name}");

    let mut stylesheet = StyleSheet::parse(
        &css_string, 
        ParserOptions {
            filename: styles_file_name.to_string(),
            flags: ParserFlags::NESTING,
            ..ParserOptions::default()
        }
    ).unwrap();
      
    stylesheet.minify(MinifyOptions {
        targets,
        ..MinifyOptions::default()
    }).unwrap();
    
    let res = stylesheet.to_css(PrinterOptions {
        minify: true,
        targets,
        ..PrinterOptions::default()
    }).unwrap();

    fs::write(styles_file_path, res.code).expect("Should be able to write minified css string to file");
    
}