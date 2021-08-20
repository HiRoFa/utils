use crate::js_utils::adapters::proxies::{JsProxy, JsProxyInstanceId};
use crate::js_utils::facades::values::{
    CachedJsArrayRef, CachedJsFunctionRef, CachedJsObjectRef, CachedJsPromiseRef, JsValueFacade,
};
use crate::js_utils::facades::{JsRuntimeFacade, JsValueType};
use crate::js_utils::{JsError, Script};
use std::sync::Weak;

pub mod proxies;

pub trait JsRuntimeAdapter {
    type JsRealmAdapterType: JsRealmAdapter + 'static;
    type JsRuntimeFacadeType: JsRuntimeFacade;

    fn js_load_module_script(&self, ref_path: &str, path: &str) -> Option<Script>;

    fn js_create_realm(&self, id: &str) -> Result<&Self::JsRealmAdapterType, JsError>;
    fn js_get_realm(&self, id: &str) -> Option<&Self::JsRealmAdapterType>;
    fn js_get_main_realm(&self) -> &Self::JsRealmAdapterType;
    fn js_add_realm_init_hook<H>(&self, hook: H) -> Result<(), JsError>
    where
        H: Fn(&Self, &Self::JsRealmAdapterType) -> Result<(), JsError> + 'static;
}

pub trait JsRealmAdapter {
    type JsRuntimeAdapterType: JsRuntimeAdapter;
    type JsValueAdapterType: JsValueAdapter + Clone + PartialEq;
    type JsPromiseAdapterType: JsPromiseAdapter + Clone;

    fn js_get_realm_id(&self) -> &str;

    fn js_get_runtime_facade_inner(
        &self,
    ) -> Weak<<<<Self as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType>;

    fn js_get_script_or_module_name(&self) -> Result<String, JsError>;

    fn to_js_value_facade(
        &self,
        js_value: &Self::JsValueAdapterType,
    ) -> Result<JsValueFacade, JsError>
    where
        Self: Sized + 'static,
    {
        let res: JsValueFacade = match js_value.js_get_type() {
            JsValueType::I32 => JsValueFacade::I32 {
                val: js_value.js_to_i32(),
            },
            JsValueType::F64 => JsValueFacade::F64 {
                val: js_value.js_to_f64(),
            },
            JsValueType::String => JsValueFacade::String {
                val: js_value.js_to_string()?,
            },
            JsValueType::Boolean => JsValueFacade::Boolean {
                val: js_value.js_to_bool(),
            },
            JsValueType::Object => JsValueFacade::JsObject {
                cached_object: CachedJsObjectRef::new(self, js_value),
            },
            JsValueType::Function => JsValueFacade::JsFunction {
                cached_function: CachedJsFunctionRef {
                    cached_object: CachedJsObjectRef::new(self, js_value),
                },
            },
            JsValueType::BigInt => {
                todo!();
            }
            JsValueType::Promise => JsValueFacade::JsPromise {
                cached_promise: CachedJsPromiseRef {
                    cached_object: CachedJsObjectRef::new(self, js_value),
                },
            },
            JsValueType::Date => {
                todo!();
            }
            JsValueType::Null => JsValueFacade::Null,
            JsValueType::Undefined => JsValueFacade::Undefined,

            JsValueType::Array => JsValueFacade::JsArray {
                cached_array: CachedJsArrayRef {
                    cached_object: CachedJsObjectRef::new(self, js_value),
                },
            },
        };
        Ok(res)
    }

    fn from_js_value_facade(
        &self,
        value_facade: JsValueFacade,
    ) -> Result<Self::JsValueAdapterType, JsError>
    where
        Self: Sized + 'static,
    {
        match value_facade {
            JsValueFacade::I32 { val } => self.js_i32_create(val),
            JsValueFacade::F64 { val } => self.js_f64_create(val),
            JsValueFacade::String { val } => self.js_string_create(val.as_str()),
            JsValueFacade::Boolean { val } => self.js_boolean_create(val),
            JsValueFacade::JsObject { cached_object } => {
                // todo check realm (else copy? or error?)
                self.js_cache_with(cached_object.id, |obj| Ok(obj.clone()))
            }
            JsValueFacade::JsPromise { cached_promise } => {
                // todo check realm (else copy? or error?)
                self.js_cache_with(cached_promise.cached_object.id, |obj| Ok(obj.clone()))
            }
            JsValueFacade::JsArray { cached_array } => {
                // todo check realm (else copy? or error?)
                self.js_cache_with(cached_array.cached_object.id, |obj| Ok(obj.clone()))
            }
            JsValueFacade::JsFunction { cached_function } => {
                // todo check realm (else copy? or error?)
                self.js_cache_with(cached_function.cached_object.id, |obj| Ok(obj.clone()))
            }
            JsValueFacade::Object { val } => {
                let obj = self.js_object_create()?;
                for entry in val {
                    let prop = self.from_js_value_facade(entry.1)?;
                    self.js_object_set_property(&obj, entry.0.as_str(), &prop)?;
                }
                Ok(obj)
            }
            JsValueFacade::Array { val } => {
                let obj = self.js_array_create()?;
                for (x, entry) in val.into_iter().enumerate() {
                    let prop = self.from_js_value_facade(entry)?;
                    self.js_array_set_element(&obj, x as u32, &prop)?;
                }
                Ok(obj)
            }
            JsValueFacade::Promise { resolve_handle: _ } => {
                let _prom = self.js_promise_create()?;
                // todo.. give promise a resolvablefut?
                // .await fut here in helper thread_pool and resolve or reject prom?
                // add prom to thread_local AutoIdMap?
                //Ok(prom.js_promise_get_value())
                self.js_null_create()
            }
            JsValueFacade::Function {
                name,
                arg_count,
                func,
            } => {
                //

                self.js_function_create(
                    name.as_str(),
                    move |realm, _this, args| {
                        let mut esvf_args = vec![];
                        for arg in args {
                            esvf_args.push(realm.to_js_value_facade(arg)?);
                        }
                        let esvf_res: Result<JsValueFacade, JsError> = func(esvf_args.as_slice());

                        match esvf_res {
                            //
                            Ok(jsvf) => realm.from_js_value_facade(jsvf),
                            Err(err) => Err(err),
                        }
                    },
                    arg_count,
                )
            }
            JsValueFacade::Null => self.js_null_create(),
            JsValueFacade::Undefined => self.js_undefined_create(),
        }
    }

