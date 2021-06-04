use crate::js_utils::{JsError, Script};

pub trait JsRuntimeAdapter {
    type JsValueAdapterType: JsValueAdapter;
    type JsContextAdapterType: JsContextAdapter;

    fn js_create_context(&self, id: &str) -> Result<Box<Self::JsContextAdapterType>, JsError>;
    fn js_get_context(&self, id: &str) -> Option<Box<Self::JsContextAdapterType>>;
    fn js_get_main_context(&self) -> &Self::JsContextAdapterType;
}

//pub type JsFunction =
//    dyn Fn(dyn JsValueAdapter, Vec<dyn JsValueAdapter>) -> Result<dyn JsValueAdapter, JsError>;

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
            &<<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsContextAdapterType,
            &<<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
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
    ) -> Result<
        <<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
        JsError,
    >;
    fn js_get_namespace(
        &self,
        namespace: &[&str],
    ) -> Result<
        <<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
        JsError,
    >;
    // function methods
    fn js_function_invoke(
        &self,
        namespace: &[&str],
        method_name: &str,
        args: &[&<<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType],
    ) -> Result<
        <<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
        JsError,
    >;
    fn js_function_invoke2(
        &self,
        this_obj: &<<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
        method_name: &str,
        args: &[&<<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType],
    ) -> Result<
        <<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
        JsError,
    >;
    fn js_function_invoke3(
        &self,
        this_obj: Option<&<<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType>,
        function_obj: &<<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
        args: &[&<<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType],
    ) -> Result<
        <<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
        JsError,
    >;
    //object functions
    fn js_object_delete_property(
        &self,
        object: &<<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
        property_name: &str,
    ) -> Result<(), JsError>;
    fn js_object_set_property(
        &self,
        object: &<<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
        property_name: &str,
        property: &<<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
    ) -> Result<(), JsError>;

    fn js_object_get_property(
        &self,
        object: &<<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
        property_name: &str,
    ) -> Result<
        <<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
        JsError,
    >;
    fn js_object_create_new(
        &self,
    ) -> Result<
        <<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
        JsError,
    >;
    fn js_object_get_properties(
        &self,
        object: &<<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
    ) -> Result<Vec<String>, JsError>;

    // array functions
    fn js_array_get_element(
        &self,
        array: &<<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
        index: u32,
    ) -> Result<
        <<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
        JsError,
    >;
    fn js_array_set_element(
        &self,
        array: &<<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
        index: u32,
        element: <<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
    ) -> Result<(), JsError>;
    fn js_array_get_length(
        &self,
        array: &<<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
    ) -> Result<u32, JsError>;
    fn js_array_create_new(
        &self,
    ) -> Result<
        <<Self as JsContextAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsValueAdapterType,
        JsError,
    >;
    // array_foreach
    // object_foreach
    // promises

    // create new
    // resolve/reject
    // add then/catch/finally

    // cache obj (add/consume/delete/with)

    // instanceof
}

pub trait JsValueAdapter {
    type JsRuntimeAdapterType: JsRuntimeAdapter;

    fn js_is_bool(&self) -> bool;
    fn js_is_i32(&self) -> bool;
    fn js_is_f64(&self) -> bool;
    fn js_is_object(&self) -> bool;
    fn js_is_string(&self) -> bool;
    fn js_is_function(&self) -> bool;
    fn js_is_bigint(&self) -> bool;
    fn js_is_null(&self) -> bool;
    fn js_is_undefined(&self) -> bool;
    fn js_is_null_or_undefined(&self) -> bool {
        self.js_is_null() || self.js_is_undefined()
    }

    fn js_type_of(&self) -> &'static str;
    fn js_to_bool(&self) -> bool;
    fn js_to_i32(&self) -> i32;
    fn js_to_f64(&self) -> f64;
    fn js_to_string(&self) -> String;
}
