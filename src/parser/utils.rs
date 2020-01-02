use pom::parser::*;
use std::{
    iter::FromIterator,
    str::{
        self,
        FromStr,
    },
};

pub fn to_chars(input: &str) -> Vec<char> {
    input.chars().collect()
}

pub fn bytes_to_chars(input: &[u8]) -> Vec<char> {
    let input_str =
        String::from_utf8(input.to_owned()).expect("must be valid utf8");
    to_chars(&input_str)
}

pub(super) fn alpha_or_underscore(ch: char) -> bool {
    pom::char_class::alpha(ch as u8) || underscore(ch)
}

pub(super) fn alphanum_or_underscore(ch: char) -> bool {
    pom::char_class::alphanum(ch as u8) || underscore(ch)
}

pub(super) fn underscore(ch: char) -> bool {
    ch == '_'
}

/// any whitespace character
#[allow(unused)]
pub fn space<'a>() -> Parser<'a, char, ()> {
    one_of(" \t\r\n").repeat(0..).discard()
}

/// a number including decimal
pub(crate) fn number<'a>() -> Parser<'a, char, f64> {
    let integer =
        one_of("123456789") - one_of("0123456789").repeat(0..) | sym('0');
    let frac = sym('.') + one_of("0123456789").repeat(1..);
    let exp =
        one_of("eE") + one_of("+-").opt() + one_of("0123456789").repeat(1..);
    let number = sym('-').opt() + integer + frac.opt() + exp.opt();
    number
        .collect()
        .map(String::from_iter)
        .convert(|s| f64::from_str(&s))
}

pub(crate) fn integer<'a>() -> Parser<'a, char, i64> {
    let int = one_of("123456789") - one_of("0123456789").repeat(0..) | sym('0');
    int.collect()
        .map(String::from_iter)
        .convert(|s| i64::from_str(&s))
}

/// quoted string literal
pub(crate) fn quoted_string<'a>() -> Parser<'a, char, String> {
    let special_char = sym('\\')
        | sym('/')
        | sym('"')
        | sym('b').map(|_| '\x08')
        | sym('f').map(|_| '\x0C')
        | sym('n').map(|_| '\n')
        | sym('r').map(|_| '\r')
        | sym('t').map(|_| '\t');
    let escape_sequence = sym('\\') * special_char;
    let char_string = (none_of(r#"\""#) | escape_sequence)
        .repeat(1..)
        .map(String::from_iter);
    let string = sym('"') * char_string.repeat(0..) - sym('"');
    string.map(|strings| strings.concat())
}

/// 'string' single quoted string
pub(crate) fn single_quoted_string<'a>() -> Parser<'a, char, String> {
    let special_char = sym('\\')
        | sym('/')
        | sym('\'')
        | sym('b').map(|_| '\x08')
        | sym('f').map(|_| '\x0C')
        | sym('n').map(|_| '\n')
        | sym('r').map(|_| '\r')
        | sym('t').map(|_| '\t');
    let escape_sequence = sym('\\') * special_char;
    let char_string = (none_of(r#"\'"#) | escape_sequence)
        .repeat(1..)
        .map(String::from_iter);
    let string = sym('\'') * char_string.repeat(0..) - sym('\'');
    string.map(|strings| strings.concat())
}

/// `string` backquoted string since the browser don't replace this with percent encoding
pub(crate) fn back_quoted_string<'a>() -> Parser<'a, char, String> {
    let special_char = sym('\\')
        | sym('/')
        | sym('\'')
        | sym('b').map(|_| '\x08')
        | sym('f').map(|_| '\x0C')
        | sym('n').map(|_| '\n')
        | sym('r').map(|_| '\r')
        | sym('t').map(|_| '\t');
    let escape_sequence = sym('\\') * special_char;
    let char_string = (none_of(r#"\`"#) | escape_sequence)
        .repeat(1..)
        .map(String::from_iter);
    let string = sym('`') * char_string.repeat(0..) - sym('`');
    string.map(|strings| strings.concat())
}

/// string with no quote
pub(crate) fn string<'a>() -> Parser<'a, char, String> {
    let char_string = none_of("=&()").repeat(1..).map(String::from_iter);
    let string = char_string.repeat(0..);
    string.map(|strings| strings.concat())
}
