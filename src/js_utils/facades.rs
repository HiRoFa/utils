use crate::js_utils::adapters::{JsContextAdapter, JsRuntimeAdapter, JsValueAdapter};
use crate::js_utils::{JsError, Script};
use std::future::Future;

pub struct JsProxy {}

pub trait JsRuntimeBuilder {
    type JsRuntimeFacadeType: JsRuntimeFacade;
    fn build(self) -> Self::JsRuntimeFacadeType;
}

pub trait JsRuntimeFacade {
    type JsRuntimeAdapterType: JsRuntimeAdapter;
    type JsContextFacadeType: JsContextFacade;

    fn js_create_context(&self, name: &str) -> Self::JsContextFacadeType;
    fn js_get_main_context(&self) -> &Self::JsContextFacadeType;
    fn js_get_context(&self, name: &str) -> &Self::JsContextFacadeType;
    fn js_loop_sync<
        R: Send + 'static,
        C: FnOnce(&Self::JsRuntimeAdapterType) -> R + Send + 'static,
    >(
        &self,
        consumer: C,
    ) -> R;
    fn js_loop<R: Send + 'static, C: FnOnce(&Self::JsRuntimeAdapterType) -> R + Send + 'static>(
        &self,
        consumer: C,
    ) -> Box<dyn Future<Output = R>>;
    fn js_loop_void<C: FnOnce(&Self::JsRuntimeAdapterType) + Send + 'static>(&self, consumer: C);
}

pub trait JsContextFacade {
    type JsRuntimeFacadeType: JsRuntimeFacade;
    type JsContextAdapterType: JsContextAdapter;

    fn js_install_proxy(&self, js_proxy: JsProxy);

    fn js_eval(&self, script: Script) -> Box<dyn Future<Output = Result<JsValueFacade, JsError>>>;

    fn js_loop_sync<
        R : Send + 'static,
        C: FnOnce(&<<Self as JsContextFacade>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType, &Self::JsContextAdapterType) -> R + Send + 'static,
    >(
        &self,
        consumer: C,
    ) -> R;
    fn js_loop<R : Send + 'static, C>(&self, consumer: C) -> Box<dyn Future<Output = Result<R, JsError>>>
    where
        C: FnOnce(&<<Self as JsContextFacade>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType, &Self::JsContextAdapterType) -> Result<R, JsError> + Send + 'static;
    fn js_loop_void<C>(&self, consumer: C) where C: FnOnce(&<<Self as JsContextFacade>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType, &Self::JsContextAdapterType) + Send + 'static;
}

pub struct JsValueFacade {}

impl JsValueFacade {
    pub fn from_js_value_adapter<C: JsContextAdapter, V: JsValueAdapter>(
        _ctx: &C,
        _value: &V,
    ) -> JsValueFacade {
        unimplemented!()
    }
}
