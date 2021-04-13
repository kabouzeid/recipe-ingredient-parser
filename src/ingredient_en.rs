use pest::Parser;
include!(concat!(env!("OUT_DIR"), "/ingredient_parser_en.rs"));

include!(concat!(env!("OUT_DIR"), "/number_dict_en.rs"));
include!(concat!(env!("OUT_DIR"), "/unit_dict_en.rs"));

use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub struct Span {
    pub from: usize,
    pub to: usize,
}

#[derive(Debug, PartialEq, Serialize)]
pub enum Constant {
    Fraction(u32, u32),
    Float(f64),
}

#[derive(Debug, PartialEq, Serialize)]
pub enum Amount {
    Range {
        value_from: Constant,
        value_to: Constant,
    },
    Constant {
        value: Constant,
    },
}

#[derive(Debug, Serialize)]
pub struct ValueWithSpan<T> {
    value: T,
    span: Span,
}

#[derive(Debug, Serialize)]
pub struct IngredientInfo {
    pub amount: Option<ValueWithSpan<Amount>>,
    pub unit: Option<ValueWithSpan<Unit>>,
    pub ingredient: Option<ValueWithSpan<String>>,
}

pub fn parse(str: &str) -> IngredientInfo {
    let pairs = IngredientParserEn::parse(Rule::main, str).unwrap();

    let mut amount: Option<ValueWithSpan<Amount>> = None;
    let mut unit: Option<ValueWithSpan<Unit>> = None;
    let mut ingredient: Option<ValueWithSpan<String>> = None;

    for pair in pairs {
        match pair.as_rule() {
            Rule::amount => {
                let span = extract_span(pair.as_span());
                amount = Some(ValueWithSpan {
                    value: parse_amount(pair),
                    span,
                });
            }
            Rule::unit => {
                let span = extract_span(pair.as_span());
                unit = Some(ValueWithSpan {
                    value: parse_unit(pair),
                    span,
                });
            }
            Rule::ingredient | Rule::ingredient_lookahead => {
                let span = extract_span(pair.as_span());
                ingredient = Some(ValueWithSpan {
                    value: pair.as_str().to_string(),
                    span,
                });
            }
            Rule::preposition => continue,
            Rule::EOI => continue,
            _ => unreachable!(),
        }
    }

    IngredientInfo {
        amount,
        unit,
        ingredient,
    }
}

fn extract_span(span: pest::Span) -> Span {
    Span {
        from: span.start(),
        to: span.end(),
    }
}

fn parse_amount(pair: pest::iterators::Pair<Rule>) -> Amount {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::constant => Amount::Constant {
            value: parse_constant(pair),
        },
        Rule::range => {
            let mut pairs = pair.into_inner();
            let value_from = parse_constant(pairs.next().unwrap());
            let value_to = parse_constant(pairs.next().unwrap());
            Amount::Range {
                value_from,
                value_to,
            }
        }
        _ => unreachable!(),
    }
}

fn parse_constant(pair: pest::iterators::Pair<Rule>) -> Constant {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::fraction => {
            let mut pairs = pair.into_inner();

            let mut pair = pairs.next().unwrap();
            let integer: u32 = match pair.as_rule() {
                Rule::integer => {
                    let integer = pair.as_str().parse().unwrap();
                    pair = pairs.next().unwrap();
                    integer
                }
                _ => 0, // integer is optional, so this happens for eg 1/2
            };

            match pair.as_rule() {
                Rule::vulgar_fraction => {
                    let (n, d) = parse_vulgar_fraction(pair);
                    Constant::Fraction(integer * d + n, d)
                }
                Rule::simple_fraction => {
                    let (n, d) = parse_simple_fraction(pair);
                    Constant::Fraction(integer * d + n, d)
                }
                _ => unreachable!(),
            }
        }
        Rule::float => Constant::Float(pair.as_str().replace(",", ".").parse().unwrap()),
        Rule::integer => Constant::Fraction(pair.as_str().parse().unwrap(), 1),
        Rule::word_digit => Constant::Fraction(expr_to_int(pair.as_str()).unwrap(), 1),
        _ => unreachable!(),
    }
}

