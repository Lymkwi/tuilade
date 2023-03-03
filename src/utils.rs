//! Various utilities I use on top of `serde_json::Value`

use serde_json::Value;

fn try_number(val: &Value) -> Result<&serde_json::Number, String> {
    match val {
        Value::Number(f) => Ok(f),
        _ => Err(String::from("Invalid JSON Value type (expected number)")),
    }
}

pub(crate) fn try_f64(val: &Value) -> Result<f64, String> {
    let num = try_number(val)?;
    if num.is_f64() {
        Ok(num.as_f64().unwrap())
    } else if num.is_u64() {
        Ok(num.as_u64().unwrap() as f64)
    } else {
        Err(String::from("Invalid JSON number: Expected a f64"))
    }
}

pub(crate) fn try_u64(val: &Value) -> Result<u64, String> {
    let num = try_number(val)?;
    if num.is_u64() {
        Ok(num.as_u64().unwrap())
    } else {
        Err(String::from("Invalid JSON number: Expected a u64"))
    }
}

pub(crate) fn try_string(val: &Value) -> Result<&str, String> {
    match val {
        Value::String(st) => Ok(st),
        _ => Err(String::from("Invalid JSON Value type (expected string)")),
    }
}

pub(crate) fn try_vec(val: &Value) -> Result<&Vec<Value>, String> {
    match val {
        Value::Array(vec) => Ok(vec),
        _ => Err(String::from("Invalid JSON Value type (expected array)"))
    }
}
