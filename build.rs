use std::{
    fs, convert::Infallible, collections::HashMap
};
use cssparser::*;
use walkdir::WalkDir;
use lightningcss::{
    error::PrinterError,
    printer::Printer,
    traits::{AtRuleParser, ToCss},
    selector::Component,
    declaration::DeclarationBlock,
    rules::{style::StyleRule, CssRule, CssRuleList, Location},
    //css_modules,
    stylesheet::{ParserOptions, MinifyOptions, PrinterOptions, StyleSheet, ParserFlags},
    visitor::{Visitor, Visit, VisitTypes},
    vendor_prefix::VendorPrefix,
    targets::{Browsers, Targets},
};

fn main() {
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

    for entry in WalkDir::new("./templates").into_iter()
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
                    //css_modules: Some(css_modules::Config::default()),
                    ..ParserOptions::default()
                }, 
                &mut ApplyAtRuleParser
            ).unwrap();
            
            /*
            let mut stylesheet = StyleSheet::parse(&css_string, ParserOptions {
                filename: f_name.to_string(),
                flags: ParserFlags::NESTING,
                //css_modules: Some(css_modules::Config::default()),
                ..ParserOptions::default()
            }).unwrap();
            */

            
            let mut style_rules: HashMap<String, DeclarationBlock<'_>> = HashMap::new();
            stylesheet.visit(&mut ApplyVisitor {
                rules: &mut style_rules,
            }).unwrap();
            

            stylesheet.minify(MinifyOptions { 
                targets, 
                ..MinifyOptions::default()
            }).unwrap();
            
            let res = stylesheet.to_css(PrinterOptions {
                //minify: true,
                targets,
                ..PrinterOptions::default()
            }).unwrap();
            css_minified.insert_str(0, &res.code);
        }
    }
    
    fs::write("./assets/styles.css", css_minified).expect("Should be able to write minified css string to file");
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