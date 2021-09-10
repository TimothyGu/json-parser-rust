// SPDX-License-Identifier: MIT OR BlueOak-1.0.0

use std::env;

fn main() {
    let json_text = env::args().nth(1).expect("input not found");
    let v = json::parse(&json_text).expect("failed to parse JSON input");
    println!("{}", v);
    println!("{}", v.clone());
}
