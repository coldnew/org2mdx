use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::HashMap;

use super::Position;

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

    pub fn data_num(mut self, key: &str, value: u64) -> Self {
        self.data
            .insert(key.to_string(), Value::Number(serde_json::Number::from(value)));
        self
    }

    pub fn data_list_val(mut self, key: &str, values: Vec<Value>) -> Self {
        self.data.insert(key.to_string(), Value::Array(values));
        self
    }

    pub fn with_data_map(mut self, map: HashMap<String, Value>) -> Self {
        self.data.extend(map);
        self
    }

    pub fn get_data_str(&self, key: &str) -> Option<&str> {
        self.data.get(key).and_then(|v| v.as_str())
    }

    pub fn get_data_num(&self, key: &str) -> Option<u64> {
        self.data.get(key).and_then(|v| v.as_u64())
    }

    pub fn get_data_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key).and_then(|v| v.as_bool())
    }

    pub fn get_data_list(&self, key: &str) -> Option<&Vec<Value>> {
        self.data.get(key).and_then(|v| v.as_array())
    }

    pub fn get_data_map(&self, key: &str) -> Option<&Map<String, Value>> {
        self.data.get(key).and_then(|v| v.as_object())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_new() {
        let node = Node::new("paragraph");
        assert_eq!(node.r#type, "paragraph");
        assert_eq!(node.children, None);
    }

    #[test]
    fn test_node_with_children() {
        let node = Node::new("section").with_children(vec![Node::text("hello")]);
        assert_eq!(node.children.unwrap().len(), 1);
    }

    #[test]
    fn test_node_data() {
        let node = Node::new("heading")
            .data_num("depth", 2)
            .data_str("id", "intro")
            .data_bool("todo", true);
        assert_eq!(node.get_data_num("depth"), Some(2));
        assert_eq!(node.get_data_str("id"), Some("intro"));
        assert_eq!(node.get_data_bool("todo"), Some(true));
    }

    #[test]
    fn test_node_data_list() {
        let node = Node::new("heading").data_list_val(
            "tags",
            vec![Value::String("rust".into()), Value::String("org".into())],
        );
        let list = node.get_data_list("tags").unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].as_str(), Some("rust"));
    }

    #[test]
    fn test_node_with_data_map() {
        let mut map = HashMap::new();
        map.insert("title".into(), Value::String("Hello".into()));
        map.insert("date".into(), Value::String("2024-01-01".into()));
        let node = Node::root(vec![]).with_data_map(map);
        assert_eq!(node.get_data_str("title"), Some("Hello"));
        assert_eq!(node.get_data_str("date"), Some("2024-01-01"));
    }
}
