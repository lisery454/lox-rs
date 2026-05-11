use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::model::literal::LiteralValue;

#[derive(Debug)]
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

    // pub(crate) fn print_parent_link(&self) {
    //     print!("{:p} ({})", self, self.values.len());
    //     let mut parent = self.parent.clone();

    //     while let Some(curr_parent) = parent {
    //         print!(
    //             "-> {:p} ({})",
    //             curr_parent.as_ptr(),
    //             curr_parent.borrow().values.len()
    //         );

    //         parent = curr_parent.borrow().parent.clone();
    //     }

    //     println!("");
    // }

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

    pub fn assign_at(&mut self, distance: &u32, name: &String, value: LiteralValue) -> bool {
        if *distance == 0 {
            return self.assign(name, value);
        } else if let Some(parent) = self.ancestor(distance) {
            return parent.borrow_mut().assign(name, value);
        } else {
            return false;
        }
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

    pub fn get_at(&self, distance: &u32, name: &String) -> LiteralValue {
        if *distance == 0 {
            return self.get(name);
        } else if let Some(parent) = self.ancestor(distance) {
            return parent.borrow().get(name);
        } else {
            return LiteralValue::Nil;
        }
    }

    fn ancestor(&self, distance: &u32) -> Option<Rc<RefCell<Environment>>> {
        let mut env: Rc<RefCell<Environment>>;
        if let Some(parent) = &self.parent {
            env = Rc::clone(parent);
        } else {
            return None;
        }

        for _ in 1..(*distance as i32) {
            let new_env = if let Some(parent) = &env.borrow().parent {
                Some(Rc::clone(parent))
            } else {
                None
            };

            if let Some(new_env) = new_env {
                env = new_env;
            } else {
                return None;
            }
        }
        return Some(env);
    }
}
