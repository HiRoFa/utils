//! This contains abstract traits and structs for use with different javascript runtimes
//! the Adapter traits are use in the worker thread (EventLoop) of the Runtime and thus are not Send, they should never leave the thread
//! The facade classes are for use outside the worker thread, they are Send
//!

pub mod adapters;
pub mod facades;
pub mod fetch;

pub trait JsError {
    fn get_message(&self) -> &str;
    fn get_filename(&self) -> Option<&str>;
    fn get_line_num(&self) -> usize;
    fn get_position(&self) -> usize;
}

pub struct Script {
    code: String,
    path: String,
}

impl Script {
    pub fn get_code(&self) -> &str {
        self.code.as_str()
    }
    pub fn get_path(&self) -> &str {
        self.path.as_str()
    }
    pub fn set_code(&mut self, code: &str) {
        self.code = code.to_string();
    }
    pub fn set_path(&mut self, path: &str) {
        self.path = path.to_string();
    }
}
