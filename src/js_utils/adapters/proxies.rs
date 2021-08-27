use crate::js_utils::adapters::JsRealmAdapter;
use crate::js_utils::JsError;
use std::collections::HashMap;

pub type JsProxyConstructor<R> = dyn Fn(
    &<R as JsRealmAdapter>::JsRuntimeAdapterType,
    &R,
    &JsProxyInstanceId,
    &[<R as JsRealmAdapter>::JsValueAdapterType],
) -> Result<(), JsError>;
pub type JsMethod<R> = dyn Fn(
    &<R as JsRealmAdapter>::JsRuntimeAdapterType,
    &R,
    &JsProxyInstanceId,
    &[<R as JsRealmAdapter>::JsValueAdapterType],
) -> Result<<R as JsRealmAdapter>::JsValueAdapterType, JsError>;
pub type JsStaticMethod<R> = dyn Fn(
    &<R as JsRealmAdapter>::JsRuntimeAdapterType,
    &R,
    &[<R as JsRealmAdapter>::JsValueAdapterType],
) -> Result<<R as JsRealmAdapter>::JsValueAdapterType, JsError>;

pub type JsFinalizer<R> =
    dyn Fn(&<R as JsRealmAdapter>::JsRuntimeAdapterType, &R, &JsProxyInstanceId);
pub type JsGetter<R> = dyn Fn(
    &<R as JsRealmAdapter>::JsRuntimeAdapterType,
    &R,
    &JsProxyInstanceId,
) -> Result<<R as JsRealmAdapter>::JsValueAdapterType, JsError>;
pub type JsSetter<R> = dyn Fn(
    &<R as JsRealmAdapter>::JsRuntimeAdapterType,
    &R,
    &JsProxyInstanceId,
    &<R as JsRealmAdapter>::JsValueAdapterType,
) -> Result<(), JsError>;
pub type JsStaticGetter<R> = dyn Fn(
    &<R as JsRealmAdapter>::JsRuntimeAdapterType,
    &R,
) -> Result<<R as JsRealmAdapter>::JsValueAdapterType, JsError>;
pub type JsStaticSetter<R> = dyn Fn(
    &<R as JsRealmAdapter>::JsRuntimeAdapterType,
    &R,
    &<R as JsRealmAdapter>::JsValueAdapterType,
) -> Result<(), JsError>;

pub enum JsProxyMember<R: JsRealmAdapter> {
    Method {
        method: Box<JsMethod<R>>,
    },
    GetterSetter {
        get: Box<JsGetter<R>>,
        set: Box<JsSetter<R>>,
    },
}
pub enum JsProxyStaticMember<R: JsRealmAdapter> {
    StaticMethod {
        method: Box<JsStaticMethod<R>>,
    },
    StaticGetterSetter {
        get: Box<JsStaticGetter<R>>,
        set: Box<JsStaticSetter<R>>,
    },
}

pub struct JsProxy<R: JsRealmAdapter> {
    pub name: &'static str,
    pub namespace: &'static [&'static str],
    pub constructor: Option<Box<JsProxyConstructor<R>>>,
    pub members: HashMap<&'static str, JsProxyMember<R>>,
    pub static_members: HashMap<&'static str, JsProxyStaticMember<R>>, // enum
    pub finalizer: Option<Box<JsFinalizer<R>>>,
    pub event_target: bool,
    pub static_event_target: bool,
}