    fn js_eval(&self, script: Script) -> Result<Self::JsValueAdapterType, JsError>;

    fn js_proxy_install(
        &self,
        proxy: JsProxy<Self>,
        add_global_var: bool,
    ) -> Result<Self::JsValueAdapterType, JsError>
    where
        Self: Sized;
    fn js_proxy_instantiate(
        &self,
        namespace: &[&str],
        class_name: &str,
        arguments: &[Self::JsValueAdapterType],
    ) -> Result<(JsProxyInstanceId, Self::JsValueAdapterType), JsError>;
    fn js_proxy_dispatch_event(
        &self,
        namespace: &[&str],
        class_name: &str,
        proxy_instance_id: &JsProxyInstanceId,
        event_id: &str,
        event_obj: &Self::JsValueAdapterType,
    ) -> Result<bool, JsError>;
    fn js_proxy_dispatch_static_event(
        &self,
        namespace: &[&str],
        class_name: &str,
        event_id: &str,
        event_obj: &Self::JsValueAdapterType,
    ) -> Result<bool, JsError>;
    #[allow(clippy::type_complexity)]
    fn js_install_function(
        &self,
        namespace: &[&str],
        name: &str,
        js_function: fn(
            &Self::JsRuntimeAdapterType,
            &Self,
            &Self::JsValueAdapterType,
            &[Self::JsValueAdapterType],
        ) -> Result<Self::JsValueAdapterType, JsError>,
        arg_count: u32,
    ) -> Result<(), JsError>;
    fn js_install_closure<
        F: Fn(
                &Self::JsRuntimeAdapterType,
                &Self,
                &Self::JsValueAdapterType,
                &[Self::JsValueAdapterType],
            ) -> Result<Self::JsValueAdapterType, JsError>
            + 'static,
    >(
        &self,
        namespace: &[&str],
        name: &str,
        js_function: F,
        arg_count: u32,
    ) -> Result<(), JsError>;
    fn js_eval_module(&self, script: Script) -> Result<Self::JsValueAdapterType, JsError>;
    fn js_get_namespace(&self, namespace: &[&str]) -> Result<Self::JsValueAdapterType, JsError>;
    // function methods
    fn js_function_invoke_by_name(
        &self,
        namespace: &[&str],
        method_name: &str,
        args: &[Self::JsValueAdapterType],
    ) -> Result<Self::JsValueAdapterType, JsError>;
    fn js_function_invoke_member_by_name(
        &self,
        this_obj: &Self::JsValueAdapterType,
        method_name: &str,
        args: &[Self::JsValueAdapterType],
    ) -> Result<Self::JsValueAdapterType, JsError>;
    fn js_function_invoke(
        &self,
        this_obj: Option<&Self::JsValueAdapterType>,
        function_obj: &Self::JsValueAdapterType,
        args: &[Self::JsValueAdapterType],
    ) -> Result<Self::JsValueAdapterType, JsError>;
    fn js_function_create<
        F: Fn(
                &Self,
                &Self::JsValueAdapterType,
                &[Self::JsValueAdapterType],
            ) -> Result<Self::JsValueAdapterType, JsError>
            + 'static,
    >(
        &self,
        name: &str,
        js_function: F,
        arg_count: u32,
    ) -> Result<Self::JsValueAdapterType, JsError>;
    //object functions
    fn js_object_delete_property(
        &self,
        object: &Self::JsValueAdapterType,
        property_name: &str,
    ) -> Result<(), JsError>;
    fn js_object_set_property(
        &self,
        object: &Self::JsValueAdapterType,
        property_name: &str,
        property: &Self::JsValueAdapterType,
    ) -> Result<(), JsError>;

