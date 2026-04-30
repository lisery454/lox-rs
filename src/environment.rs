use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::model::literal::LiteralValue;

pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, LiteralValue>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            parent: None,
            values: HashMap::new(),
        }
    }

    pub fn new_with_parent(env: &Rc<RefCell<Environment>>) -> Self {
        Self {
            parent: Some(Rc::clone(env)),
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
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().assign(name, value);
            return true;
        }
        return false;
    }

    pub fn get(&self, name: &String) -> LiteralValue {
        if let Some(v) = self.values.get(name) {
            return v.clone();
        } else if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        } else {
            return LiteralValue::Nil;
        }
    }
}
