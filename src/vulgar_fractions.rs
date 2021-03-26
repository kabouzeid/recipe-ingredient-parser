use std::collections::HashMap;

pub fn look_up_vulgar_fraction(fraction: &str) -> (u32, u32) {
    vulgar_fractions()[fraction]
}

fn vulgar_fractions() -> HashMap<&'static str, (u32, u32)> {
    [
        ("\u{00BC}", (1,4)),
        ("\u{00BD}", (1,2)),
        ("\u{00BE}", (3,4)),
        ("\u{2150}", (1,7)),
        ("\u{2151}", (1,9)),
        ("\u{2152}", (1,10)),
        ("\u{2153}", (1,3)),
        ("\u{2154}", (2,3)),
        ("\u{2155}", (1,5)),
        ("\u{2156}", (2,5)),
        ("\u{2157}", (3,5)),
        ("\u{2158}", (4,5)),
        ("\u{2159}", (1,6)),
        ("\u{215A}", (5,6)),
        ("\u{215B}", (1,8)),
        ("\u{215C}", (3,8)),
        ("\u{215D}", (5,8)),
        ("\u{215E}", (7,8))
    ].iter().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_look_up_vulgar_fraction() {
        assert_eq!(look_up_vulgar_fraction("\u{215A}"), (5,6));
        assert_eq!(look_up_vulgar_fraction("\u{2150}"), (1,7));
    }
}