    fn js_object_get_property(
        &self,
        object: &Self::JsValueAdapterType,
        property_name: &str,
    ) -> Result<Self::JsValueAdapterType, JsError>;
    fn js_object_create(&self) -> Result<Self::JsValueAdapterType, JsError>;
    fn js_object_construct(
        &self,
        constructor: &Self::JsValueAdapterType,
        args: &[Self::JsValueAdapterType],
    ) -> Result<Self::JsValueAdapterType, JsError>;
    fn js_object_get_properties(
        &self,
        object: &Self::JsValueAdapterType,
    ) -> Result<Vec<String>, JsError>;
    fn js_object_traverse<F, R>(
        &self,
        object: &Self::JsValueAdapterType,
        visitor: F,
    ) -> Result<Vec<R>, JsError>
    where
        F: Fn(&str, &Self::JsValueAdapterType) -> Result<R, JsError>;
    // array functions
    fn js_array_get_element(
        &self,
        array: &Self::JsValueAdapterType,
        index: u32,
    ) -> Result<Self::JsValueAdapterType, JsError>;
    fn js_array_set_element(
        &self,
        array: &Self::JsValueAdapterType,
        index: u32,
        element: &Self::JsValueAdapterType,
    ) -> Result<(), JsError>;
    fn js_array_get_length(&self, array: &Self::JsValueAdapterType) -> Result<u32, JsError>;
    fn js_array_create(&self) -> Result<Self::JsValueAdapterType, JsError>;
    fn js_array_traverse<F, R>(
        &self,
        array: &Self::JsValueAdapterType,
        visitor: F,
    ) -> Result<Vec<R>, JsError>
    where
        F: Fn(u32, &Self::JsValueAdapterType) -> Result<R, JsError>;
    // primitives

    fn js_null_create(&self) -> Result<Self::JsValueAdapterType, JsError>;
    fn js_undefined_create(&self) -> Result<Self::JsValueAdapterType, JsError>;
    fn js_i32_create(&self, val: i32) -> Result<Self::JsValueAdapterType, JsError>;
    fn js_string_create(&self, val: &str) -> Result<Self::JsValueAdapterType, JsError>;
    fn js_boolean_create(&self, val: bool) -> Result<Self::JsValueAdapterType, JsError>;
    fn js_f64_create(&self, val: f64) -> Result<Self::JsValueAdapterType, JsError>;

    // promises
    fn js_promise_create(&self) -> Result<Box<Self::JsPromiseAdapterType>, JsError>;

    fn js_promise_add_reactions(
        &self,
        promise: &Self::JsValueAdapterType,
        then: Option<Self::JsValueAdapterType>,
        catch: Option<Self::JsValueAdapterType>,
        finally: Option<Self::JsValueAdapterType>,
    ) -> Result<(), JsError>;

    // cache
    fn js_cache_add(&self, object: &Self::JsValueAdapterType) -> i32;
    fn js_cache_dispose(&self, id: i32);
    fn js_cache_with<C, R>(&self, id: i32, consumer: C) -> R
    where
        C: FnOnce(&Self::JsValueAdapterType) -> R;
    fn js_cache_consume(&self, id: i32) -> Self::JsValueAdapterType;

    // instanceof
    fn js_instance_of(
        &self,
        object: &Self::JsValueAdapterType,
        constructor: &Self::JsValueAdapterType,
    ) -> bool;

    // json
    fn js_json_stringify(
        &self,
        object: &Self::JsValueAdapterType,
        opt_space: Option<&str>,
    ) -> Result<String, JsError>;
    fn js_json_parse(&self, json_string: &str) -> Result<Self::JsValueAdapterType, JsError>;
}

pub trait JsPromiseAdapter {
    type JsRealmAdapterType: JsRealmAdapter;
    fn js_promise_resolve(
        &self,
        realm: &Self::JsRealmAdapterType,
        resolution: &<<Self as JsPromiseAdapter>::JsRealmAdapterType as JsRealmAdapter>::JsValueAdapterType,
    ) -> Result<(), JsError>;
    fn js_promise_reject(
        &self,
        realm: &Self::JsRealmAdapterType,
        rejection: &<<Self as JsPromiseAdapter>::JsRealmAdapterType as JsRealmAdapter>::JsValueAdapterType,
    ) -> Result<(), JsError>;
    fn js_promise_get_value(
        &self,
    ) -> <<Self as JsPromiseAdapter>::JsRealmAdapterType as JsRealmAdapter>::JsValueAdapterType;
}

pub trait JsValueAdapter {
    type JsRuntimeAdapterType: JsRuntimeAdapter;

    /// js_get_type returns the rust type of the value (more extensive list than javascript typeof)
    fn js_get_type(&self) -> JsValueType;

    fn js_is_null_or_undefined(&self) -> bool {
        self.js_get_type() == JsValueType::Null || self.js_get_type() == JsValueType::Undefined
    }

    /// js_type_of returns the Javascript typeof string
    fn js_type_of(&self) -> &'static str;
    fn js_to_bool(&self) -> bool;
    fn js_to_i32(&self) -> i32;
    fn js_to_f64(&self) -> f64;
    fn js_to_string(&self) -> Result<String, JsError>;
}
