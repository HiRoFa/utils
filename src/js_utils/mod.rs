//! This contains abstract traits and structs for use with different javascript runtimes
//! the Adapter traits are use in the worker thread (EventLoop) of the Runtime and thus are not Send, they should never leave the thread
//! The facade classes are for use outside the worker thread, they are Send
//!

use std::fmt::{Error, Formatter};

pub mod adapters;
pub mod facades;
pub mod modules;

pub trait ScriptPreProcessor {
    fn process(&self, script: &mut Script) -> Result<(), JsError>;
}

pub struct JsError {
    name: String,
    message: String,
    stack: String,
}

impl JsError {
    pub fn new(name: String, message: String, stack: String) -> Self {
        Self {
            name,
            message,
            stack,
        }
    }
    pub fn new_str(err: &str) -> Self {
        Self::new_string(err.to_string())
    }
    pub fn new_string(err: String) -> Self {
        JsError {
            name: "".to_string(),
            message: err,
            stack: "".to_string(),
        }
    }
    pub fn get_message(&self) -> &str {
        self.message.as_str()
    }
    pub fn get_stack(&self) -> &str {
        self.stack.as_str()
    }
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
}

impl std::fmt::Display for JsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let e = format!("{}: {}\n{}", self.name, self.message, self.stack);
        f.write_str(e.as_str())
    }
}

pub struct Script {
    path: String,
    code: String,
}

impl Script {
    pub fn new(absolute_path: &str, script_code: &str) -> Self {
        Self {
            path: absolute_path.to_string(),
            code: script_code.to_string(),
        }
    }
    pub fn get_path(&self) -> &str {
        self.path.as_str()
    }
    pub fn get_code(&self) -> &str {
        self.code.as_str()
    }
    pub fn set_code(&mut self, code: String) {
        self.code = code;
    }
}

impl Clone for Script {
    fn clone(&self) -> Self {
        Self {
            path: self.get_path().to_string(),
            code: self.get_code().to_string(),
        }
    }
}
