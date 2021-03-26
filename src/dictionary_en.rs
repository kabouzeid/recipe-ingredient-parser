use std::collections::HashMap;

pub fn look_up_word_digit(str: &str) -> u32 {
    let word_digits: HashMap<&str, u32> = [
        ("a", 1),
        ("an", 1),
        ("one", 1),
        ("two", 2),
        ("three", 3),
    ].iter().cloned().collect();
    word_digits[str]
}

use super::ingredient_en::Unit;

pub fn canonical_unit(str: &str) -> Unit {
    let units: HashMap<&str, Unit> = [
        ("kg", Unit::Kilogram),
        ("kilogram", Unit::Kilogram),
        ("lb", Unit::Pound),
        ("pound", Unit::Pound),
        ("oz", Unit::Ounce),
        ("ounce", Unit::Ounce),
        ("ounces", Unit::Ounce),
        ("cup", Unit::Cup),
        ("cups", Unit::Cup),
    ].iter().cloned().collect();
    units[str]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_look_up_word_digit() {
        assert_eq!(look_up_word_digit("a"), 1);
        assert_eq!(look_up_word_digit("two"), 2);
    }

    #[test]
    fn test_canonical_unit() {
        assert_eq!(canonical_unit("kilogram"), Unit::Kilogram);
        assert_eq!(canonical_unit("lb"), Unit::Pound);
    }
}
