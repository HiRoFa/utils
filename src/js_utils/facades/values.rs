use crate::js_utils::adapters::{JsRealmAdapter, JsRuntimeAdapter};
use crate::js_utils::facades::{JsRuntimeFacade, JsRuntimeFacadeInner, JsValueType};
use crate::js_utils::JsError;
use crate::resolvable_future::ResolvableFuture;
use std::collections::HashMap;
use std::sync::Arc;

pub struct CachedJsObjectRef {
    pub(crate) id: i32,
    realm_id: String,
    drop_action: Option<Box<dyn FnOnce() + Send>>,
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
            drop_action: Some(Box::new(drop_action)),
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
        if let Some(da) = self.drop_action.take() {
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
        _rti: &R,
        _args: Vec<JsValueFacade>,
    ) -> Result<JsValueFacade, JsError> {
        todo!()
    }
}

#[allow(clippy::type_complexity)]
pub enum JsValueFacade {
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
    // Promise created from rust
    Promise {
        resolve_handle: Arc<()>, //todo put JsPromiseAdapter in thread_local_map?
    },
    // Function created from rust
    Function {
        name: String,
        arg_count: u32,
        func: Arc<Box<dyn Fn(&[JsValueFacade]) -> Result<JsValueFacade, JsError> + Send + Sync>>,
    },
    Null,
    Undefined,
}

impl JsValueFacade {
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
    pub fn new_promise<F: FnOnce() -> Result<JsValueFacade, JsValueFacade>>(_resolver: F) -> Self {
        JsValueFacade::Promise {
            resolve_handle: Arc::new(()),
        }
    }
    pub fn new_resolvable_promise() -> Self {
        JsValueFacade::Promise {
            resolve_handle: Arc::new(()),
        }
    }
    pub fn js_is_null_or_undefined(&self) -> bool {
        matches!(self, JsValueFacade::Null | JsValueFacade::Undefined)
    }
    pub fn js_get_value_type(&self) -> JsValueType {
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
        }
    }
}
