use crate::js_utils::adapters::proxies::JsProxyInstanceId;
use crate::js_utils::adapters::{JsRealmAdapter, JsRuntimeAdapter};
use crate::js_utils::facades::{JsRuntimeFacade, JsRuntimeFacadeInner, JsValueType};
use crate::js_utils::JsError;
use crate::resolvable_future::ResolvableFuture;
use futures::Future;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

pub struct CachedJsObjectRef {
    pub(crate) id: i32,
    realm_id: String,
    drop_action: Mutex<Option<Box<dyn FnOnce() + Send>>>,
}

pub struct CachedJsPromiseRef {
    pub cached_object: CachedJsObjectRef,
}

pub struct CachedJsArrayRef {
    pub cached_object: CachedJsObjectRef,
}

pub struct CachedJsFunctionRef {
    pub cached_object: CachedJsObjectRef,
}

impl CachedJsObjectRef {
    pub(crate) fn new<R: JsRealmAdapter + 'static>(realm: &R, obj: &R::JsValueAdapterType) -> Self {
        let id = realm.js_cache_add(obj);
        let rti_ref = realm.js_get_runtime_facade_inner();

        let drop_id = id;
        let drop_realm_name = realm.js_get_realm_id().to_string();
        Self::new2(id, realm.js_get_realm_id().to_string(), move || {
            if let Some(rti) = rti_ref.upgrade() {
                rti.js_add_rt_task_to_event_loop_void(move |rt| {
                    if let Some(realm) = rt.js_get_realm(drop_realm_name.as_str()) {
                        realm.js_cache_dispose(drop_id);
                    }
                })
            }
        })
    }
    fn new2<F: FnOnce() + Send + 'static>(id: i32, realm_name: String, drop_action: F) -> Self {
        Self {
            id,
            realm_id: realm_name,
            drop_action: Mutex::new(Some(Box::new(drop_action))),
        }
    }
    pub async fn js_get_object<R: JsRuntimeFacadeInner>(
        &self,
        rti: &R,
    ) -> Result<HashMap<String, JsValueFacade>, JsError> {
        let id = self.id;
        let realm_name = self.realm_id.clone();
        rti.js_add_rt_task_to_event_loop(move |rt| {
            if let Some(realm) = rt.js_get_realm(realm_name.as_str()) {
                //let realm: JsRealmAdapter = realm;
                let mut ret = HashMap::new();
                let results = realm.js_cache_with(id, |obj| {
                    realm.js_object_traverse(obj, |name, value| {
                        //
                        Ok((name.to_string(), realm.to_js_value_facade(value)))
                    })
                })?;
                for result in results {
                    ret.insert(result.0, result.1?);
                }
                Ok(ret)
            } else {
                Err(JsError::new_str("no such realm"))
            }
        })
        .await
    }
    pub fn with_obj_sync<
        S: Send + 'static,
        R: JsRuntimeFacadeInner,
        C: FnOnce(&<<<R as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType, &<<<<R as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType as JsRealmAdapter>::JsValueAdapterType) -> S + Send + 'static,
    >(
        &self,
        rti: &R,
        consumer: C,
    ) -> Result<S, JsError>{
        let id = self.id;
        let realm_id = self.realm_id.clone();
        rti.js_exe_rt_task_in_event_loop(move |rt| {
            if let Some(realm) = rt.js_get_realm(realm_id.as_str()) {
                Ok(realm.js_cache_with(id, |obj| consumer(realm, obj)))
            } else {
                Err(JsError::new_str("Realm was disposed"))
            }
        })
    }
    pub fn with_obj_void<
        S: Send + 'static,
        R: JsRuntimeFacadeInner,
        C: FnOnce(&<<<R as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType, &<<<<R as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType as JsRealmAdapter>::JsValueAdapterType) -> S + Send + 'static,
    >(
        &self,
        rti: &R,
        consumer: C,
    ){
        let id = self.id;
        let realm_id = self.realm_id.clone();
        rti.js_add_rt_task_to_event_loop_void(move |rt| {
            if let Some(realm) = rt.js_get_realm(realm_id.as_str()) {
                realm.js_cache_with(id, |obj| consumer(realm, obj));
            } else {
                log::error!("no such realm");
            }
        })
    }
    pub async fn with_obj<
        S: Send + 'static,
        R: JsRuntimeFacadeInner,
        C: FnOnce(&<<<R as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType, &<<<<R as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType as JsRealmAdapter>::JsValueAdapterType) -> S + Send + 'static,
    >(
        &self,
        rti: &R,
        consumer: C,
    ) -> Result<S, JsError>{
        let id = self.id;
        let realm_id = self.realm_id.clone();
        rti.js_add_rt_task_to_event_loop(move |rt| {
            if let Some(realm) = rt.js_get_realm(realm_id.as_str()) {
                Ok(realm.js_cache_with(id, |obj| consumer(realm, obj)))
            } else {
                Err(JsError::new_str("Realm was disposed"))
            }
        })
        .await
    }
}

