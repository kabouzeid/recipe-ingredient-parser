extern crate yaml_rust;
use yaml_rust::YamlLoader;

extern crate heck;
use heck::CamelCase;

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let content = fs::read_to_string("dictionaries/en.yml").unwrap();
    let dict = &YamlLoader::load_from_str(&content).unwrap()[0];

    let mut all_unit_exprs = Vec::new();
    let mut all_word_digit_exprs = Vec::new();

    // units
    {
        let mut dict_code = String::from(r#"use super::unit::Unit;"#);
        dict_code.push_str("\r\n");
        dict_code.push_str(r#"pub fn expr_to_unit(unit: &str) -> Option<Unit> { match unit.to_ascii_lowercase().as_str() {"#);
        dict_code.push_str("\r\n");

        for unit in dict["units"].as_hash().unwrap() {
            let unit_value_camel = unit.0.as_str().unwrap().to_camel_case();
            for unit_expr in unit.1.as_vec().unwrap() {
                let unit_expr_str = unit_expr.as_str().unwrap();
                dict_code.push_str(&format!(r#""{}" => Some(Unit::{}),"#, unit_expr_str, unit_value_camel));
                dict_code.push_str("\r\n");
                all_unit_exprs.push(String::from(unit_expr_str));
            }
        }

        dict_code.push_str(r#"&_ => None,"#);
        dict_code.push_str("\r\n");
        dict_code.push_str(r#"}}"#);

        fs::write(&Path::new(&out_dir).join("unit_dict_en.rs"), dict_code).unwrap();
    }

    // numbers
    {
        let mut dict_code = String::from(r#"pub fn expr_to_int(number: &str) -> Option<u32> { match number.to_ascii_lowercase().as_str() {"#);
        dict_code.push_str("\r\n");

        for number in dict["numbers"].as_hash().unwrap() {
            let number_expr = number.0.as_str().unwrap();
            let number_value = number.1.as_i64().unwrap();
            dict_code.push_str(&format!(r#""{}" => Some({}),"#, number_expr, number_value));
            dict_code.push_str("\r\n");
            all_word_digit_exprs.push(String::from(number_expr));
        }

        dict_code.push_str(r#"&_ => None,"#);
        dict_code.push_str("\r\n");
        dict_code.push_str(r#"}}"#);

        fs::write(&Path::new(&out_dir).join("number_dict_en.rs"), dict_code).unwrap();
    }

    // parser
    let grammar_path = Path::new(&out_dir).join("ingredient_grammar_en.pest");
    let parser_code = String::from(
        format!(
r#"
#[derive(Parser)]
#[grammar = "{}"]
struct IngredientParserEn;
"#, grammar_path.to_str().unwrap())
        );
    fs::write(Path::new(&out_dir).join("ingredient_parser_en.rs"), parser_code).unwrap();

    // grammar
    let mut grammar_code = fs::read_to_string("ingredient_grammar.pest").unwrap();
    grammar_code.push_str("\r\n");

    grammar_code.push_str("unit = { ");
    all_unit_exprs.sort_by(|a, b| b.len().cmp(&a.len()));
    for x in &mut all_unit_exprs { *x = format!(r#"^"{}""#, *x); }
    grammar_code.push_str(&all_unit_exprs.join(" | "));
    grammar_code.push_str("}");

    grammar_code.push_str("\r\n");
    grammar_code.push_str("\r\n");

    grammar_code.push_str("word_digit = { ");
    all_word_digit_exprs.sort_by(|a, b| b.len().cmp(&a.len()));
    for x in &mut all_word_digit_exprs { *x = format!(r#"^"{}""#, *x); }
    grammar_code.push_str(&all_word_digit_exprs.join(" | "));
    grammar_code.push_str(" }");

    grammar_code.push_str("\r\n");
    grammar_code.push_str("\r\n");

    let mut prepositions = Vec::new();
    for preposition in dict["prepositions"].as_vec().unwrap() {
        prepositions.push(String::from(preposition.as_str().unwrap()));
    }
    grammar_code.push_str("preposition = { ");
    prepositions.sort_by(|a, b| b.len().cmp(&a.len()));
    for x in &mut prepositions { *x = format!(r#"^"{}""#, *x); }
    grammar_code.push_str(&prepositions.join(" | "));
    grammar_code.push_str(" }");

    fs::write(grammar_path, grammar_code).unwrap();
    println!("cargo:rerun-if-changed=ingredient_grammar.pest");
    println!("cargo:rerun-if-changed=dictionaries/en.yml");
}
