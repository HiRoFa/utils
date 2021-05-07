use crate::js_utils::adapters::{JsContextAdapter, JsValueAdapter};

pub trait JsRuntimeFacade {}

pub trait JsContextFacade {}

pub struct JsValueFacade {}

impl JsValueFacade {
    pub fn from_js_value_adapter<C: JsContextAdapter + Sized, V: JsValueAdapter + Sized>(
        _ctx: C,
        _value: V,
    ) -> JsValueFacade {
        unimplemented!()
    }
}
