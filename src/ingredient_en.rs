use pest::Parser;

#[derive(Parser)]
#[grammar = "grammar_en.pest"]
struct IngreedientParserEn;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize)]
pub struct Span {
    pub from: usize,
    pub to: usize,
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize)]
pub enum Unit {
    Kilogram,
    Pound,
    Ounce,
    Cup,
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
    pub container_amount: Option<ValueWithSpan<Amount>>,
    pub container_unit: Option<ValueWithSpan<Unit>>,
    pub ingredient: Option<ValueWithSpan<String>>,
}

pub fn parse(str: &str) -> IngredientInfo {
    let pairs = IngreedientParserEn::parse(Rule::ingredient_and_amount_and_unit, str).unwrap();

    let mut amount: Option<ValueWithSpan<Amount>> = None;
    let mut unit: Option<ValueWithSpan<Unit>> = None;
    let mut container_amount: Option<ValueWithSpan<Amount>> = None;
    let mut container_unit: Option<ValueWithSpan<Unit>> = None;
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
            Rule::container_size => {
                let mut pairs = pair.into_inner();

                let amount_pair = pairs.next().unwrap();
                let amount_span = extract_span(amount_pair.as_span());
                container_amount = Some(ValueWithSpan {
                    value: parse_amount(amount_pair),
                    span: amount_span,
                });

                let unit_pair = pairs.next().unwrap();
                let unit_span = extract_span(unit_pair.as_span());
                container_unit = Some(ValueWithSpan {
                    value: parse_unit(unit_pair),
                    span: unit_span,
                });
            }
            Rule::ingredient | Rule::ingredient_alt => {
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
        container_amount,
        container_unit,
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
                Rule::integer_and_whitespace => {
                    let pair_ = pair.into_inner().next().unwrap();
                    pair = pairs.next().unwrap();
                    pair_.as_str().parse().unwrap()
                }
                Rule::integer => {
                    let integer = pair.as_str().parse().unwrap();
                    pair = pairs.next().unwrap();
                    integer
                }
                _ => 0,
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
        Rule::word_digit => {
            Constant::Fraction(super::dictionary_en::look_up_word_digit(pair.as_str()), 1)
        }
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
    super::dictionary_en::canonical_unit(pair.as_str())
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
}
