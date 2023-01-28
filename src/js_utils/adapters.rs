use crate::js_utils::adapters::proxies::{JsProxy, JsProxyInstanceId};
use crate::js_utils::facades::values::{
    CachedJsArrayRef, CachedJsFunctionRef, CachedJsObjectRef, CachedJsPromiseRef, JsValueFacade,
    TypedArrayType,
};
use crate::js_utils::facades::{JsRuntimeFacade, JsRuntimeFacadeInner, JsValueType};
use crate::js_utils::{JsError, Script};
use futures::Future;
use serde_json::Value;
use std::sync::Weak;
use string_cache::DefaultAtom;

pub mod promises;
pub mod proxies;

pub trait JsRuntimeAdapter {
    type JsRealmAdapterType: JsRealmAdapter + 'static;
    type JsRuntimeFacadeType: JsRuntimeFacade;

    /// this method can be used to load the script code for a module (via any ScriptModuleLoader)
    fn js_load_module_script(&self, ref_path: &str, path: &str) -> Option<Script>;

    /// create a new Realm
    fn js_create_realm(&self, id: &str) -> Result<&Self::JsRealmAdapterType, JsError>;

    /// drop a Realm, please note that the Realm might not really be dropped until all promises have fulfilled
    /// please note that this should not be able to remove the main realm
    fn js_remove_realm(&self, id: &str);

    /// get a Realm, if the realm does not exists None will be returned
    fn js_get_realm(&self, id: &str) -> Option<&Self::JsRealmAdapterType>;

    /// get the main realm, this realm is always present and cannot be removed
    fn js_get_main_realm(&self) -> &Self::JsRealmAdapterType;

    /// add a hook to add custom code for when a Realm is initialized
    /// when adding a hook it will also be called for existing realms including the main realm
    fn js_add_realm_init_hook<H>(&self, hook: H) -> Result<(), JsError>
    where
        H: Fn(&Self, &Self::JsRealmAdapterType) -> Result<(), JsError> + 'static;
}

pub trait JsRealmAdapter {
    type JsRuntimeAdapterType: JsRuntimeAdapter;
    type JsValueAdapterType: JsValueAdapter + Clone + PartialEq;

    /// get the id of this Realm
    fn js_get_realm_id(&self) -> &str;

    /// get a Weak reference to the JsRuntimeFacadeInner
    /// this can be used to e.g. add a job to that JsRuntimeFacadeInner for resolving promises async
    fn js_get_runtime_facade_inner(
        &self,
    ) -> Weak<<<<Self as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType> where Self: 'static;

    /// get the name of the current module or script
    /// for modules loaded via the HttpModuleLoader these will have a https:// prefix
    /// modules loaded via the FileSystemModuleLoader will have a file:// prefix
    fn js_get_script_or_module_name(&self) -> Result<String, JsError>;

