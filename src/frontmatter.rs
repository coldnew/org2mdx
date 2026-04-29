use std::collections::HashMap;

pub enum FmVal {
    Str(String),
    List(Vec<String>),
}

pub struct FrontmatterBuilder {
    keys: Vec<String>,
    vals: HashMap<String, FmVal>,
}

impl FrontmatterBuilder {
    pub fn new() -> Self {
        Self {
            keys: Vec::new(),
            vals: HashMap::new(),
        }
    }

    pub fn set_str(&mut self, key: &str, val: String) {
        if !self.vals.contains_key(key) {
            self.keys.push(key.to_string());
        }
        self.vals.insert(key.to_string(), FmVal::Str(val));
    }

    pub fn push_list(&mut self, key: &str, val: String) {
        if let Some(FmVal::List(ref mut v)) = self.vals.get_mut(key) {
            v.push(val);
        } else {
            if !self.vals.contains_key(key) {
                self.keys.push(key.to_string());
            }
            self.vals.insert(key.to_string(), FmVal::List(vec![val]));
        }
    }

    pub fn set_list(&mut self, key: &str, vals: Vec<String>) {
        if !self.vals.contains_key(key) {
            self.keys.push(key.to_string());
        }
        self.vals.insert(key.to_string(), FmVal::List(vals));
    }

    pub fn build(&self) -> String {
        let mut s = String::from("---\n");
        for key in &self.keys {
            match self.vals.get(key) {
                Some(FmVal::Str(v)) => {
                    if key == "abbrlink" {
                        s += &format!("{}: {}\n", key, v);
                    } else {
                        s += &format!("{}: {}\n", key, crate::util::yaml_str(v));
                    }
                }
                Some(FmVal::List(items)) => {
                    s += &format!("{}:\n", key);
                    for item in items {
                        s += &format!("  - {}\n", item);
                    }
                }
                None => {}
            }
        }
        s += "---\n\n";
        s
    }
}
