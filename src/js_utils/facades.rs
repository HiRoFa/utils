use crate::js_utils::adapters::{JsContextAdapter, JsValueAdapter};

pub struct JsProxy {}

pub trait JsRuntimeFacade {}

pub trait JsContextFacade {
    fn install_proxy(&self, js_proxy: JsProxy);
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
