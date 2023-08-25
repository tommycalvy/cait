//use core::panic;
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
    dbg!(&out_path);

    //println!("cargo:rerun-if-changed={templates_dir}");

    let assets_path = format!("{out_path}/assets");
    if !Path::new(&assets_path).is_dir() {
        fs::create_dir_all(&assets_path).expect("Should be able to create assets directory if not there");
    }

    // Download htmx js library to the assets folder if it doesn't already exist there
    let htmx_file_path = format!("{assets_path}/htmx.min.js");
    if !Path::new(&htmx_file_path).is_file() {
        let htmx_body = reqwest::blocking::get("https://unpkg.com/htmx.org@1.9.4/dist/htmx.min.js")
            .expect("Should be able to download htmx source code");
        let htmx_text = htmx_body.text().expect("Should be able to convert htmx body to text");
        fs::write(htmx_file_path, htmx_text).expect("Should be able to write htmx text to file");
    }


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
        //minify: true,
        targets,
        ..PrinterOptions::default()
    }).unwrap();

    fs::write(styles_file_path, res.code).expect("Should be able to write minified css string to file");
    //panic!()
    
}