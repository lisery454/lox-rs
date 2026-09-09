use crate::model::Value;

pub type ObjAddr = usize;

pub struct Object {
    pub is_marked: bool,  // GC 标记位
    pub kind: ObjectKind, // 具体的对象类型
}

pub enum ObjectKind {
    String(String),
}

pub struct Memory {
    pub heap: Vec<Option<Object>>,
    pub stack: Vec<Value>,
}

impl Memory {
    const STACK_MAX: usize = 256;

    pub fn new() -> Self {
        Self {
            heap: Vec::new(),
            stack: Vec::new(),
        }
    }

    pub fn get_obj(&self, addr: ObjAddr) -> Option<&ObjectKind> {
        self.heap.get(addr)?.as_ref().map(|obj| &obj.kind)
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
}
