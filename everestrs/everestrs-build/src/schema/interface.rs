use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Interface {
    pub description: String,
    #[serde(default)]
    pub cmds: BTreeMap<String, Command>,
    #[serde(default)]
    pub vars: BTreeMap<String, Variable>,
    // The errors interface is currently just a hull.
    #[serde(default)]
    pub errors: Vec<ErrorEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub description: String,
    #[serde(default)]
    pub arguments: BTreeMap<String, Variable>,
    pub result: Option<Variable>,
}

#[derive(Debug, Serialize)]
pub struct Variable {
    pub description: Option<String>,
    pub arg: Argument,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Argument {
    Single(Type),
    Multiple(Vec<Type>),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NumberOptions {
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub default: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IntegerOptions {
    pub minimum: Option<i64>,
    pub maximum: Option<i64>,
    pub default: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArrayOptions {
    pub min_items: Option<usize>,
    pub max_items: Option<usize>,
    pub items: Option<Box<Variable>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ObjectOptions {
    #[serde(default)]
    pub properties: BTreeMap<String, Variable>,

    #[serde(default)]
    pub required: HashSet<String>,

    #[serde(default)]
    pub additional_properties: bool,

    #[serde(rename = "$ref")]
    pub object_reference: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum StringFormat {
    #[serde(rename = "date-time")]
    DateTime,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StringOptions {
    pub pattern: Option<String>,
    pub format: Option<StringFormat>,
    pub max_length: Option<usize>,
    pub min_length: Option<usize>,

    #[serde(rename = "enum")]
    pub enum_items: Option<Vec<String>>,

    pub default: Option<String>,

    #[serde(rename = "$ref")]
    pub object_reference: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", tag = "type", deny_unknown_fields)]
pub enum Type {
    Null,
    Boolean,
    String(StringOptions),
    Number(NumberOptions),
    Integer(IntegerOptions),
    Array(ArrayOptions),
    Object(ObjectOptions),
}

impl<'de> Deserialize<'de> for Variable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let serde_yaml::Value::Mapping(mut map) = Deserialize::deserialize(deserializer)? else {
            return Err(serde::de::Error::custom("Variable must be a mapping"));
        };

        let description: Option<String> = match map.remove("description") {
            None => None,
            Some(v) => Some(
                serde_yaml::from_value(v)
                    .map_err(|_| serde::de::Error::custom("'description' is not a String'"))?,
            ),
        };

        let arg_type = map
            .remove("type")
            .unwrap_or(serde_yaml::Value::String("object".to_string()));

        let arg = match arg_type {
            val @ serde_yaml::Value::String(_) => {
                map.insert(serde_yaml::Value::String("type".to_string()), val);
                let t: Type = serde_yaml::from_value(serde_yaml::Value::Mapping(map))
                    .map_err(|e| serde::de::Error::custom(e.to_string()))?;
                Argument::Single(t)
            }
            serde_yaml::Value::Sequence(s) => {
                let mut types = Vec::new();
                for t in s.into_iter() {
                    let mut mapping = serde_yaml::Mapping::new();
                    mapping.insert(serde_yaml::Value::String("type".to_string()), t);
                    let t: Type = serde_yaml::from_value(serde_yaml::Value::Mapping(mapping))
                        .map_err(|e| serde::de::Error::custom(e.to_string()))?;
                    types.push(t);
                }
                Argument::Multiple(types)
            }
            _ => {
                return Err(serde::de::Error::custom(
                    "'type' must be a sequence or a string.",
                ))
            }
        };

        Ok(Variable { description, arg })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorEntry {
    pub reference: String,
}