    /// convert a JSValueAdapter to a Send able JSValueFacade
    /// you'll need this to pass values out of the JSRuntimeAdapter's worker thread
    /// the other way around you'll need from_js_value_facade to move values into the worker thread
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
                val: DefaultAtom::from(js_value.js_to_string()?),
            },
            JsValueType::Boolean => JsValueFacade::Boolean {
                val: js_value.js_to_bool(),
            },
            JsValueType::Object => {
                if js_value.js_is_typed_array() {
                    // todo TypedArray as JsValueType?
                    // passing a typedarray out of the worker thread is sketchy because you either copy the buffer like we do here, or you detach the buffer effectively destroying the jsvalue
                    // you should be better of optimizing this in native methods
                    JsValueFacade::TypedArray {
                        buffer: self.js_typed_array_copy_buffer(js_value)?,
                        array_type: TypedArrayType::Uint8,
                    }
                } else {
                    JsValueFacade::JsObject {
                        cached_object: CachedJsObjectRef::new(self, js_value),
                    }
                }
            }
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
            JsValueType::Error => {
                let name = self
                    .js_object_get_property(js_value, "name")?
                    .js_to_string()?;
                let message = self
                    .js_object_get_property(js_value, "message")?
                    .js_to_string()?;
                let stack = self
                    .js_object_get_property(js_value, "stack")?
                    .js_to_string()?;
                JsValueFacade::JsError {
                    val: JsError::new(name, message, stack),
                }
            }
        };
        Ok(res)
    }

    /// convert a JSValueFacade into a JSValueAdapter
    /// you need this to move values into the worker thread from a different thread (JSValueAdapter cannot leave the worker thread)
    #[allow(clippy::wrong_self_convention)]
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
            JsValueFacade::String { val } => self.js_string_create(&val),
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
            JsValueFacade::Promise { producer } => {
                let producer = &mut *producer.lock().unwrap();
                if producer.is_some() {
                    self.js_promise_create_resolving_async(
                        producer.take().unwrap(),
                        |realm, jsvf| realm.from_js_value_facade(jsvf),
                    )
                } else {
                    self.js_null_create()
                }
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
            JsValueFacade::JsError { val } => {
                self.js_error_create(val.get_name(), val.get_message(), val.get_stack())
            }
            JsValueFacade::ProxyInstance {
                instance_id,
                namespace,
                class_name,
            } => self.js_proxy_instantiate_with_id(namespace, class_name, instance_id),
            JsValueFacade::TypedArray { buffer, array_type } => match array_type {
                TypedArrayType::Uint8 => self.js_typed_array_uint8_create(buffer),
            },
            JsValueFacade::JsonStr { json } => self.js_json_parse(json.as_str()),
            JsValueFacade::SerdeValue { value } => self.serde_value_to_js_value_adapter(value),
        }
    }

    fn serde_value_to_js_value_adapter(
        &self,
        value: Value,
    ) -> Result<Self::JsValueAdapterType, JsError> {
        match value {
            Value::Null => self.js_null_create(),
            Value::Bool(b) => self.js_boolean_create(b),
            Value::Number(n) => {
                if n.is_i64() {
                    let i = n.as_i64().unwrap();
                    if i <= i32::MAX as i64 {
                        self.js_i32_create(i as i32)
                    } else {
                        self.js_f64_create(i as f64)
                    }
                } else if n.is_u64() {
                    let i = n.as_u64().unwrap();
                    if i <= i32::MAX as u64 {
                        self.js_i32_create(i as i32)
                    } else {
                        self.js_f64_create(i as f64)
                    }
                } else {
                    // f64
                    let i = n.as_f64().unwrap();
                    self.js_f64_create(i)
                }
            }
            Value::String(s) => self.js_string_create(s.as_str()),
            Value::Array(a) => {
                let arr = self.js_array_create()?;
                for (x, aval) in (0_u32..).zip(a.into_iter()) {
                    let entry = self.serde_value_to_js_value_adapter(aval)?;
                    self.js_array_set_element(&arr, x, &entry)?;
                }
                Ok(arr)
            }
            Value::Object(o) => {
                let obj = self.js_object_create()?;
                for oval in o {
                    let entry = self.serde_value_to_js_value_adapter(oval.1)?;
                    self.js_object_set_property(&obj, oval.0.as_str(), &entry)?;
                }
                Ok(obj)
            }
        }
    }

    /// eval a script
    /// please only use this for debugging/testing purposes
    /// although most JS engines will return values as if really calling a eval() method some may not (e.g. StarLight requires a return statement)
    fn js_eval(&self, script: Script) -> Result<Self::JsValueAdapterType, JsError>;

    /// install a JsProxy into this Realm
    fn js_proxy_install(
        &self,
        proxy: JsProxy<Self>,
        add_global_var: bool,
    ) -> Result<Self::JsValueAdapterType, JsError>
    where
        Self: Sized;

    /// instantiate a JsProxy instance value based on an instance_id
    /// this instance_id will be passed to the JsProxy's member methods
    fn js_proxy_instantiate_with_id(
        &self,
        namespace: &[&str],
        class_name: &str,
        instance_id: JsProxyInstanceId,
    ) -> Result<Self::JsValueAdapterType, JsError>;

    /// instantiate a JsProxy instance value
    /// an instance_id will be generated and returned
    /// this instance_id will be passed to the JsProxy's member methods
    fn js_proxy_instantiate(
        &self,
        namespace: &[&str],
        class_name: &str,
        arguments: &[Self::JsValueAdapterType],
    ) -> Result<(JsProxyInstanceId, Self::JsValueAdapterType), JsError>;

    /// dispatch an event to a JsProxy instance
    fn js_proxy_dispatch_event(
        &self,
        namespace: &[&str],
        class_name: &str,
        proxy_instance_id: &JsProxyInstanceId,
        event_id: &str,
        event_obj: &Self::JsValueAdapterType,
    ) -> Result<bool, JsError>;

    /// dispatch a static event to a JsProxy
    fn js_proxy_dispatch_static_event(
        &self,
        namespace: &[&str],
        class_name: &str,
        event_id: &str,
        event_obj: &Self::JsValueAdapterType,
    ) -> Result<bool, JsError>;

    /// install a function into this realm
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

    /// install a function into this realm based on a closure
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

    /// evaluate a module
    fn js_eval_module(&self, script: Script) -> Result<Self::JsValueAdapterType, JsError>;

    /// get the global object
    fn js_get_global(&self) -> Result<Self::JsValueAdapterType, JsError>;

    /// get a namespace obj, when not present parts will be created
    fn js_get_namespace(&self, namespace: &[&str]) -> Result<Self::JsValueAdapterType, JsError>;
    // function methods
    /// invoke a function by name
    fn js_function_invoke_by_name(
        &self,
        namespace: &[&str],
        method_name: &str,
        args: &[Self::JsValueAdapterType],
    ) -> Result<Self::JsValueAdapterType, JsError>;

    /// invoke a member function of an object by name
    fn js_function_invoke_member_by_name(
        &self,
        this_obj: &Self::JsValueAdapterType,
        method_name: &str,
        args: &[Self::JsValueAdapterType],
    ) -> Result<Self::JsValueAdapterType, JsError>;

    /// invoke a Funtion
    fn js_function_invoke(
        &self,
        this_obj: Option<&Self::JsValueAdapterType>,
        function_obj: &Self::JsValueAdapterType,
        args: &[&Self::JsValueAdapterType],
    ) -> Result<Self::JsValueAdapterType, JsError>;

    /// create a new Function
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
    fn js_function_create_async<R, F>(
        &self,
        name: &str,
        js_function: F,
        arg_count: u32,
    ) -> Result<Self::JsValueAdapterType, JsError>
    where
        Self: Sized + 'static,
        R: Future<Output = Result<JsValueFacade, JsError>> + Send + 'static,
        F: Fn(JsValueFacade, Vec<JsValueFacade>) -> R + 'static,
    {
        //
        self.js_function_create(
            name,
            move |realm, this, args| {
                let this_fac = realm.to_js_value_facade(this)?;
                let mut args_fac = vec![];
                for arg in args {
                    args_fac.push(realm.to_js_value_facade(arg)?);
                }
                let fut = js_function(this_fac, args_fac);
                realm.js_promise_create_resolving_async(async move { fut.await }, |realm, pres| {
                    //
                    realm.from_js_value_facade(pres)
                })
            },
            arg_count,
        )
    }
    //error functions
    fn js_error_create(
        &self,
        name: &str,
        message: &str,
        stack: &str,
    ) -> Result<Self::JsValueAdapterType, JsError>;
    //object functions
    /// delete a property of an Object
    fn js_object_delete_property(
        &self,
        object: &Self::JsValueAdapterType,
        property_name: &str,
    ) -> Result<(), JsError>;
    /// set a property of an Object
    fn js_object_set_property(
        &self,
        object: &Self::JsValueAdapterType,
        property_name: &str,
        property: &Self::JsValueAdapterType,
    ) -> Result<(), JsError>;
    /// get a property of an Object
    fn js_object_get_property(
        &self,
        object: &Self::JsValueAdapterType,
        property_name: &str,
    ) -> Result<Self::JsValueAdapterType, JsError>;
    /// create a new Object
    fn js_object_create(&self) -> Result<Self::JsValueAdapterType, JsError>;

    /// invoke a constructor Function to create new Object
    fn js_object_construct(
        &self,
        constructor: &Self::JsValueAdapterType,
        args: &[&Self::JsValueAdapterType],
    ) -> Result<Self::JsValueAdapterType, JsError>;

    /// get all property names of an Object
    fn js_object_get_properties(
        &self,
        object: &Self::JsValueAdapterType,
    ) -> Result<Vec<String>, JsError>;
    /// traverse all properties of an Object
    fn js_object_traverse<F, R>(
        &self,
        object: &Self::JsValueAdapterType,
        visitor: F,
    ) -> Result<Vec<R>, JsError>
    where
        F: Fn(&str, &Self::JsValueAdapterType) -> Result<R, JsError>;
    /// traverse all properties of an Object with a FnMut
    fn js_object_traverse_mut<F>(
        &self,
        object: &Self::JsValueAdapterType,
        visitor: F,
    ) -> Result<(), JsError>
    where
        F: FnMut(&str, &Self::JsValueAdapterType) -> Result<(), JsError>;
    // array functions
    /// get an element of an Array
    fn js_array_get_element(
        &self,
        array: &Self::JsValueAdapterType,
        index: u32,
    ) -> Result<Self::JsValueAdapterType, JsError>;
    /// set an element of an Array
    fn js_array_set_element(
        &self,
        array: &Self::JsValueAdapterType,
        index: u32,
        element: &Self::JsValueAdapterType,
    ) -> Result<(), JsError>;
    /// push an element into an Array
    fn js_array_push_element(
        &self,
        array: &Self::JsValueAdapterType,
        element: &Self::JsValueAdapterType,
    ) -> Result<u32, JsError> {
        let push_func = self.js_object_get_property(array, "push")?;
        let res = self.js_function_invoke(Some(array), &push_func, &[element])?;
        Ok(res.js_to_i32() as u32)
    }
    /// get the length of an Array
    fn js_array_get_length(&self, array: &Self::JsValueAdapterType) -> Result<u32, JsError>;
    /// create a new Array
    fn js_array_create(&self) -> Result<Self::JsValueAdapterType, JsError>;
    /// traverse all objects in an Array
    fn js_array_traverse<F, R>(
        &self,
        array: &Self::JsValueAdapterType,
        visitor: F,
    ) -> Result<Vec<R>, JsError>
    where
        F: Fn(u32, &Self::JsValueAdapterType) -> Result<R, JsError>;
    /// traverse all objects in an Array
    fn js_array_traverse_mut<F>(
        &self,
        array: &Self::JsValueAdapterType,
        visitor: F,
    ) -> Result<(), JsError>
    where
        F: FnMut(u32, &Self::JsValueAdapterType) -> Result<(), JsError>;
    // primitives

    /// create a null value
    fn js_null_create(&self) -> Result<Self::JsValueAdapterType, JsError>;
    /// create an undefined value
    fn js_undefined_create(&self) -> Result<Self::JsValueAdapterType, JsError>;
    /// create a Number value based on an i32
    fn js_i32_create(&self, val: i32) -> Result<Self::JsValueAdapterType, JsError>;
    /// create a String value
    fn js_string_create(&self, val: &str) -> Result<Self::JsValueAdapterType, JsError>;
    /// create a Boolean value
    fn js_boolean_create(&self, val: bool) -> Result<Self::JsValueAdapterType, JsError>;
    /// create a Number value based on an f64
    fn js_f64_create(&self, val: f64) -> Result<Self::JsValueAdapterType, JsError>;

    // promises
    /// create a new Promise
    /// this returns JsPromiseAdapter which can be turned into a JsValueAdapter but can also be used to fulfill the promise
    fn js_promise_create(&self) -> Result<Box<dyn JsPromiseAdapter<Self>>, JsError>;
    /// create a new Promise with a Future which will run async and then resolve or reject the promise
    /// the mapper is used to convert the result of the future into a JSValueAdapter
    fn js_promise_create_resolving_async<P, R: Send + 'static, M>(
        &self,
        producer: P,
        mapper: M,
    ) -> Result<Self::JsValueAdapterType, JsError>
    where
        P: Future<Output = Result<R, JsError>> + Send + 'static,
        M: FnOnce(&<<<<<<Self as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType, R) -> Result<<<<<<<<Self as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType as JsRealmAdapter>::JsValueAdapterType, JsError> + Send + 'static,
        Self: Sized + 'static,
    {
        crate::js_utils::adapters::promises::new_resolving_promise_async(self, producer, mapper)
    }
    /// create a new Promise with a FnOnce producer which will run async and then resolve or reject the promise
    /// the mapper is used to convert the result of the future into a JSValueAdapter
    fn js_promise_create_resolving<P, R: Send + 'static, M>(
        &self,
        producer: P,
        mapper: M,
    ) -> Result<Self::JsValueAdapterType, JsError>
        where
            P: FnOnce() -> Result<R, JsError> + Send + 'static,
            M: FnOnce(&<<<<<<Self as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType, R) -> Result<<<<<<<<Self as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType as JsRealmAdapter>::JsValueAdapterType, JsError> + Send + 'static,
            Self: Sized + 'static,
    {
        crate::js_utils::adapters::promises::new_resolving_promise(self, producer, mapper)
    }

    /// add reactions to an existing Promise object
    fn js_promise_add_reactions(
        &self,
        promise: &Self::JsValueAdapterType,
        then: Option<Self::JsValueAdapterType>,
        catch: Option<Self::JsValueAdapterType>,
        finally: Option<Self::JsValueAdapterType>,
    ) -> Result<(), JsError>;

    // cache
    /// cache a JsPromiseAdapter so it can be accessed later based on an id, while cached the JsPromiseAdapter object will not be garbage collected
    fn js_promise_cache_add(&self, promise_ref: Box<dyn JsPromiseAdapter<Self>>) -> usize;
    /// Consume a JsPromiseAdapter (remove from cache)
    fn js_promise_cache_consume(&self, id: usize) -> Box<dyn JsPromiseAdapter<Self>>;

    /// cache a Object so it can be accessed later based on an id, while cached the Object will not be garbage collected
    fn js_cache_add(&self, object: &Self::JsValueAdapterType) -> i32;
    /// remove an Object from the cache
    fn js_cache_dispose(&self, id: i32);
    /// use an Object in the cache
    fn js_cache_with<C, R>(&self, id: i32, consumer: C) -> R
    where
        C: FnOnce(&Self::JsValueAdapterType) -> R;
    /// get and remove an Object from the cache
    fn js_cache_consume(&self, id: i32) -> Self::JsValueAdapterType;

    // instanceof
    /// check if a JsValueAdapter is an instance of another JsValueAdapter
    fn js_instance_of(
        &self,
        object: &Self::JsValueAdapterType,
        constructor: &Self::JsValueAdapterType,
    ) -> bool;
    /// check if a JsValueAdapter is an instance of a constructor by name
    /// # Example
    /// ```dontrun
    /// if realm.js_instance_of_by_name(val, "Date")? {
    ///    // it's a date
    /// }
    /// ```
    fn js_instance_of_by_name(
        &self,
        object: &Self::JsValueAdapterType,
        constructor_name: &str,
    ) -> Result<bool, JsError> {
        let global = self.js_get_global()?;
        let constructor = self.js_object_get_property(&global, constructor_name)?;
        if constructor.js_is_null_or_undefined() {
            Err(JsError::new_string(format!(
                "no such constructor: {}",
                constructor_name
            )))
        } else {
            Ok(self.js_instance_of(object, &constructor))
        }
    }
    // json
    /// turn any JsValueAdapter into a JSON string
    fn js_json_stringify(
        &self,
        object: &Self::JsValueAdapterType,
        opt_space: Option<&str>,
    ) -> Result<String, JsError>;
    /// parse a JSON string into a JsValueAdapter
    fn js_json_parse(&self, json_string: &str) -> Result<Self::JsValueAdapterType, JsError>;

    ///create a new typed array
    fn js_typed_array_uint8_create(
        &self,
        buffer: Vec<u8>,
    ) -> Result<Self::JsValueAdapterType, JsError>;

    ///create a new typed array
    fn js_typed_array_uint8_create_copy(
        &self,
        buffer: &[u8],
    ) -> Result<Self::JsValueAdapterType, JsError>;

    fn js_typed_array_detach_buffer(
        &self,
        array: &Self::JsValueAdapterType,
    ) -> Result<Vec<u8>, JsError>;

    fn js_typed_array_copy_buffer(
        &self,
        array: &Self::JsValueAdapterType,
    ) -> Result<Vec<u8>, JsError>;

    fn js_proxy_instance_get_info(
        &self,
        obj: &Self::JsValueAdapterType,
    ) -> Result<(String, JsProxyInstanceId), JsError>
    where
        Self: Sized;
}