impl Drop for CachedJsObjectRef {
    fn drop(&mut self) {
        let lck = &mut *self.drop_action.lock().unwrap();
        if let Some(da) = lck.take() {
            da();
        }
    }
}

impl CachedJsPromiseRef {
    pub async fn js_get_promise_result<R: JsRuntimeFacadeInner>(
        &self,
        rti: &R,
    ) -> Result<Result<JsValueFacade, JsValueFacade>, JsError> {
        let fut: ResolvableFuture<Result<Result<JsValueFacade, JsValueFacade>, JsError>> =
            ResolvableFuture::new();
        let resolver = fut.get_resolver();
        let resolver1 = resolver.clone();
        let resolver2 = resolver.clone();

        self.cached_object.with_obj_void(rti, move |realm, obj| {
            let res = || {
                let then_func = realm.js_function_create(
                    "then",
                    move |realm, _this, args| {
                        //
                        let resolution = &args[0];
                        let send_res = match realm.to_js_value_facade(resolution) {
                            Ok(vf) => resolver1.resolve(Ok(Ok(vf))),
                            Err(conv_err) => resolver1.resolve(Err(conv_err)),
                        };
                        send_res
                            .map_err(|e| JsError::new_string(format!("could not send: {}", e)))?;
                        realm.js_undefined_create()
                    },
                    1,
                )?;
                let catch_func = realm.js_function_create(
                    "catch",
                    move |realm, _this, args| {
                        //
                        let rejection = &args[0];
                        let send_res = match realm.to_js_value_facade(rejection) {
                            Ok(vf) => resolver2.resolve(Ok(Err(vf))),
                            Err(conv_err) => resolver2.resolve(Err(conv_err)),
                        };
                        send_res
                            .map_err(|e| JsError::new_string(format!("could not send: {}", e)))?;
                        realm.js_undefined_create()
                    },
                    1,
                )?;

                realm.js_promise_add_reactions(obj, Some(then_func), Some(catch_func), None)?;
                Ok(())
            };
            match res() {
                Ok(_) => {}
                Err(e) => match resolver.resolve(Err(e)) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("failed to resolve 47643: {}", e);
                    }
                },
            }
        });

        fut.await
    }
}

impl CachedJsArrayRef {
    pub async fn js_get_array<R: JsRuntimeFacadeInner>(
        &self,
        _rti: &R,
    ) -> Result<Vec<JsValueFacade>, JsError> {
        todo!()
    }
}

impl CachedJsFunctionRef {
    pub async fn js_invoke_function<R: JsRuntimeFacadeInner>(
        &self,
        rti: &R,
        args: Vec<JsValueFacade>,
    ) -> Result<JsValueFacade, JsError> {
        let cached_obj_id = self.cached_object.id;
        let realm_id = self.cached_object.realm_id.clone();

        rti.js_add_rt_task_to_event_loop(move |rt| {
            //
            if let Some(realm) = rt.js_get_realm(realm_id.as_str()) {
                realm.js_cache_with(cached_obj_id, |func_adapter| {
                    let mut adapter_args = vec![];
                    for arg in args {
                        adapter_args.push(realm.from_js_value_facade(arg)?);
                    }

                    let adapter_refs: Vec<&<<<<R as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType as JsRealmAdapter>::JsValueAdapterType> = adapter_args.iter().collect();

                    let val_adapter = realm.js_function_invoke(
                        None,
                        func_adapter,
                        &adapter_refs,
                    )?;

                    realm.to_js_value_facade(&val_adapter)
                })
            } else {
                Ok(JsValueFacade::Null)
            }
        })
        .await
    }
    pub fn js_invoke_function_sync<R: JsRuntimeFacadeInner>(
        &self,
        _rti: &R,
        _args: Vec<JsValueFacade>,
    ) -> Result<JsValueFacade, JsError> {
        todo!()
    }
}

