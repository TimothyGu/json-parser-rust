// SPDX-License-Identifier: MIT OR BlueOak-1.0.0

use std::collections::HashMap;
use std::fmt;
use std::vec::Vec;

#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.append(f)
    }
}

impl Value {
    fn append_str(s: &str, out: &mut impl fmt::Write) -> fmt::Result {
        out.write_char('"')?;
        for ch in s.chars() {
            match ch {
                '\x08' => out.write_str("\\b"),
                '\x0c' => out.write_str("\\f"),
                '\n' => out.write_str("\\n"),
                '\r' => out.write_str("\\r"),
                '\t' => out.write_str("\\t"),
                '"' => out.write_str("\\\""),
                '\\' => out.write_str("\\\\"),
                '\x00'..='\x19' => write!(out, "\\u{:04x}", ch as u8),
                _ => out.write_char(ch),
            }?
        }
        out.write_char('"')?;
        Ok(())
    }

    fn append(&self, out: &mut impl fmt::Write) -> fmt::Result {
        match self {
            Value::Null => out.write_str("null"),
            Value::Bool(b) => out.write_str(if *b { "true" } else { "false" }),
            Value::Number(n) if n.is_finite() => {
                // Ryū's rendering is closer to JavaScript's than Rust std (1 => "1", 1e-123 => "1e-123")
                let mut buf = ryu::Buffer::new();
                out.write_str(buf.format_finite(*n))
            }
            Value::Number(_) => out.write_str("null"), // match JavaScript
            Value::String(s) => Self::append_str(&s, out),
            Value::Object(o) => {
                out.write_char('{')?;
                let mut it = o.iter();
                if let Some((k, v)) = it.next() {
                    Self::append_str(&k, out)?;
                    out.write_char(':')?;
                    v.append(out)?;
                    for (k, v) in it {
                        out.write_char(',')?;
                        Self::append_str(&k, out)?;
                        out.write_char(':')?;
                        v.append(out)?;
                    }
                }
                out.write_char('}')
            }
            Value::Array(a) => {
                out.write_char('[')?;
                let mut it = a.iter();
                if let Some(v) = it.next() {
                    v.append(out)?;
                    for v in it {
                        out.write_char(',')?;
                        v.append(out)?;
                    }
                }
                out.write_char(']')
            }
        }
    }
}
