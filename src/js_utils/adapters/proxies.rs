use crate::js_utils::adapters::{JsRuntimeAdapter, JsValueAdapter};
use crate::js_utils::JsError;
use std::collections::HashMap;

pub type JsProxyConstructor<R> =
    dyn Fn(JsProxyHandle, &[dyn JsValueAdapter<JsRuntimeAdapterType = R>]) -> Result<(), JsError>;
pub type JsMethod<R> = dyn Fn(
    &JsProxyHandle,
    &[dyn JsValueAdapter<JsRuntimeAdapterType = R>],
)
    -> Result<Box<dyn JsValueAdapter<JsRuntimeAdapterType = R>>, JsError>;

pub type JsGetter<R> =
    dyn Fn(&JsProxyHandle) -> Result<Box<dyn JsValueAdapter<JsRuntimeAdapterType = R>>, JsError>;
pub type JsSetter<R> =
    dyn Fn(&JsProxyHandle, dyn JsValueAdapter<JsRuntimeAdapterType = R>) -> Result<(), JsError>;

pub enum JsProxyMember<R: JsRuntimeAdapter> {
    Method {
        method: Box<JsMethod<R>>,
    },
    GetterSetter {
        get: Box<JsGetter<R>>,
        set: Box<JsSetter<R>>,
    },
}

pub struct JsProxy<R: JsRuntimeAdapter> {
    _constructor_opt: Option<Box<JsProxyConstructor<R>>>,
    _members: HashMap<&'static str, JsProxyMember<R>>,
    _static_members: HashMap<&'static str, JsProxyMember<R>>, // enum
    _event_handlers: HashMap<&'static str, Vec<Box<dyn JsValueAdapter<JsRuntimeAdapterType = R>>>>,
}

impl<R: JsRuntimeAdapter> JsProxy<R> {}

pub type JsProxyHandle = usize;
