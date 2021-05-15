use crate::js_utils::{JsError, Script};

pub trait JsRuntimeAdapter {}

pub trait JsContextAdapter {
    fn eval(
        &self,
        script: Script,
        return_value: &mut dyn JsValueAdapter,
    ) -> Result<(), Box<dyn JsError>>;
    fn eval_module(
        &self,
        script: Script,
        return_value: &mut dyn JsValueAdapter,
    ) -> Result<(), Box<dyn JsError>>;
    fn get_namespace(
        &self,
        namespace: &[&str],
        return_value: &mut dyn JsValueAdapter,
    ) -> Result<(), Box<dyn JsError>>;
    fn invoke_function(
        &self,
        namespace: &[&str],
        method_name: &str,
        args: &[&dyn JsValueAdapter],
        return_value: &mut dyn JsValueAdapter,
    ) -> Result<(), Box<dyn JsError>>;
    fn invoke_function2(
        &self,
        this_obj: &dyn JsValueAdapter,
        method_name: &str,
        args: &[&dyn JsValueAdapter],
        return_value: &mut dyn JsValueAdapter,
    ) -> Result<(), Box<dyn JsError>>;
    fn get_object_property(
        &self,
        object: &dyn JsValueAdapter,
        property_name: &str,
        return_value: &mut dyn JsValueAdapter,
    ) -> Result<(), Box<dyn JsError>>;
    fn delete_object_property(
        &self,
        object: &dyn JsValueAdapter,
        property_name: &str,
    ) -> Result<(), Box<dyn JsError>>;
    fn set_object_property(
        &self,
        object: &dyn JsValueAdapter,
        property_name: &str,
        property: &mut dyn JsValueAdapter,
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