impl<R: JsRealmAdapter> JsProxy<R> {
    pub fn new(namespace: &'static [&'static str], name: &'static str) -> Self {
        Self {
            name,
            namespace,
            constructor: None,
            members: Default::default(),
            static_members: Default::default(),
            finalizer: None,
            event_target: false,
            static_event_target: false,
        }
    }
    pub fn set_constructor<C>(mut self, constructor: C) -> Self
    where
        C: Fn(
                &R::JsRuntimeAdapterType,
                &R,
                &JsProxyInstanceId,
                &[R::JsValueAdapterType],
            ) -> Result<(), JsError>
            + 'static,
    {
        assert!(self.constructor.is_none());
        self.constructor.replace(Box::new(constructor));
        self
    }
    pub fn set_finalizer<F>(mut self, finalizer: F) -> Self
    where
        F: Fn(&<R as JsRealmAdapter>::JsRuntimeAdapterType, &R, &JsProxyInstanceId) + 'static,
    {
        assert!(self.finalizer.is_none());
        self.finalizer.replace(Box::new(finalizer));
        self
    }
    pub fn add_method<M>(mut self, name: &'static str, method: M) -> Self
    where
        M: Fn(
                &R::JsRuntimeAdapterType,
                &R,
                &JsProxyInstanceId,
                &[R::JsValueAdapterType],
            ) -> Result<<R as JsRealmAdapter>::JsValueAdapterType, JsError>
            + 'static,
    {
        assert!(!self.members.contains_key(name));
        self.members.insert(
            name,
            JsProxyMember::Method {
                method: Box::new(method),
            },
        );
        self
    }
    pub fn add_getter<G, S>(self, name: &'static str, getter: G) -> Self
    where
        G: Fn(
                &R::JsRuntimeAdapterType,
                &R,
                &JsProxyInstanceId,
            ) -> Result<<R as JsRealmAdapter>::JsValueAdapterType, JsError>
            + 'static,
    {
        self.add_getter_setter(name, getter, |_rt, _realm, _id, _val| {
            Err(JsError::new_str("Cannot update read-only member"))
        })
    }
    pub fn add_getter_setter<G, S>(mut self, name: &'static str, getter: G, setter: S) -> Self
    where
        G: Fn(
                &R::JsRuntimeAdapterType,
                &R,
                &JsProxyInstanceId,
            ) -> Result<<R as JsRealmAdapter>::JsValueAdapterType, JsError>
            + 'static,
        S: Fn(
                &R::JsRuntimeAdapterType,
                &R,
                &JsProxyInstanceId,
                &<R as JsRealmAdapter>::JsValueAdapterType,
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
        self
    }
    pub fn add_static_method<M>(mut self, name: &'static str, method: M) -> Self
    where
        M: Fn(
                &R::JsRuntimeAdapterType,
                &R,
                &[R::JsValueAdapterType],
            ) -> Result<<R as JsRealmAdapter>::JsValueAdapterType, JsError>
            + 'static,
    {
        assert!(!self.static_members.contains_key(name));
        self.static_members.insert(
            name,
            JsProxyStaticMember::StaticMethod {
                method: Box::new(method),
            },
        );
        self
    }
    pub fn add_static_getter<G, S>(self, name: &'static str, getter: G) -> Self
    where
        G: Fn(
                &R::JsRuntimeAdapterType,
                &R,
            ) -> Result<<R as JsRealmAdapter>::JsValueAdapterType, JsError>
            + 'static,
    {
        self.add_static_getter_setter(name, getter, |_rt, _realm, _val| {
            Err(JsError::new_str("Cannot update read-only member"))
        })
    }
    pub fn add_static_getter_setter<G, S>(
        mut self,
        name: &'static str,
        getter: G,
        setter: S,
    ) -> Self
    where
        G: Fn(
                &R::JsRuntimeAdapterType,
                &R,
            ) -> Result<<R as JsRealmAdapter>::JsValueAdapterType, JsError>
            + 'static,
        S: Fn(
                &R::JsRuntimeAdapterType,
                &R,
                &<R as JsRealmAdapter>::JsValueAdapterType,
            ) -> Result<(), JsError>
            + 'static,
    {
        assert!(!self.static_members.contains_key(name));
        self.static_members.insert(
            name,
            JsProxyStaticMember::StaticGetterSetter {
                get: Box::new(getter),
                set: Box::new(setter),
            },
        );
        self
    }
    pub fn set_event_target(mut self, is_event_target: bool) -> Self {
        self.event_target = is_event_target;
        self
    }
    pub fn set_static_event_target(mut self, is_static_event_target: bool) -> Self {
        self.static_event_target = is_static_event_target;
        self
    }
}

pub type JsProxyInstanceId = usize;
