use serde_json::Value;

pub fn extract_value_from_json(record: &Value, path: &[String]) -> Option<String> {
    let mut current = record;
    for segment in path {
        match current.get(segment) {
            Some(val) => {
                if let Value::Object(_) = val {
                    current = val;
                } else {
                    return match val {
                        Value::String(s) => Some(s.clone()),
                        Value::Number(n) => Some(n.to_string()),
                        Value::Bool(b) => Some(b.to_string()),
                        Value::Null => None,
                        _ => Some(val.to_string()),
                    };
                }
            }
            None => return None,
        }
    }
    None
}
