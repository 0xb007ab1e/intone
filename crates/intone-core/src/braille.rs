//! Braille translation: render text to Unicode braille patterns (U+2800–U+28FF).
//!
//! This is **uncontracted (Grade 1) English braille** — a pure, deterministic mapping covering
//! letters, digits, space, and common punctuation, unit-tested against known cells with no C
//! dependency. It is the platform-agnostic translation seam: contracted (Grade 2) braille and
//! other languages (via liblouis) can later slot in behind [`to_braille`], and a back-end can
//! send the resulting cells to a physical display (e.g. via BrlAPI).

/// Empty braille cell (a braille space), U+2800.
const BLANK: char = '\u{2800}';
/// Number sign (dots 3-4-5-6): digits that follow are read as numbers.
const NUMBER_SIGN: char = '\u{283c}';
/// Capital sign (dot 6): the single letter that follows is uppercase.
const CAPITAL_SIGN: char = '\u{2820}';

/// Translate `text` to a string of Unicode braille-pattern characters (Grade 1).
///
/// Letters map to their English braille cells, digits are preceded by the number sign, a single
/// uppercase letter is preceded by the capital sign, spaces become blank cells, and a few common
/// punctuation marks are supported. Unsupported characters become a blank cell.
#[must_use]
pub fn to_braille(text: &str) -> String {
    let mut out = String::new();
    let mut number_mode = false;
    for ch in text.chars() {
        if ch.is_ascii_digit() {
            if !number_mode {
                out.push(NUMBER_SIGN);
                number_mode = true;
            }
            out.push(dots_for(digit_letter(ch)).map_or(BLANK, cell));
            continue;
        }
        number_mode = false;
        if ch == ' ' {
            out.push(BLANK);
            continue;
        }
        if ch.is_ascii_uppercase() {
            out.push(CAPITAL_SIGN);
        }
        out.push(dots_for(ch.to_ascii_lowercase()).map_or(BLANK, cell));
    }
    out
}

/// Build the braille cell for a set of raised dots (1–8) as a Unicode braille pattern.
fn cell(dots: &[u8]) -> char {
    let mask = dots
        .iter()
        .fold(0u32, |acc, &dot| acc | (1u32 << (dot - 1)));
    char::from_u32(0x2800 + mask).unwrap_or(BLANK)
}

/// The letter whose cell represents a digit (1→a … 9→i, 0→j).
fn digit_letter(digit: char) -> char {
    match digit {
        '1' => 'a',
        '2' => 'b',
        '3' => 'c',
        '4' => 'd',
        '5' => 'e',
        '6' => 'f',
        '7' => 'g',
        '8' => 'h',
        '9' => 'i',
        _ => 'j', // '0'
    }
}

/// The raised dots for a lowercase letter or supported punctuation mark, if any.
fn dots_for(ch: char) -> Option<&'static [u8]> {
    let dots: &[u8] = match ch {
        'a' => &[1],
        'b' => &[1, 2],
        'c' => &[1, 4],
        'd' => &[1, 4, 5],
        'e' => &[1, 5],
        'f' => &[1, 2, 4],
        'g' => &[1, 2, 4, 5],
        'h' => &[1, 2, 5],
        'i' => &[2, 4],
        'j' => &[2, 4, 5],
        'k' => &[1, 3],
        'l' => &[1, 2, 3],
        'm' => &[1, 3, 4],
        'n' => &[1, 3, 4, 5],
        'o' => &[1, 3, 5],
        'p' => &[1, 2, 3, 4],
        'q' => &[1, 2, 3, 4, 5],
        'r' => &[1, 2, 3, 5],
        's' => &[2, 3, 4],
        't' => &[2, 3, 4, 5],
        'u' => &[1, 3, 6],
        'v' => &[1, 2, 3, 6],
        'w' => &[2, 4, 5, 6],
        'x' => &[1, 3, 4, 6],
        'y' => &[1, 3, 4, 5, 6],
        'z' => &[1, 3, 5, 6],
        ',' => &[2],
        ';' => &[2, 3],
        ':' => &[2, 5],
        '.' => &[2, 5, 6],
        '!' => &[2, 3, 5],
        '?' => &[2, 3, 6],
        '\'' => &[3],
        '-' => &[3, 6],
        _ => return None,
    };
    Some(dots)
}

#[cfg(test)]
mod tests {
    use super::to_braille;

    #[test]
    fn translates_letters() {
        // h e l l o
        assert_eq!(
            to_braille("hello"),
            "\u{2813}\u{2811}\u{2807}\u{2807}\u{2815}"
        );
        assert_eq!(to_braille("abc"), "\u{2801}\u{2803}\u{2809}");
    }

    #[test]
    fn space_becomes_a_blank_cell() {
        assert_eq!(to_braille("a b"), "\u{2801}\u{2800}\u{2803}");
    }

    #[test]
    fn digits_get_a_number_sign() {
        // number-sign, d (=4), b (=2)
        assert_eq!(to_braille("42"), "\u{283c}\u{2819}\u{2803}");
        assert_eq!(to_braille("1"), "\u{283c}\u{2801}");
    }

    #[test]
    fn uppercase_gets_a_capital_sign() {
        assert_eq!(to_braille("A"), "\u{2820}\u{2801}");
    }

    #[test]
    fn punctuation_and_unknown() {
        assert_eq!(to_braille(","), "\u{2802}");
        // An unsupported character degrades to a blank cell rather than vanishing.
        assert_eq!(to_braille("~"), "\u{2800}");
        assert_eq!(to_braille(""), "");
    }
}
