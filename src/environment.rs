use std::collections::HashMap;

use crate::{model::literal::LiteralValue};

pub struct Environment {
    values: HashMap<String, LiteralValue>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &String, value: LiteralValue) {
        self.values.insert(name.clone(), value);
    }

    pub fn assign(&mut self, name: &String, value: LiteralValue) -> bool {
        if let Some(v) = self.values.get_mut(name) {
            *v = value;
            return true;
        }
        return false;
    }

    pub fn get(&self, name: &String) -> &LiteralValue {
        if let Some(v) = self.values.get(name) {
            return v;
        } else {
            return &LiteralValue::Nil;
        }
    }
}