/// The JsValueFacade is a Send-able representation of a value in the Script engine
#[allow(clippy::type_complexity)]
pub enum JsValueFacade {
    // todo new proxy instance
    I32 {
        val: i32,
    },
    F64 {
        val: f64,
    },
    String {
        val: String,
    },
    Boolean {
        val: bool,
    },
    JsObject {
        // obj which is a ref to obj in Js
        cached_object: CachedJsObjectRef,
    },
    JsPromise {
        cached_promise: CachedJsPromiseRef,
    },
    JsArray {
        cached_array: CachedJsArrayRef,
    },
    JsFunction {
        cached_function: CachedJsFunctionRef,
    },
    // obj created from rust
    Object {
        val: HashMap<String, JsValueFacade>,
    },
    // array created from rust
    Array {
        val: Vec<JsValueFacade>,
    },
    // promise created from rust which will run an async producer
    Promise {
        producer: Mutex<
            Option<Pin<Box<dyn Future<Output = Result<JsValueFacade, JsError>> + Send + 'static>>>,
        >,
    },
    // Function created from rust
    Function {
        name: String,
        arg_count: u32,
        func: Arc<Box<dyn Fn(&[JsValueFacade]) -> Result<JsValueFacade, JsError> + Send + Sync>>,
    },
    JsError {
        val: JsError,
    },
    ProxyInstance {
        namespace: &'static [&'static str],
        class_name: &'static str,
        instance_id: JsProxyInstanceId,
    },
    Null,
    Undefined,
}

