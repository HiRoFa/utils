use crate::js_utils::{JsError, Script};

pub trait JsRuntimeAdapter {
    type JsValueAdapterType: JsValueAdapter;
    type JsContextAdapterType: JsContextAdapter;

    fn js_create_context(&self, id: &str) -> Result<Box<Self::JsValueAdapterType>, JsError>;
    fn js_get_context(&self, id: &str) -> Option<Box<Self::JsValueAdapterType>>;
    fn js_get_main_context(&self) -> Option<&Self::JsContextAdapterType>;
}

pub type JsFunction =
    dyn Fn(dyn JsValueAdapter, Vec<dyn JsValueAdapter>) -> Result<dyn JsValueAdapter, JsError>;

pub trait JsContextAdapter {
    type JsRuntimeAdapterType: JsRuntimeAdapter;

    fn js_eval(
        &self,
        script: Script,
    ) -> Result<
        <<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
        JsError,
    >;

    fn js_install_function<
        F: Fn(
            <<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
            Vec<<<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType>,
        ) -> Result<<<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType, JsError>,
    >(
        &self,
        namespace: Vec<&str>,
        name: &str,
        js_function: F,
        arg_count: u32,
    ) -> Result<(), JsError>;
    fn js_eval_module(
        &self,
        script: Script,
        return_value: &mut dyn JsValueAdapter,
    ) -> Result<(), JsError>;
    fn js_get_namespace(
        &self,
        namespace: &[&str],
        return_value: &mut dyn JsValueAdapter,
    ) -> Result<(), JsError>;
    fn js_invoke_function(
        &self,
        namespace: &[&str],
        method_name: &str,
        args: &[&dyn JsValueAdapter],
        return_value: &mut dyn JsValueAdapter,
    ) -> Result<(), JsError>;
    fn js_invoke_function2(
        &self,
        this_obj: &dyn JsValueAdapter,
        method_name: &str,
        args: &[&dyn JsValueAdapter],
        return_value: &mut dyn JsValueAdapter,
    ) -> Result<(), JsError>;
    fn js_get_object_property(
        &self,
        object: &dyn JsValueAdapter,
        property_name: &str,
        return_value: &mut dyn JsValueAdapter,
    ) -> Result<(), JsError>;
    fn js_delete_object_property(
        &self,
        object: &dyn JsValueAdapter,
        property_name: &str,
    ) -> Result<(), JsError>;
    fn js_set_object_property(
        &self,
        object: &dyn JsValueAdapter,
        property_name: &str,
        property: &mut dyn JsValueAdapter,
    ) -> Result<(), JsError>;
}

pub trait JsValueAdapter {
    fn js_is_bool(&self) -> bool;
    fn js_is_i32(&self) -> bool;
    fn js_is_f64(&self) -> bool;
    fn js_is_object(&self) -> bool;
    fn js_is_string(&self) -> bool;
    fn js_is_function(&self) -> bool;
    fn js_is_bigint(&self) -> bool;
    fn js_type_of(&self) -> &'static str;
    fn js_to_bool(&self) -> bool;
    fn js_to_i32(&self) -> i32;
    fn js_to_f64(&self) -> f64;
    fn js_to_string(&self) -> String;
}
