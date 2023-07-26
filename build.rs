//use core::panic;
use std::{
    fs, convert::Infallible, 
    collections::{HashMap, HashSet, hash_map::RandomState}, 
    path::Path
};
use regex::Regex;
use cssparser::*;
use walkdir::{WalkDir, DirEntry};
use lightningcss::{
    error::PrinterError,
    printer::Printer,
    traits::{AtRuleParser, ToCss},
    selector::Component,
    declaration::DeclarationBlock,
    rules::{style::StyleRule, CssRule, CssRuleList, Location},
    css_modules,
    stylesheet::{ParserOptions, MinifyOptions, PrinterOptions, StyleSheet, ParserFlags},
    visitor::{Visitor, Visit, VisitTypes},
    vendor_prefix::VendorPrefix,
    targets::{Browsers, Targets},
};

fn main() {
    let stpl_templates_dir = "./templates";
    let stpl_output_dir = "./templates/output";
    let assets_path = "./assets";
    let styles_file_name = "styles.css";
    let styles_file_path = format!("{assets_path}/{styles_file_name}");
    if !Path::new(assets_path).is_dir() {
        fs::create_dir_all(assets_path).expect("Should be able to create assets directory if not there");
    }
    let mut css_minified = String::new();
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

    let walker = WalkDir::new(stpl_templates_dir).into_iter();
    for entry in walker.filter_entry(|e| !is_dir(e, stpl_output_dir))
        .filter_map(|e| e.ok()) {
        let f_name = entry.file_name().to_string_lossy();
        let entry_path = entry.path();
        let file_path = entry_path.to_string_lossy();
        dbg!(&file_path);
        if f_name.ends_with(".css") {
            let css_string = fs::read_to_string(entry.path())
                .expect("Should have been able to read string from css file");

            let mut stylesheet = StyleSheet::parse_with(
                &css_string, 
                ParserOptions {
                    filename: f_name.to_string(),
                    flags: ParserFlags::NESTING,
                    css_modules: Some(css_modules::Config::default()),
                    ..ParserOptions::default()
                }, 
                &mut ApplyAtRuleParser
            ).unwrap();

            let mut style_rules: HashMap<String, DeclarationBlock<'_>> = HashMap::new();
            stylesheet.visit(&mut ApplyVisitor {
                rules: &mut style_rules,
            }).unwrap();

            let mut matched_classes: Vec<MatchedClassString> = Vec::new();
            
            // Removes classes from style_rules hashmap that aren't in the stpl files
            let stpl_file = file_path.replace(".css", ".stpl");
            if let Ok(stpl_string) = fs::read_to_string(&stpl_file) {
                let re = Regex::new("class\\s{0,1}=\\s{0,1}[\"\']([^\"\']*)[\"\']").unwrap();
                for cap in re.captures_iter(&stpl_string) {
                    if let Some(whole_class_string) = cap.get(0) {
                        let whole_class_string = whole_class_string.as_str();
                        
                        // Get the capture group of the regex which removes the class=""
                        let separated_classes = cap.get(1).expect("Should have capture group of classes");
                        let mut class_list: Vec<String> = Vec::new();
                        // Now we just need to split the classes by the whitespace between them
                        for class in separated_classes.as_str().split_whitespace() {
                            style_rules.remove(class);
                            class_list.push(class.to_string());
                        }
                        // Save class_list for later for the replacing stage
                        matched_classes.push(
                            MatchedClassString { 
                                whole: String::from(whole_class_string), 
                                list: class_list 
                            }
                        );
                    }
                }
            };
            
            
            // Convert remaining symbols in style_rules hashmap to a hashset put in minify options
            let mut unused_symbols: HashSet<String, RandomState> = HashSet::new();
            for symbol in style_rules.into_keys() {
                unused_symbols.insert(symbol);
            }

            stylesheet.minify(MinifyOptions { 
                targets,
                unused_symbols,
            }).unwrap();
            
            let res = stylesheet.to_css(PrinterOptions {
                //minify: true,
                targets,
                ..PrinterOptions::default()
            }).unwrap();
            css_minified.insert_str(0, &res.code);

            // Break down saved classes from stpl file, find their replacements, generate new class strings, and replace 
            if let Ok(mut content) = fs::read_to_string(&stpl_file) {
                let exported_css_classes = res.exports.unwrap();
                for matched_class in matched_classes {
                    let mut hashed_class_list = String::from("class=\"");
                    for class in matched_class.list {
                        if let Some(hashed_class) = exported_css_classes.get(&class) {
                            hashed_class_list.push_str(&hashed_class.name);
                            hashed_class_list.push(' ');
                        }
                    }
                    hashed_class_list.push('"');
                    content = content.replace(&matched_class.whole, &hashed_class_list);
                }
                write_file_to_new_dir(content, entry_path, stpl_file, &stpl_templates_dir, &stpl_output_dir);
            }
        }
        if f_name.ends_with(".stpl") {
            let css_file = file_path.replace(".stpl", ".css");
            let css_path = Path::new(&css_file);
            if !css_path.exists() {
                let content = fs::read_to_string(entry_path)
                    .expect("Should be able to read stpl file to string");
                write_file_to_new_dir(content, entry_path, file_path.to_string(), &stpl_templates_dir, &stpl_output_dir);
            } 

        }
    }
    fs::write(styles_file_path, css_minified).expect("Should be able to write minified css string to file");
    //panic!()
    
}

fn is_dir(entry: &DirEntry, dir: &str) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with(dir))
         .unwrap_or(false)
}

fn write_file_to_new_dir(content: String, entry_path: &Path, file_path: String, old_dir: &str, new_dir: &str) {
    
    let stem = file_path.strip_prefix(&old_dir).unwrap();
    let new_path = format!("{new_dir}/{stem}");

    let parent_stem = entry_path.parent().unwrap()
        .strip_prefix(&old_dir).unwrap().to_string_lossy();


    let new_parent = format!("{new_dir}/{parent_stem}");
    
    if !Path::new(&new_parent).is_dir() {
        fs::create_dir_all(new_parent).expect("Should be able to create assets directory if not there");
    }
    fs::write(new_path, content).expect("Should be able to write new string to file in output dir");
}

struct MatchedClassString {
    whole: String,
    list: Vec<String>
}

/// An @apply rule.
#[derive(Debug, Clone)]
struct ApplyRule {
  names: Vec<String>,
  loc: SourceLocation,
}

#[derive(Debug, Clone)]
struct ApplyAtRuleParser;
impl<'i> AtRuleParser<'i> for ApplyAtRuleParser {
    type Prelude = Vec<String>;
    type Error = Infallible;
    type AtRule = ApplyRule;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        _options: &ParserOptions<'_, 'i>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        match_ignore_ascii_case! {&*name,
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
            _ => Err(input.new_error(BasicParseErrorKind::AtRuleInvalid(name)))
        }
    }

    fn rule_without_block(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
        _options: &ParserOptions<'_, 'i>,
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