impl JsValueFacade {
    pub fn new_i32(val: i32) -> Self {
        Self::I32 { val }
    }
    pub fn new_f64(val: f64) -> Self {
        Self::F64 { val }
    }
    pub fn new_bool(val: bool) -> Self {
        Self::Boolean { val }
    }
    pub fn new_str(val: &str) -> Self {
        Self::String {
            val: val.to_string(),
        }
    }
    pub fn new_string(val: String) -> Self {
        Self::String { val }
    }
    pub fn new_callback<
        F: Fn(&[JsValueFacade]) -> Result<JsValueFacade, JsError> + Send + Sync + 'static,
    >(
        callback: F,
    ) -> Self {
        Self::Function {
            name: "".to_string(),
            arg_count: 0,
            func: Arc::new(Box::new(callback)),
        }
    }
    pub fn new_function<
        F: Fn(&[JsValueFacade]) -> Result<JsValueFacade, JsError> + Send + Sync + 'static,
    >(
        name: &str,
        function: F,
        arg_count: u32,
    ) -> Self {
        Self::Function {
            name: name.to_string(),
            arg_count,
            func: Arc::new(Box::new(function)),
        }
    }
    /// create a new promise with a producer which will run async in a threadpool
    pub fn new_promise<T, R, P, M>(producer: P) -> Self
    where
        T: JsRealmAdapter,
        P: Future<Output = Result<JsValueFacade, JsError>> + Send + 'static,
    {
        JsValueFacade::Promise {
            producer: Mutex::new(Some(Box::pin(producer))),
        }
    }
    pub fn is_i32(&self) -> bool {
        matches!(self, JsValueFacade::I32 { .. })
    }
    pub fn is_f64(&self) -> bool {
        matches!(self, JsValueFacade::F64 { .. })
    }
    pub fn is_bool(&self) -> bool {
        matches!(self, JsValueFacade::Boolean { .. })
    }
    pub fn is_string(&self) -> bool {
        matches!(self, JsValueFacade::String { .. })
    }
    pub fn get_i32(&self) -> i32 {
        match self {
            JsValueFacade::I32 { val } => *val,
            _ => {
                panic!("Not an i32");
            }
        }
    }
    pub fn get_f64(&self) -> f64 {
        match self {
            JsValueFacade::F64 { val } => *val,
            _ => {
                panic!("Not an f64");
            }
        }
    }
    pub fn get_bool(&self) -> bool {
        match self {
            JsValueFacade::Boolean { val } => *val,
            _ => {
                panic!("Not a boolean");
            }
        }
    }
    pub fn get_str(&self) -> &str {
        match self {
            JsValueFacade::String { val } => val.as_str(),
            _ => {
                panic!("Not a string");
            }
        }
    }
    pub fn is_null_or_undefined(&self) -> bool {
        matches!(self, JsValueFacade::Null | JsValueFacade::Undefined)
    }
    pub fn get_value_type(&self) -> JsValueType {
        match self {
            JsValueFacade::I32 { .. } => JsValueType::I32,
            JsValueFacade::F64 { .. } => JsValueType::F64,
            JsValueFacade::String { .. } => JsValueType::String,
            JsValueFacade::Boolean { .. } => JsValueType::Boolean,
            JsValueFacade::JsObject { .. } => JsValueType::Object,
            JsValueFacade::Null => JsValueType::Null,
            JsValueFacade::Undefined => JsValueType::Undefined,
            JsValueFacade::Object { .. } => JsValueType::Object,
            JsValueFacade::Array { .. } => JsValueType::Array,
            JsValueFacade::Promise { .. } => JsValueType::Promise,
            JsValueFacade::Function { .. } => JsValueType::Function,
            JsValueFacade::JsPromise { .. } => JsValueType::Promise,
            JsValueFacade::JsArray { .. } => JsValueType::Array,
            JsValueFacade::JsFunction { .. } => JsValueType::Function,
            JsValueFacade::JsError { .. } => JsValueType::Error,
            JsValueFacade::ProxyInstance { .. } => JsValueType::Object,
        }
    }
    pub fn stringify(&self) -> String {
        match self {
            JsValueFacade::I32 { val } => {
                format!("I32: {}", val)
            }
            JsValueFacade::F64 { val } => {
                format!("F64: {}", val)
            }
            JsValueFacade::String { val } => {
                format!("String: {}", val)
            }
            JsValueFacade::Boolean { val } => {
                format!("Boolean: {}", val)
            }
            JsValueFacade::JsObject { cached_object } => {
                format!(
                    "JsObject: [{}.{}]",
                    cached_object.realm_id, cached_object.id
                )
            }
            JsValueFacade::JsPromise { cached_promise } => {
                format!(
                    "JsPromise: [{}.{}]",
                    cached_promise.cached_object.realm_id, cached_promise.cached_object.id
                )
            }
            JsValueFacade::JsArray { cached_array } => {
                format!(
                    "JsArray: [{}.{}]",
                    cached_array.cached_object.realm_id, cached_array.cached_object.id
                )
            }
            JsValueFacade::JsFunction { cached_function } => {
                format!(
                    "JsFunction: [{}.{}]",
                    cached_function.cached_object.realm_id, cached_function.cached_object.id
                )
            }
            JsValueFacade::Object { val } => {
                format!("Object: [len={}]", val.keys().len())
            }
            JsValueFacade::Array { val } => {
                format!("Array: [len={}]", val.len())
            }
            JsValueFacade::Promise { .. } => "Promise".to_string(),
            JsValueFacade::Function { .. } => "Function".to_string(),
            JsValueFacade::Null => "Null".to_string(),
            JsValueFacade::Undefined => "Undefined".to_string(),
            JsValueFacade::JsError { val } => format!("{}", val),
            JsValueFacade::ProxyInstance { .. } => "ProxyInstance".to_string(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::js_utils::facades::values::JsValueFacade;

    #[test]
    fn test_jsvf() {
        fn test<A: Send + Sync>(_a: A) {}

        test(JsValueFacade::Null);
    }
}
