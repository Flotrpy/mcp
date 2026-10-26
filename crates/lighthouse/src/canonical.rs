//! Canonical JSON and hashing, so a tool definition hashes the same no matter
//! how a server orders its keys.

use serde_json::Value;
use sha2::{Digest, Sha256};

/// Recursively sort object keys. Arrays keep their order (it is meaningful,
/// e.g. `required` or `enum`).
pub fn canonicalize(v: &Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            let mut out = serde_json::Map::new();
            for k in keys {
                out.insert(k.clone(), canonicalize(&map[k]));
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(canonicalize).collect()),
        other => other.clone(),
    }
}

/// Compact canonical serialisation. Keys are emitted in sorted order
/// explicitly, so this does not depend on serde_json's map implementation.
pub fn to_canonical_string(v: &Value) -> String {
    fn write(v: &Value, out: &mut String) {
        match v {
            Value::Object(map) => {
                let mut keys: Vec<_> = map.keys().collect();
                keys.sort();
                out.push('{');
                for (i, k) in keys.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    out.push_str(&Value::String((*k).clone()).to_string());
                    out.push(':');
                    write(&map[*k], out);
                }
                out.push('}');
            }
            Value::Array(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write(item, out);
                }
                out.push(']');
            }
            other => out.push_str(&other.to_string()),
        }
    }
    let mut out = String::new();
    write(v, &mut out);
    out
}

/// `sha256:` + lowercase hex digest of the canonical form.
pub fn hash(v: &Value) -> String {
    let digest = Sha256::digest(to_canonical_string(v).as_bytes());
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    format!("sha256:{hex}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn key_order_does_not_change_the_hash() {
        let a = json!({"b": 1, "a": {"y": [3, 1], "x": null}});
        let b = json!({"a": {"x": null, "y": [3, 1]}, "b": 1});
        assert_eq!(hash(&a), hash(&b));
        assert_eq!(to_canonical_string(&a), r#"{"a":{"x":null,"y":[3,1]},"b":1}"#);
    }

    #[test]
    fn array_order_does_change_the_hash() {
        assert_ne!(hash(&json!([1, 2])), hash(&json!([2, 1])));
    }

    #[test]
    fn hash_is_prefixed_hex() {
        let h = hash(&json!({}));
        assert!(h.starts_with("sha256:"));
        assert_eq!(h.len(), 7 + 64);
        assert_eq!(canonicalize(&json!({"z": 1, "a": 2})).as_object().unwrap().keys().next().unwrap(), "a");
    }
}
