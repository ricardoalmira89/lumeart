//! `gomath` da los mismos bits que `Math` en goja (el motor de pxcli), con
//! los valores de `fixtures/gomath.json` (generados por
//! `scripts/vec-golden/gomath/`).

use std::path::Path;

use lume_domain::gomath;
use serde_json::Value;

fn f(hex: &Value) -> f64 {
    f64::from_bits(u64::from_str_radix(hex.as_str().unwrap(), 16).unwrap())
}

type Unary = (&'static str, fn(f64) -> f64);
type Binary = (&'static str, fn(f64, f64) -> f64);

fn same(a: f64, b: f64) -> bool {
    (a.is_nan() && b.is_nan()) || a.to_bits() == b.to_bits()
}

#[test]
fn every_function_matches_goja_bit_for_bit() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gomath.json");
    let data: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let one: [Unary; 8] = [
        ("sin", gomath::sin),
        ("cos", gomath::cos),
        ("tan", gomath::tan),
        ("atan", gomath::atan),
        ("asin", gomath::asin),
        ("acos", gomath::acos),
        ("exp", gomath::exp),
        ("log", gomath::log),
    ];
    let mut wrong = Vec::new();
    let mut total = 0;
    for (name, func) in one {
        for case in data[name].as_array().unwrap() {
            let (x, want) = (f(&case[0]), f(&case[1]));
            let got = func(x);
            total += 1;
            if !same(got, want) {
                wrong.push(format!("{name}({x:e}) = {got:e}, goja {want:e}"));
            }
        }
    }
    let two: [Binary; 3] = [
        ("atan2", gomath::atan2),
        ("pow", gomath::pow),
        ("hypot", |a, b| gomath::hypot(&[a, b])),
    ];
    for (name, func) in two {
        for case in data[name].as_array().unwrap() {
            let (a, b, want) = (f(&case[0]), f(&case[1]), f(&case[2]));
            let got = func(a, b);
            total += 1;
            if !same(got, want) {
                wrong.push(format!("{name}({a:e}, {b:e}) = {got:e}, goja {want:e}"));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "{} de {total} distintos:\n{}",
        wrong.len(),
        wrong
            .iter()
            .take(30)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
