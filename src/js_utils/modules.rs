use crate::js_utils::adapters::JsRealmAdapter;

pub trait ScriptModuleLoader {
    fn normalize_path(&self, ref_path: &str, path: &str) -> Option<String>;
    fn load_module(&self, absolute_path: &str) -> String;
}

pub trait NativeModuleLoader<R: JsRealmAdapter> {
    fn has_module(&self, q_ctx: &R, module_name: &str) -> bool;
    fn get_module_export_names(&self, q_ctx: &R, module_name: &str) -> Vec<&str>;
    fn get_module_exports(
        &self,
        q_ctx: &R,
        module_name: &str,
    ) -> Vec<(&str, R::JsValueAdapterType)>;
}