pub trait JsPromiseAdapter<R: JsRealmAdapter> {
    /// resolve this JsPromiseAdapter
    fn js_promise_resolve(
        &self,
        realm: &R,
        resolution: &R::JsValueAdapterType,
    ) -> Result<(), JsError>;
    /// reject this JsPromiseAdapter
    fn js_promise_reject(
        &self,
        realm: &R,
        rejection: &R::JsValueAdapterType,
    ) -> Result<(), JsError>;
    /// get the JsValueAdapter for this Promise
    fn js_promise_get_value(&self, realm: &R) -> R::JsValueAdapterType;
}

pub trait JsValueAdapter {
    type JsRuntimeAdapterType: JsRuntimeAdapter;

    /// js_get_type returns the rust type of the value (more extensive list than javascript typeof)
    fn js_get_type(&self) -> JsValueType;

    fn js_is_i32(&self) -> bool {
        self.js_get_type() == JsValueType::I32
    }
    fn js_is_f64(&self) -> bool {
        self.js_get_type() == JsValueType::F64
    }
    fn js_is_bool(&self) -> bool {
        self.js_get_type() == JsValueType::Boolean
    }
    fn js_is_string(&self) -> bool {
        self.js_get_type() == JsValueType::String
    }
    fn js_is_object(&self) -> bool {
        self.js_get_type() == JsValueType::Object
    }
    fn js_is_function(&self) -> bool {
        self.js_get_type() == JsValueType::Function
    }
    fn js_is_array(&self) -> bool {
        self.js_get_type() == JsValueType::Array
    }
    fn js_is_error(&self) -> bool {
        self.js_get_type() == JsValueType::Error
    }
    fn js_is_promise(&self) -> bool {
        self.js_get_type() == JsValueType::Promise
    }

    fn js_is_typed_array(&self) -> bool;
    fn js_is_proxy_instance(&self) -> bool;

    fn js_is_null_or_undefined(&self) -> bool {
        self.js_get_type() == JsValueType::Null || self.js_get_type() == JsValueType::Undefined
    }

    /// js_type_of returns the Javascript typeof string
    fn js_type_of(&self) -> &'static str;
    fn js_to_bool(&self) -> bool;
    fn js_to_i32(&self) -> i32;
    fn js_to_f64(&self) -> f64;
    fn js_to_string(&self) -> Result<String, JsError>;
    fn js_to_str(&self) -> Result<&str, JsError>;
}
