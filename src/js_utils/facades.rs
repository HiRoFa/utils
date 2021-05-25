use crate::js_utils::adapters::{JsContextAdapter, JsRuntimeAdapter, JsValueAdapter};
use crate::js_utils::{JsError, Script};
use std::future::Future;

pub struct JsProxy {}

pub trait JsRuntimeFacade {
    type JsRuntimeAdapterType: JsRuntimeAdapter;
    type ContextFacadeType: JsContextFacade;

    fn js_create_context(&self) -> Self::ContextFacadeType;
    fn js_loop_exe<
        R: Send + 'static,
        C: FnOnce(&Self::JsRuntimeAdapterType) -> R + Send + 'static,
    >(
        &self,
        consumer: C,
    ) -> R;
    fn js_loop_add(&self);
}

pub trait JsContextFacade {
    type JsRuntimeAdapterType: JsRuntimeAdapter;

    fn js_install_proxy(&self, js_proxy: JsProxy);
    fn js_eval(&self, script: Script) -> Box<dyn Future<Output = Result<JsValueFacade, JsError>>>;
    fn js_loop_exe<
        R,
        C: FnOnce(&Self::JsRuntimeAdapterType, &<<Self as JsContextFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsContextAdapterType) -> R,
    >(
        &self,
        consumer: C,
    ) -> R;
    fn js_loop_add(&self);
}

pub struct JsValueFacade {}

impl JsValueFacade {
    pub fn from_js_value_adapter<C: JsContextAdapter + Sized, V: JsValueAdapter + Sized>(
        _ctx: C,
        _value: V,
    ) -> JsValueFacade {
        unimplemented!()
    }
}
