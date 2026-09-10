use std::{collections::HashMap, fmt::write};

use anyhow::bail;

use crate::model::Value;

pub type ObjAddr = usize;

pub struct Object {
    pub is_marked: bool,  // GC 标记位
    pub kind: ObjectKind, // 具体的对象类型
}

pub enum ObjectKind {
    String(String),
}

impl std::fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectKind::String(s) => write!(f, "{}", s),
        }
    }
}

pub struct Memory {
    pub heap: Vec<Option<Object>>,
    pub stack: Vec<Value>,
    pub globals: HashMap<String, Value>,

    pub string_pool: HashMap<String, ObjAddr>,
}

impl Memory {
    const STACK_MAX: usize = 256;

    pub fn new() -> Self {
        Self {
            heap: Vec::new(),
            stack: Vec::new(),
            globals: HashMap::new(),
            string_pool: HashMap::new(),
        }
    }

    pub fn stack_push(&mut self, v: Value) {
        if self.stack.len() >= Self::STACK_MAX {
            panic!("stack over flow");
        }
        self.stack.push(v);
    }

    pub fn stack_peek(&self) -> &Value {
        let v = self.stack.last();
        if let Some(v) = v {
            return v;
        } else {
            panic!("stack is empty");
        }
    }

    pub fn stack_pop(&mut self) -> Value {
        let v = self.stack.pop();
        if let Some(v) = v {
            return v;
        } else {
            panic!("stack is empty");
        }
    }

    pub fn stack_get(&self, slot: usize) -> Option<&Value> {
        self.stack.get(slot)
    }

    pub fn stack_set(&mut self, slot: usize, value: Value) {
        self.stack[slot] = value;
    }

    pub fn get_obj(&self, addr: ObjAddr) -> Option<&ObjectKind> {
        self.heap.get(addr)?.as_ref().map(|obj| &obj.kind)
    }

    pub fn alloc(&mut self, kind: ObjectKind) -> ObjAddr {
        let obj = Object {
            is_marked: false,
            kind,
        };

        // 简易实现：直接推入 heap 数组末尾（后续可扩展：优先复用空槽位）
        let addr = self.heap.len();
        self.heap.push(Some(obj));
        addr
    }

    pub fn alloc_string(&mut self, s: String) -> ObjAddr {
        if let Some(&addr) = self.string_pool.get(&s) {
            return addr;
        }

        let addr = self.alloc(ObjectKind::String(s.clone()));

        self.string_pool.insert(s, addr);

        addr
    }

    pub fn get_string(&self, obj: Value) -> anyhow::Result<String> {
        let Value::Object(addr) = obj else {
            bail!("not find obj");
        };

        let Some(ObjectKind::String(name_str)) = self.get_obj(addr) else {
            bail!("obj is not a string");
        };
        let name_str = name_str.to_string();
        Ok(name_str)
    }

    pub fn insert_global(&mut self, name: &str, val: Value) {
        self.globals.insert(name.to_string(), val);
    }

    pub fn get_global(&mut self, name: &str) -> Option<&Value> {
        self.globals.get(name)
    }

    pub fn set_global(&mut self, name: &str, new_val: Value) -> anyhow::Result<()> {
        if let Some(val) = self.globals.get_mut(name) {
            *val = new_val;
            return Ok(());
        }
        bail!("not find name {} in global", name);
    }
}
