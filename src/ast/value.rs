use crate::sql;
use serde::{
    Deserialize,
    Serialize,
};
use std::fmt;

/// coarse value from the parsing
/// this is close to the json values
///
/// TODO: include Integer variant
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Value {
    Null,
    String(String),
    Number(f64),
    Bool(bool),
}

/// if string is empty use the default value in the underlying database
impl Into<sql::Value> for &Value {
    fn into(self) -> sql::Value {
        match self {
            Value::Null => sql::Value::Null,
            Value::String(v) => {
                if v.is_empty() {
                    sql::Value::Default
                } else {
                    sql::Value::SingleQuotedString(v.to_string())
                }
            }
            Value::Number(v) => sql::Value::Number(format!("{}", v)),
            Value::Bool(v) => sql::Value::Boolean(*v),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::String(v) => write!(f, "'{}'", v),
            Value::Number(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
        }
    }
}
