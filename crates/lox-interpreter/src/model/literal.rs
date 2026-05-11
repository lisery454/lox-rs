use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{self, Display},
    rc::Rc,
};

use ordered_float::OrderedFloat;

use crate::{
    error::LoxResult,
    model::{environment::Environment, token::Token},
};

type NativeFunction =
    Rc<dyn Fn(Rc<RefCell<Environment>>, Vec<LiteralValue>) -> LoxResult<LiteralValue>>;

pub struct LoxFunction {
    pub function: NativeFunction,
    pub arg_size: usize,
    pub closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    pub fn new(f: NativeFunction, arg_size: usize, closure: Rc<RefCell<Environment>>) -> Self {
        Self {
            function: f,
            arg_size,
            closure,
        }
    }

    pub fn bind(self: Rc<Self>, this: LiteralValue) -> Rc<Self> {
        let new_env = Rc::new(RefCell::new(Environment::new_with_parent(&self.closure)));
        new_env.borrow_mut().define(&"this".into(), this);
        return Rc::new(Self {
            function: Rc::clone(&self.function),
            arg_size: self.arg_size,
            closure: new_env,
        });
    }
}

pub struct LoxClass {
    pub name: String,
    pub methods: HashMap<String, Rc<LoxFunction>>,
    pub constructor: Option<Rc<LoxFunction>>,
    pub super_class: Option<Rc<LoxClass>>,
}

impl LoxClass {
    pub fn new(
        name: &String,
        methods: HashMap<String, Rc<LoxFunction>>,
        super_class: Option<Rc<LoxClass>>,
    ) -> Self {
        let mut result = Self {
            name: name.clone(),
            methods,
            constructor: None,
            super_class,
        };

        result.constructor = result.find_method(&"init".to_string()).map(|t| t.clone());

        return result;
    }

    pub fn cons_len(&self) -> usize {
        match &self.constructor {
            Some(f) => f.arg_size,
            None => 0,
        }
    }

    pub fn find_method(&self, name: &String) -> Option<&Rc<LoxFunction>> {
        let f = self.methods.get(name);
        if let Some(_) = f {
            return f;
        }

        if let Some(sc) = &self.super_class {
            let method = sc.find_method(name);
            return method;
        }

        return None;
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "class <{}>", self.name)
    }
}

pub struct LoxInstance {
    pub class: Rc<LoxClass>,
    pub fields: RefCell<HashMap<String, LiteralValue>>,
}

impl LoxInstance {
    pub fn new(class: &Rc<LoxClass>) -> Self {
        Self {
            class: Rc::clone(&class),
            fields: RefCell::new(HashMap::new()),
        }
    }

    pub fn get(self: Rc<Self>, name: &Token) -> LiteralValue {
        let fields = self.fields.borrow();
        let value = fields.get(&name.lexeme);
        if let Some(v) = value {
            v.clone()
        } else if let Some(f) = self.class.find_method(&name.lexeme) {
            let this = LiteralValue::ClassInstance(Rc::clone(&self));
            let new_f = Rc::clone(&f).bind(this);
            LiteralValue::Callable(new_f)
        } else {
            LiteralValue::Nil
        }
    }

    pub fn set(&self, name: &Token, value: LiteralValue) {
        let mut fields = self.fields.borrow_mut();
        fields.insert(name.lexeme.clone(), value);
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "class instance <{}>", self.class)
    }
}

#[derive(Clone)]
pub enum LiteralValue {
    Number(OrderedFloat<f64>),
    String(String),
    Bool(bool),
    Callable(Rc<LoxFunction>),
    Class(Rc<LoxClass>),
    ClassInstance(Rc<LoxInstance>),
    Nil,
}

impl fmt::Debug for LiteralValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteralValue::Number(n) => write!(f, "Number({})", n),
            LiteralValue::String(s) => write!(f, "String({:?})", s),
            LiteralValue::Bool(b) => write!(f, "Bool({})", b),
            LiteralValue::Nil => write!(f, "Nil"),
            LiteralValue::Callable(func) => {
                write!(f, "<native fn ({} args)>", func.arg_size)
            }
            LiteralValue::Class(c) => {
                write!(f, "<native class ({})>", c.name)
            }
            LiteralValue::ClassInstance(i) => {
                write!(f, "<class instance ({})>", i.class.name)
            }
        }
    }
}

impl LiteralValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            LiteralValue::Number(n) => {
                if n.abs() > f64::EPSILON {
                    true
                } else {
                    true
                }
            }
            LiteralValue::String(s) => {
                if s.len() > 0 {
                    true
                } else {
                    false
                }
            }
            LiteralValue::Bool(b) => {
                if *b {
                    true
                } else {
                    false
                }
            }
            LiteralValue::Nil => false,
            LiteralValue::Callable(_) => true,
            LiteralValue::Class(_) => true,
            LiteralValue::ClassInstance(_) => true,
        }
    }
}

impl Display for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralValue::Number(n) => write!(f, "{}", n)?,
            LiteralValue::String(s) => write!(f, "{}", s)?,
            LiteralValue::Bool(b) => write!(f, "{}", b)?,
            LiteralValue::Nil => write!(f, "nil",)?,
            LiteralValue::Callable(_) => write!(f, "callable",)?,
            LiteralValue::Class(class) => write!(f, "{class}")?,
            LiteralValue::ClassInstance(class_instance) => write!(f, "{class_instance}")?,
        }
        Ok(())
    }
}

impl From<String> for LiteralValue {
    fn from(s: String) -> Self {
        LiteralValue::String(s)
    }
}

impl From<OrderedFloat<f64>> for LiteralValue {
    fn from(n: OrderedFloat<f64>) -> Self {
        LiteralValue::Number(n)
    }
}

impl From<bool> for LiteralValue {
    fn from(b: bool) -> Self {
        LiteralValue::Bool(b)
    }
}

impl From<()> for LiteralValue {
    fn from(_: ()) -> Self {
        LiteralValue::Nil
    }
}
