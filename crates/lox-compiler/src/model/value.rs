pub type Value = f64;

#[derive(Clone)]
pub struct ValueArray {
    pub(crate) values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn read(&self, i: usize) -> Option<&Value> {
        self.values.get(i)
    }

    pub fn write(&mut self, t: Value) -> usize {
        self.values.push(t);
        self.values.len() - 1
    }
}
