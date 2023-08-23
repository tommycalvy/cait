//use core::panic;
use std::{
    fs,
    path::Path,
    env,
    collections::HashMap, 
};
use walkdir::{WalkDir, DirEntry};
use lightningcss::{
    stylesheet::{ParserOptions, MinifyOptions, PrinterOptions, StyleSheet, ParserFlags},
    targets::{Browsers, Targets},
    visitor::{Visitor, Visit, VisitTypes},
    traits::{AtRuleParser, ToCss},
    rules::{style::StyleRule, CssRule, CssRuleList, Location},
    printer::Printer,
    error::PrinterError,
    declaration::DeclarationBlock,
    selector::Component,
    vendor_prefix::VendorPrefix,
};
use reqwest;
use std::convert::Infallible;
use cssparser;
use cssparser::_cssparser_internal_to_lowercase;

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

    /*
    let ui_dirs = vec!["pages", "templates", "components"];
    let mut css_combined_string = String::new();
    for dir in ui_dirs {
        let walker = WalkDir::new(dir).into_iter();
        for entry in walker.filter_entry(|e| is_css_file(e)) {
            if let Ok(css_file) = entry {
                let css_string = fs::read_to_string(css_file.path())
                .expect("Should have been able to read string from css file");
                css_combined_string.push_str(&css_string);
            }
        }
    }
    */
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

fn is_css_file(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.ends_with(".css"))
         .unwrap_or(false)
}

/// An @apply rule.
#[derive(Debug, Clone)]
struct ApplyRule {
  names: Vec<String>,
  loc: cssparser::SourceLocation,
}

#[derive(Debug, Clone)]
struct ApplyAtRuleParser;
impl<'i> AtRuleParser<'i> for ApplyAtRuleParser {
    type Prelude = Vec<String>;
    type Error = Infallible;
    type AtRule = ApplyRule;

    fn parse_prelude<'t>(
        &mut self,
        name: cssparser::CowRcStr<'i>,
        input: &mut cssparser::Parser<'i, 't>,
        _options: &ParserOptions<'_, 'i>,
    ) -> Result<Self::Prelude, cssparser::ParseError<'i, Self::Error>> {
        cssparser::match_ignore_ascii_case! {&*name,
            "apply" => {
                let mut names = Vec::new();
                loop {
                    if let Ok(name) = input.try_parse(|input| input.expect_ident_cloned()) {
                        names.push(name.as_ref().into());
                    } else {
                        break
                    }
                }
                Ok(names)
            },
            _ => Err(input.new_error(cssparser::BasicParseErrorKind::AtRuleInvalid(name)))
        }
    }

    fn rule_without_block(
        &mut self,
        prelude: Self::Prelude,
        start: &cssparser::ParserState,
        _options: &ParserOptions<'_, 'i>,
        _is_nested: bool,
    ) -> Result<Self::AtRule, ()> {
        let loc = start.source_location();
        Ok(ApplyRule { names: prelude, loc })
    }
}

struct ApplyVisitor<'a, 'i> {
    rules: &'a mut HashMap<String, DeclarationBlock<'i>>,
}
  
impl<'a, 'i> Visitor<'i, ApplyRule> for ApplyVisitor<'a, 'i> {
    type Error = Infallible;
  
    const TYPES: VisitTypes = VisitTypes::RULES;
  
    fn visit_rule(&mut self, rule: &mut CssRule<'i, ApplyRule>) -> Result<(), Self::Error> {
      match rule {
        CssRule::Style(rule) => {
          for selector in rule.selectors.0.iter() {
            if selector.len() != 1 {
                continue; // TODO
            }
            for component in selector.iter_raw_match_order() {
              match component {
                Component::Class(name) => {
                    self.rules.insert(name.0.to_string(), rule.declarations.clone());
                }
                _ => {}
              }
            }
          }
        }
        CssRule::Custom(ApplyRule { names, loc }) => {
            let mut declarations = DeclarationBlock::new();
            for name in names {
                let Some(applied) = self.rules.get(name) else {
                    continue;
                };
                declarations.important_declarations
                    .extend(applied.important_declarations.iter().cloned());
                declarations.declarations.extend(applied.declarations.iter().cloned());
            }
            *rule = CssRule::Style(StyleRule {
                selectors: Component::Nesting.into(),
                vendor_prefix: VendorPrefix::None,
                declarations,
                rules: CssRuleList(vec![]),
                loc: Location {
                    source_index: 0,
                    line: loc.line,
                    column: loc.column,
                },
            })
        }
        _ => {}
      }
  
      rule.visit_children(self)
    }
}

impl<'i, V: Visitor<'i, ApplyRule>> Visit<'i, ApplyRule, V> for ApplyRule {
    const CHILD_TYPES: VisitTypes = VisitTypes::empty();
  
    fn visit_children(&mut self, _: &mut V) -> Result<(), V::Error> {
      Ok(())
    }
}

impl ToCss for ApplyRule {
    fn to_css<W: std::fmt::Write>(&self, _dest: &mut Printer<W>) -> Result<(), PrinterError> {
        Ok(())
    }
}
