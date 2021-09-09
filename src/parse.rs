// SPDX-License-Identifier: MIT OR BlueOak-1.0.0

use crate::unicode;
use crate::value::Value;
use std::collections::HashMap;
use std::option::Option;
use std::str::Chars;

fn is_ws(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\n' || c == '\r'
}

fn peek(chars: &Chars) -> Option<char> {
    chars.clone().next()
}

fn next_if(chars: &mut Chars, func: impl FnOnce(char) -> bool) -> Option<char> {
    match peek(chars) {
        Some(ch) if func(ch) => Some(chars.next().unwrap()),
        _ => None,
    }
}

fn skip_while(chars: &mut Chars, func: impl Fn(char) -> bool) {
    while next_if(chars, |c| func(c)).is_some() {}
}

fn consume_object_entry(chars: &mut Chars, out: &mut HashMap<String, Value>) -> Option<()> {
    let key = parse_string(chars)?;
    skip_while(chars, is_ws);
    if chars.next()? != ':' {
        return None;
    }
    skip_while(chars, is_ws);
    let val = parse_value(chars)?;
    out.insert(key, val);
    Some(())
}

fn parse_object(input: &mut Chars) -> Option<HashMap<String, Value>> {
    let mut chars = input.clone();
    if chars.next()? != '{' {
        return None;
    }
    skip_while(&mut chars, is_ws);

    let mut out = HashMap::new();
    if peek(&chars)? == '}' {
        chars.next();
        *input = chars;
        return Some(out);
    }
    consume_object_entry(&mut chars, &mut out)?;
    skip_while(&mut chars, is_ws);

    loop {
        match chars.next()? {
            '}' => {
                *input = chars;
                return Some(out);
            }
            ',' => (),
            _ => return None,
        }
        skip_while(&mut chars, is_ws);

        consume_object_entry(&mut chars, &mut out)?;
        skip_while(&mut chars, is_ws);
    }
}

fn parse_array(chars: &mut Chars) -> Option<Vec<Value>> {
    if chars.next()? != '[' {
        return None;
    }
    skip_while(chars, is_ws);
    let mut out = Vec::new();

    if peek(chars)? == ']' {
        chars.next();
        return Some(out);
    }

    out.push(parse_value(chars)?);
    skip_while(chars, is_ws);

    loop {
        match chars.next()? {
            ']' => return Some(out),
            ',' => (),
            _ => return None,
        }
        skip_while(chars, is_ws);
        out.push(parse_value(chars)?);
        skip_while(chars, is_ws);
    }
}

fn parse_four_hex(chars: &mut Chars) -> Option<u16> {
    let mut tmp = String::new();
    tmp.push(chars.next()?);
    tmp.push(chars.next()?);
    tmp.push(chars.next()?);
    tmp.push(chars.next()?);
    u16::from_str_radix(&tmp, 16).ok()
}

fn parse_string(chars: &mut Chars) -> Option<String> {
    if chars.next()? != '"' {
        return None;
    }
    let mut out = String::new();
    let mut pending = None;

    macro_rules! flush_pending {
        () => {
            if pending.is_some() {
                out.push(char::REPLACEMENT_CHARACTER);
                pending = None;
                let _ = pending; // silence #[warn(unused_assignments)]
            }
        };
    }
    // Push a single char to out.
    macro_rules! push {
        ($ch:expr) => {{
            flush_pending!();
            out.push($ch);
        }};
    }

    loop {
        match chars.next()? {
            '"' => {
                flush_pending!();
                return Some(out);
            }
            '\\' => match chars.next()? {
                '"' => push!('"'),
                '\\' => push!('\\'),
                '/' => push!('/'),
                'b' => push!('\x08'),
                'f' => push!('\x0c'),
                'n' => push!('\n'),
                'r' => push!('\r'),
                't' => push!('\t'),
                'u' => {
                    let cu = parse_four_hex(chars)?;
                    if let Some(ch) = char::from_u32(cu as u32) {
                        push!(ch);
                    } else if unicode::is_lead_surrogate(cu) {
                        flush_pending!();
                        pending = Some(cu);
                    } else {
                        assert!(unicode::is_trail_surrogate(cu));
                        if let Some(lcu) = pending {
                            out.push(unicode::compose_surrogates(lcu, cu));
                            pending = None;
                        } else {
                            out.push(char::REPLACEMENT_CHARACTER);
                        }
                    }
                }
                _ => return None,
            },
            '\x00'..='\x19' => return None,
            c => push!(c),
        }
    }
}

fn parse_number(input: &mut Chars) -> Option<f64> {
    let mut chars = input.clone();
    if peek(&chars)? == '-' {
        chars.next();
    }

    // Consume integer part.
    match peek(&chars)? {
        '0' => {
            chars.next();
        }
        '1'..='9' => skip_while(&mut chars, |c| c.is_ascii_digit()),
        _ => return None,
    }

    // Consume fractional part.
    if peek(&chars) == Some('.') {
        let mut fchars = chars.clone();
        fchars.next();
        if fchars.next().filter(|c| c.is_ascii_digit()).is_some() {
            skip_while(&mut fchars, |c| c.is_ascii_digit());
            chars = fchars;
        }
    }

    // Consume exponential part.
    if peek(&chars).filter(|&c| c == 'e' || c == 'E').is_some() {
        let mut echars = chars.clone();
        echars.next();
        if peek(&echars).filter(|&c| c == '+' || c == '-').is_some() {
            echars.next();
        }
        if echars.next().filter(|&c| c.is_ascii_digit()).is_some() {
            skip_while(&mut echars, |c| c.is_ascii_digit());
            chars = echars;
        }
    }

    let consumed_bytes = input.as_str().len() - chars.as_str().len();
    let num_str = &input.as_str()[..consumed_bytes];
    *input = chars;
    Some(num_str.parse().unwrap())
}

fn parse_keyword(input: &mut Chars, kw: &str, expected: Value) -> Option<Value> {
    let mut chars = input.clone();
    for c in kw.chars() {
        if chars.next()? != c {
            return None;
        }
    }
    *input = chars;
    Some(expected)
}

fn parse_value(chars: &mut Chars) -> Option<Value> {
    match peek(chars)? {
        '{' => parse_object(chars).map(Value::Object),
        '[' => parse_array(chars).map(Value::Array),
        '"' => parse_string(chars).map(Value::String),
        '-' | '0'..='9' => parse_number(chars).map(Value::Number),
        'f' => parse_keyword(chars, "false", Value::Bool(false)),
        'n' => parse_keyword(chars, "null", Value::Null),
        't' => parse_keyword(chars, "true", Value::Bool(true)),
        _ => None,
    }
}

pub fn parse(s: &str) -> Option<Value> {
    let mut chars = s.chars();
    skip_while(&mut chars, is_ws);

    let out = parse_value(&mut chars)?;
    skip_while(&mut chars, is_ws);

    if peek(&chars).is_none() {
        Some(out)
    } else {
        None
    }
}
