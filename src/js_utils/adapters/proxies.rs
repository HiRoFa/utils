use crate::js_utils::adapters::{JsRealmAdapter, JsValueAdapter};
use crate::js_utils::facades::JsValueType;
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
) -> Result<Box<<R as JsRealmAdapter>::JsValueAdapterType>, JsError>;
pub type JsStaticMethod<R> =
    dyn Fn(
        &<R as JsRealmAdapter>::JsRuntimeAdapterType,
        &R,
        &[<R as JsRealmAdapter>::JsValueAdapterType],
    ) -> Result<Box<<R as JsRealmAdapter>::JsValueAdapterType>, JsError>;

pub type JsFinalizer<R> = dyn Fn(&R, &JsProxyInstanceId);
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
    pub event_handlers: HashMap<String, Vec<Box<R::JsValueAdapterType>>>,
    pub finalizer: Option<Box<JsFinalizer<R>>>,
}

impl<R: JsRealmAdapter> JsProxy<R> {
    pub fn set_constructor<C>(&mut self, constructor: C)
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
    }
    pub fn set_finalizer<F>(&mut self, finalizer: F)
    where
        F: Fn(&R, &JsProxyInstanceId) + 'static,
    {
        assert!(self.finalizer.is_none());
        self.finalizer.replace(Box::new(finalizer));
    }
    pub fn add_method<M>(&mut self, name: &'static str, method: M)
    where
        M: Fn(
                &R::JsRuntimeAdapterType,
                &R,
                &JsProxyInstanceId,
                &[R::JsValueAdapterType],
            ) -> Result<Box<<R as JsRealmAdapter>::JsValueAdapterType>, JsError>
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
    pub fn add_getter_setter<G, S>(&mut self, name: &'static str, getter: G, setter: S)
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
    }
    pub fn add_static_method<M>(&mut self, name: &'static str, method: M)
    where
        M: Fn(
                &R::JsRuntimeAdapterType,
                &R,
                &[R::JsValueAdapterType],
            ) -> Result<Box<<R as JsRealmAdapter>::JsValueAdapterType>, JsError>
            + 'static,
    {
        assert!(!self.static_members.contains_key(name));
        self.static_members.insert(
            name,
            JsProxyStaticMember::StaticMethod {
                method: Box::new(method),
            },
        );
    }
    pub fn add_static_getter<G, S>(&mut self, name: &'static str, getter: G)
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
    pub fn add_static_getter_setter<G, S>(&mut self, name: &'static str, getter: G, setter: S)
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
    }
    pub fn add_event_handler(&mut self, event_id: &str, handler: R::JsValueAdapterType) {
        //
        assert!(handler.js_get_type() == JsValueType::Function);
        let mut vec_opt = self.event_handlers.get_mut(event_id);
        if vec_opt.is_none() {
            self.event_handlers.insert(event_id.to_string(), vec![]);
            vec_opt = self.event_handlers.get_mut(event_id);
            debug_assert!(vec_opt.is_some());
        }

        let vec = vec_opt.unwrap();

        vec.push(Box::new(handler));
    }
    pub fn remove_event_handler(&mut self, event_id: &str, handler: R::JsValueAdapterType) {
        //
        assert!(handler.js_get_type() == JsValueType::Function);
        let mut vec_opt = self.event_handlers.get_mut(event_id);
        if vec_opt.is_none() {
            self.event_handlers.insert(event_id.to_string(), vec![]);
            vec_opt = self.event_handlers.get_mut(event_id);
            debug_assert!(vec_opt.is_some());
        }

        let vec = vec_opt.unwrap();
        if let Some(index) = vec.iter().position(|r| r.as_ref() == &handler) {
            vec.remove(index);
        }
    }
    pub fn invoke_event(
        &self,
        realm: &R,
        event_id: &str,
        args: &[R::JsValueAdapterType],
    ) -> Result<(), JsError> {
        //
        if let Some(vec) = self.event_handlers.get(event_id) {
            for handler in vec {
                //
                let h = handler.as_ref();
                realm.js_function_invoke(None, h, args)?;
            }
        }
        Ok(())
    }
}

pub type JsProxyInstanceId = usize;
