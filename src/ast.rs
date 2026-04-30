use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Position {
    pub start: Point,
    pub end: Point,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Point {
    pub line: u32,
    pub column: u32,
    pub offset: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Node {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Node>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub data: HashMap<String, Value>,
}

impl Node {
    pub fn new(r#type: &str) -> Self {
        Node {
            r#type: r#type.to_string(),
            children: None,
            value: None,
            position: None,
            data: HashMap::new(),
        }
    }

    pub fn root(children: Vec<Node>) -> Self {
        Node {
            r#type: "root".to_string(),
            children: Some(children),
            value: None,
            position: None,
            data: HashMap::new(),
        }
    }

    pub fn text(value: &str) -> Self {
        Node {
            r#type: "text".to_string(),
            children: None,
            value: Some(value.to_string()),
            position: None,
            data: HashMap::new(),
        }
    }

    pub fn with_children(mut self, children: Vec<Node>) -> Self {
        self.children = Some(children);
        self
    }

    pub fn with_value(mut self, value: &str) -> Self {
        self.value = Some(value.to_string());
        self
    }

    pub fn data_str(mut self, key: &str, value: &str) -> Self {
        self.data
            .insert(key.to_string(), Value::String(value.to_string()));
        self
    }

    pub fn data_bool(mut self, key: &str, value: bool) -> Self {
        self.data.insert(key.to_string(), Value::Bool(value));
        self
    }

    pub fn data_num(mut self, key: &str, value: u8) -> Self {
        self.data
            .insert(key.to_string(), Value::Number(value.into()));
        self
    }

    pub fn data_list_val(mut self, key: &str, values: Vec<Value>) -> Self {
        self.data.insert(key.to_string(), Value::Array(values));
        self
    }

    pub fn with_data_map(mut self, map: HashMap<String, Value>) -> Self {
        self.data = map;
        self
    }

    pub fn get_data_str(&self, key: &str) -> Option<&str> {
        self.data.get(key).and_then(|v| v.as_str())
    }

    pub fn get_data_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key).and_then(|v| v.as_bool())
    }

    pub fn get_data_num(&self, key: &str) -> Option<u8> {
        self.data.get(key).and_then(|v| v.as_u64()).map(|n| n as u8)
    }

    pub fn get_data_list(&self, key: &str) -> Vec<String> {
        self.data
            .get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }
}