fn parse_vulgar_fraction(pair: pest::iterators::Pair<Rule>) -> (u32, u32) {
    super::vulgar_fractions::look_up_vulgar_fraction(pair.as_str())
}

fn parse_simple_fraction(pair: pest::iterators::Pair<Rule>) -> (u32, u32) {
    let mut pairs = pair.into_inner();
    let n = pairs.next().unwrap().as_str().parse().unwrap();
    let d = pairs.next().unwrap().as_str().parse().unwrap();
    return (n, d);
}

fn parse_unit(pair: pest::iterators::Pair<Rule>) -> Unit {
    expr_to_unit(pair.as_str()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        {
            let info = parse("1 1/2 kg potatoes");

            let amount = info.amount.unwrap();
            assert_eq!(
                amount.value,
                Amount::Constant {
                    value: Constant::Fraction(3, 2)
                }
            );
            assert_eq!(amount.span.from, 0);
            assert_eq!(amount.span.to, 5);

            let unit = info.unit.unwrap();
            assert_eq!(unit.value, Unit::Kilogram);
            assert_eq!(unit.span.from, 6);
            assert_eq!(unit.span.to, 8);
        }

        {
            let info = parse("2-3 lb potatoes");

            let amount = info.amount.unwrap();
            assert_eq!(
                amount.value,
                Amount::Range {
                    value_from: Constant::Fraction(2, 1),
                    value_to: Constant::Fraction(3, 1)
                }
            );
            assert_eq!(amount.span.from, 0);
            assert_eq!(amount.span.to, 3);

            let unit = info.unit.unwrap();
            assert_eq!(unit.value, Unit::Pound);
            assert_eq!(unit.span.from, 4);
            assert_eq!(unit.span.to, 6);
        }

        {
            let info = parse("1½ cups flour");

            let amount = info.amount.unwrap();
            assert_eq!(
                amount.value,
                Amount::Constant {
                    value: Constant::Fraction(3, 2)
                }
            );
            assert_eq!(amount.span.from, 0);
            assert_eq!(amount.span.to, 3); // because ½ takes two bytes

            let unit = info.unit.unwrap();
            assert_eq!(unit.value, Unit::Cup);
            assert_eq!(unit.span.from, 4);
            assert_eq!(unit.span.to, 8);
        }

        {
            let info = parse("400 ml milk");

            let amount = info.amount.unwrap();
            assert_eq!(
                amount.value,
                Amount::Constant {
                    value: Constant::Fraction(400, 1)
                }
            );
            assert_eq!(amount.span.from, 0);
            assert_eq!(amount.span.to, 3); // because ½ takes two bytes

            let unit = info.unit.unwrap();
            assert_eq!(unit.value, Unit::Milliliter);
            assert_eq!(unit.span.from, 4);
            assert_eq!(unit.span.to, 6);
        }
    }

    #[test]
    fn test_reverse_format() {
        {
            let info = parse("Flour 1 kg");

            let amount = info.amount.unwrap();
            assert_eq!(
                amount.value,
                Amount::Constant {
                    value: Constant::Fraction(1, 1)
                }
            );
            assert_eq!(amount.span.from, 6);
            assert_eq!(amount.span.to, 7); // because ½ takes two bytes

            let unit = info.unit.unwrap();
            assert_eq!(unit.value, Unit::Kilogram);
            assert_eq!(unit.span.from, 8);
            assert_eq!(unit.span.to, 10);
        }

        {
            // (2 kg) is not recognized as container size in reverse
            let info = parse("Flour (2 kg) 1 kg");

            let amount = info.amount.unwrap();
            assert_eq!(
                amount.value,
                Amount::Constant {
                    value: Constant::Fraction(1, 1)
                }
            );
            assert_eq!(amount.span.from, 13);
            assert_eq!(amount.span.to, 14); // because ½ takes two bytes

            let unit = info.unit.unwrap();
            assert_eq!(unit.value, Unit::Kilogram);
            assert_eq!(unit.span.from, 15);
            assert_eq!(unit.span.to, 17);
        }
    }

    #[test]
    fn test_unit_case_insensitive() {
        {
            let info = parse("1 1/2 KG potatoes");

            let amount = info.amount.unwrap();
            assert_eq!(
                amount.value,
                Amount::Constant {
                    value: Constant::Fraction(3, 2)
                }
            );
            assert_eq!(amount.span.from, 0);
            assert_eq!(amount.span.to, 5);

            let unit = info.unit.unwrap();
            assert_eq!(unit.value, Unit::Kilogram);
            assert_eq!(unit.span.from, 6);
            assert_eq!(unit.span.to, 8);
        }
    }

    #[test]
    fn test_space_between_unit_and_ingredient() {
        // not "1 L ettuce"
        let info = parse("1 lettuce");
        assert!(info.amount.is_some());
        assert!(info.unit.is_none());

        // not "8 t omatoes"
        let info = parse("8 tomatoes");
        assert!(info.amount.is_some());
        assert!(info.unit.is_none());

        // not "olive oi L"
        let info = parse("olive oil");
        assert!(info.amount.is_none());
        assert!(info.unit.is_none());
    }

    #[test]
    fn test_simple_fractions() {
        let value = Amount::Constant {
            value: Constant::Fraction(2, 3),
        };

        assert_eq!(parse("2/3").amount.unwrap().value, value);
        assert_eq!(parse("2 /3").amount.unwrap().value, value);
        assert_eq!(parse("2/ 3").amount.unwrap().value, value);
        assert_eq!(parse("2 / 3").amount.unwrap().value, value);
    }

    #[test]
    fn test_compound_fractions() {
        let value = Amount::Constant {
            value: Constant::Fraction(5, 3),
        };

        assert_eq!(parse("1 2/3").amount.unwrap().value, value);
        assert_eq!(parse("1 2 /3").amount.unwrap().value, value);
        assert_eq!(parse("1 2/ 3").amount.unwrap().value, value);
        assert_eq!(parse("1 2 / 3").amount.unwrap().value, value);
    }

    #[test]
    fn test_ingredient_only() {
        // not "1 L ettuce"
        let info = parse("lettuce");
        assert!(info.amount.is_none());
        assert!(info.unit.is_none());
    }

    #[test]
    fn test_space_between_word_digit_and_unit() {
        // not "a L milk"
        let info = parse("Al milk");
        assert!(info.unit.is_none());
        assert!(info.amount.is_none());
    }

    #[test]
    fn test_amount_only() {
        let info = parse("1");
        assert!(info.amount.is_some());

        let info = parse("1/2 - 2/3");
        assert!(info.amount.is_some());

        let info = parse(".3");
        assert!(info.amount.is_some());

        let info = parse("one");
        assert!(info.amount.is_some());
    }

    #[test]
    fn test_unit_only() {
        let info = parse("kg");
        assert_eq!(info.unit.unwrap().value, Unit::Kilogram);

        let info = parse("handful");
        assert_eq!(info.unit.unwrap().value, Unit::Handful);
    }

    #[test]
    fn test_quantity_only() {
        let info = parse("1 kg");
        println!("INFO: {:?}", info);
        assert!(info.amount.is_some());
        assert!(info.unit.is_some());

        let info = parse(" 2 g ");
        assert!(info.amount.is_some());
        assert!(info.unit.is_some());
    }

    #[test]
    fn test_space_in_unit() {
        let info = parse("salt & pepper to taste");
        assert_eq!(info.unit.unwrap().value, Unit::ToTaste);
    }

    #[test]
    fn test_expr_to_int() {
        assert_eq!(expr_to_int("a").unwrap(), 1);
        assert_eq!(expr_to_int("two").unwrap(), 2);
    }

    #[test]
    fn test_expr_to_unit() {
        assert_eq!(expr_to_unit("kilogram").unwrap(), Unit::Kilogram);
        assert_eq!(expr_to_unit("lb").unwrap(), Unit::Pound);
        // assert_eq!(expr_to_unit("t").unwrap(), Unit::Teaspoon);
        // assert_eq!(expr_to_unit("T").unwrap(), Unit::Tablespoon);
        assert_eq!(expr_to_unit("ml.").unwrap(), Unit::Milliliter);
    }
}
