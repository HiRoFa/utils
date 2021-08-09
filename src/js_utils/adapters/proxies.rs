use crate::js_utils::adapters::{JsRuntimeAdapter, JsValueAdapter};
use crate::js_utils::JsError;
use std::collections::HashMap;

pub type JsProxyConstructor<R> = dyn Fn(
    &R,
    &<R as JsRuntimeAdapter>::JsRealmAdapterType,
    &JsProxyInstanceId,
    &[&<R as JsRuntimeAdapter>::JsValueAdapterType],
) -> Result<(), JsError>;
pub type JsMethod<R> = dyn Fn(
    &R,
    &<R as JsRuntimeAdapter>::JsRealmAdapterType,
    &JsProxyInstanceId,
    &[&<R as JsRuntimeAdapter>::JsValueAdapterType],
) -> Result<Box<<R as JsRuntimeAdapter>::JsValueAdapterType>, JsError>;
pub type JsStaticMethod<R> =
    dyn Fn(
        &R,
        &<R as JsRuntimeAdapter>::JsRealmAdapterType,
        &[&<R as JsRuntimeAdapter>::JsValueAdapterType],
    ) -> Result<Box<<R as JsRuntimeAdapter>::JsValueAdapterType>, JsError>;

pub type JsFinalizer<R> =
    dyn Fn(&R, &<R as JsRuntimeAdapter>::JsRealmAdapterType, &JsProxyInstanceId);
pub type JsGetter<R> = dyn Fn(
    &R,
    &<R as JsRuntimeAdapter>::JsRealmAdapterType,
    &JsProxyInstanceId,
) -> Result<<R as JsRuntimeAdapter>::JsValueAdapterType, JsError>;
pub type JsSetter<R> = dyn Fn(
    &R,
    &<R as JsRuntimeAdapter>::JsRealmAdapterType,
    &JsProxyInstanceId,
    &<R as JsRuntimeAdapter>::JsValueAdapterType,
) -> Result<(), JsError>;
pub type JsStaticGetter<R> = dyn Fn(
    &R,
    &<R as JsRuntimeAdapter>::JsRealmAdapterType,
)
    -> Result<<R as JsRuntimeAdapter>::JsValueAdapterType, JsError>;
pub type JsStaticSetter<R> = dyn Fn(
    &R,
    &<R as JsRuntimeAdapter>::JsRealmAdapterType,
    &<R as JsRuntimeAdapter>::JsValueAdapterType,
) -> Result<(), JsError>;

pub enum JsProxyMember<R: JsRuntimeAdapter> {
    Method {
        method: Box<JsMethod<R>>,
    },
    GetterSetter {
        get: Box<JsGetter<R>>,
        set: Box<JsSetter<R>>,
    },
}
pub enum JsStaticProxyMember<R: JsRuntimeAdapter> {
    StaticMethod {
        method: Box<JsStaticMethod<R>>,
    },
    StaticGetterSetter {
        get: Box<JsStaticGetter<R>>,
        set: Box<JsStaticSetter<R>>,
    },
}

pub struct JsProxy<R: JsRuntimeAdapter> {
    pub name: &'static str,
    pub namespace: &'static [&'static str],
    constructor: Option<Box<JsProxyConstructor<R>>>,
    members: HashMap<&'static str, JsProxyMember<R>>,
    _static_members: HashMap<&'static str, JsStaticProxyMember<R>>, // enum
    _event_handlers: HashMap<&'static str, Vec<Box<dyn JsValueAdapter<JsRuntimeAdapterType = R>>>>,
    finalizer: Option<Box<JsFinalizer<R>>>,
}

impl<R: JsRuntimeAdapter> JsProxy<R> {
    pub fn set_constructor<C>(&mut self, constructor: C)
    where
        C: Fn(
                &R,
                &R::JsRealmAdapterType,
                &JsProxyInstanceId,
                &[&R::JsValueAdapterType],
            ) -> Result<(), JsError>
            + 'static,
    {
        assert!(self.constructor.is_none());
        self.constructor.replace(Box::new(constructor));
    }
    pub fn set_finalizer<F>(&mut self, finalizer: F)
    where
        F: Fn(&R, &R::JsRealmAdapterType, &JsProxyInstanceId) + 'static,
    {
        assert!(self.finalizer.is_none());
        self.finalizer.replace(Box::new(finalizer));
    }
    pub fn add_method<M>(&mut self, name: &'static str, method: M)
    where
        M: Fn(
                &R,
                &R::JsRealmAdapterType,
                &JsProxyInstanceId,
                &[&R::JsValueAdapterType],
            ) -> Result<Box<<R as JsRuntimeAdapter>::JsValueAdapterType>, JsError>
            + 'static,
    {
        assert!(!self.members.contains_key(name));
        self.members.insert(
            name,
            JsProxyMember::Method {
                method: Box::new(method),
            },
        );
    }
    pub fn add_getter<G, S>(&mut self, name: &'static str, getter: G)
    where
        G: Fn(
                &R,
                &<R as JsRuntimeAdapter>::JsRealmAdapterType,
                &JsProxyInstanceId,
            ) -> Result<<R as JsRuntimeAdapter>::JsValueAdapterType, JsError>
            + 'static,
    {
        self.add_getter_setter(name, getter, |_rt, _realm, _id, _val| {
            Err(JsError::new_str("Cannot update read-only member"))
        })
    }
    pub fn add_getter_setter<G, S>(&mut self, name: &'static str, getter: G, setter: S)
    where
        G: Fn(
                &R,
                &<R as JsRuntimeAdapter>::JsRealmAdapterType,
                &JsProxyInstanceId,
            ) -> Result<<R as JsRuntimeAdapter>::JsValueAdapterType, JsError>
            + 'static,
        S: Fn(
                &R,
                &<R as JsRuntimeAdapter>::JsRealmAdapterType,
                &JsProxyInstanceId,
                &<R as JsRuntimeAdapter>::JsValueAdapterType,
            ) -> Result<(), JsError>
            + 'static,
    {
        assert!(!self.members.contains_key(name));
        self.members.insert(
            name,
            JsProxyMember::GetterSetter {
                get: Box::new(getter),
                set: Box::new(setter),
            },
        );
    }
}

pub type JsProxyInstanceId = usize;
