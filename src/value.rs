// SPDX-License-Identifier: MIT OR BlueOak-1.0.0

use std::collections::HashMap;
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
