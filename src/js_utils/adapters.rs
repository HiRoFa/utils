use crate::js_utils::{JsError, Script};

pub trait JsRuntimeAdapter {}

pub trait JsContextAdapter {
    fn eval(script: Script, return_value: &mut dyn JsValueAdapter) -> Result<(), Box<dyn JsError>>;
    fn eval_module(
        script: Script,
        return_value: &mut dyn JsValueAdapter,
    ) -> Result<(), Box<dyn JsError>>;
    fn invoke_function(
        namespace: &[&str],
        method_name: &str,
        args: &[&dyn JsValueAdapter],
        return_value: &mut dyn JsValueAdapter,
    ) -> Result<(), Box<dyn JsError>>;
}

pub trait JsValueAdapter {
    fn is_bool(&self) -> bool;
    fn is_i32(&self) -> bool;
    fn is_f64(&self) -> bool;
    fn is_object(&self) -> bool;
    fn is_string(&self) -> bool;
    fn is_function(&self) -> bool;
    fn is_bigint(&self) -> bool;
}
