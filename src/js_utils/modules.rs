use crate::js_utils::adapters::JsRealmAdapter;
use std::sync::Arc;

pub trait ScriptModuleLoader<R: JsRealmAdapter> {
    fn normalize_path(&self, realm: &R, ref_path: &str, path: &str) -> Option<String>;
    fn load_module(&self, realm: &R, absolute_path: &str) -> String;
}

pub trait CompiledModuleLoader<R: JsRealmAdapter> {
    fn normalize_path(&self, realm: &R, ref_path: &str, path: &str) -> Option<String>;
    fn load_module(&self, realm: &R, absolute_path: &str) -> Arc<Vec<u8>>;
}

pub trait NativeModuleLoader<R: JsRealmAdapter> {
    fn has_module(&self, realm: &R, module_name: &str) -> bool;
    fn get_module_export_names(&self, realm: &R, module_name: &str) -> Vec<&str>;
    fn get_module_exports(
        &self,
        realm: &R,
        module_name: &str,
    ) -> Vec<(&str, R::JsValueAdapterType)>;
}
