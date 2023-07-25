//use core::panic;
use std::{
    fs, convert::Infallible, 
    collections::{HashMap, HashSet, hash_map::RandomState}, 
    path::Path
};
use regex::Regex;
use cssparser::*;
use walkdir::WalkDir;
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

    for entry in WalkDir::new(stpl_templates_dir).into_iter()
            .filter_map(|e| e.ok()) {
        let f_name = entry.file_name().to_string_lossy();
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
            let stpl_file_name_stripped = entry.path().file_stem().unwrap().to_string_lossy();
            let stpl_file_path = format!(
                "{}/{}.stpl", 
                entry.path().parent().unwrap().to_string_lossy(),
                stpl_file_name_stripped
            );
            //dbg!(&stpl_file_path);
            if let Ok(stpl_string) = fs::read_to_string(&stpl_file_path) {
                let re = Regex::new("class\\s{0,1}=\\s{0,1}[\"\']([^\"\']*)[\"\']").unwrap();
                //let re = Regex::new("class\\s{0,1}=\\s{0,1}[\"\']([^\"\']*)[\"\']/gm").unwrap();
                for cap in re.captures_iter(&stpl_string) {
                    //dbg!(&cap.get(0));
                    //dbg!(&cap.get(1));
                    if let Some(whole_class_string) = cap.get(0) {
                        let whole_class_string = whole_class_string.as_str();
                        // Save class_list for later for the replacing stage
                        //classes_non_hashed.push(class_list.to_string());

                        // Get the capture group of the regex which removes the class=""
                        let separated_classes = cap.get(1).expect("Should have capture group of classes");
                        let mut class_list: Vec<String> = Vec::new();
                        // Now we just need to split the classes by the whitespace between them
                        for class in separated_classes.as_str().split_whitespace() {
                            style_rules.remove(class);
                            class_list.push(class.to_string());
                        }
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
            if let Ok(mut stpl_string) = fs::read_to_string(&stpl_file_path) {
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
                    stpl_string = stpl_string.replace(&matched_class.whole, &hashed_class_list);
                }
                // Then write stpl_string to new file in the ./template/output dir
                let stpl_file_stem = stpl_file_path.strip_prefix(&stpl_templates_dir).unwrap();
                let stpl_output_path = format!("{stpl_output_dir}{stpl_file_stem}");
                let mut stpl_file_name = String::from(stpl_file_name_stripped);
                stpl_file_name.push_str(".stpl");
                let stpl_parent_dir = stpl_output_path.strip_suffix(&stpl_file_name).unwrap();
                
                if !Path::new(stpl_parent_dir).is_dir() {
                    fs::create_dir_all(stpl_parent_dir).expect("Should be able to create assets directory if not there");
                }
                fs::write(stpl_output_path, stpl_string)
                    .expect("Should be able to write new stpl string to file in output dir");
            }
        }
    }
    fs::write(styles_file_path, css_minified).expect("Should be able to write minified css string to file");

    //panic!()
    
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