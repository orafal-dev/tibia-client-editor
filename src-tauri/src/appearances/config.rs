use std::collections::HashMap;
use std::path::Path;

use serde_json::Value;
use toml::Value as TomlValue;

use super::AppearancesError;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AppearanceEdit {
    pub id: String,
    pub fields: HashMap<String, Value>,
}

pub fn edits_from_inputs(edits: Vec<AppearanceEdit>) -> Vec<AppearanceEdit> {
    edits
}

pub fn load_edits(config_path: &Path) -> Result<Vec<AppearanceEdit>, AppearancesError> {
    let content = std::fs::read_to_string(config_path)
        .map_err(|e| AppearancesError::Config(e.to_string()))?;
    let parsed: TomlValue =
        toml::from_str(&content).map_err(|e| AppearancesError::Config(e.to_string()))?;

    let Some(TomlValue::Array(edits)) = parsed.get("edit") else {
        return Ok(Vec::new());
    };

    let mut result = Vec::new();
    for item in edits {
        let Some(table) = item.as_table() else {
            continue;
        };
        let Some(id) = table.get("id").and_then(|v| v.as_str()) else {
            continue;
        };
        let mut fields = HashMap::new();
        for (key, value) in table {
            if key == "id" {
                continue;
            }
            fields.insert(key.clone(), toml_value_to_json(value));
        }
        result.push(AppearanceEdit {
            id: id.to_string(),
            fields,
        });
    }
    Ok(result)
}

fn toml_value_to_json(value: &TomlValue) -> Value {
    match value {
        TomlValue::String(s) => Value::String(s.clone()),
        TomlValue::Integer(i) => Value::Number((*i).into()),
        TomlValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        TomlValue::Boolean(b) => Value::Bool(*b),
        TomlValue::Array(arr) => Value::Array(arr.iter().map(toml_value_to_json).collect()),
        TomlValue::Table(table) => {
            if table.is_empty() {
                Value::Object(serde_json::Map::new())
            } else {
                let mut map = serde_json::Map::new();
                for (k, v) in table {
                    map.insert(k.clone(), toml_value_to_json(v));
                }
                Value::Object(map)
            }
        }
        TomlValue::Datetime(dt) => Value::String(dt.to_string()),
    }
}
