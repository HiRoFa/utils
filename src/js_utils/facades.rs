use crate::js_utils::adapters::{JsRealmAdapter, JsRuntimeAdapter};
use crate::js_utils::{JsError, Script, ScriptPreProcessor};
use crate::resolvable_future::ResolvableFuture;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc::channel;
use std::sync::{Arc, Weak};

pub mod values;

pub struct JsProxy {}

pub trait JsRuntimeFacadeInner {
    type JsRuntimeFacadeType: JsRuntimeFacade;
    fn js_exe_rt_task_in_event_loop<
        R: Send + 'static,
        J: FnOnce(&<<Self as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType) -> R + Send + 'static,
    >(
        &self,
        task: J,
    ) -> R;
    fn js_add_rt_task_to_event_loop<
        R: Send + 'static,
        J: FnOnce(&<<Self as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType) -> R + Send + 'static,
    >(
        &self,
        task: J,
    ) -> Pin<Box<dyn Future<Output = R>>>;
    fn js_add_rt_task_to_event_loop_void<J: FnOnce(&<<Self as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType) + Send + 'static>(
        &self,
        task: J,
    );
}

pub trait JsRuntimeBuilder {
    type JsRuntimeFacadeType: JsRuntimeFacade;
    fn js_build(self) -> Self::JsRuntimeFacadeType;
    fn js_runtime_init_hook<
        H: FnOnce(&Self::JsRuntimeFacadeType) -> Result<(), JsError> + Send + 'static,
    >(
        &mut self,
        hook: H,
    ) -> &mut Self;
    fn js_script_pre_processor<S: ScriptPreProcessor + Send + 'static>(
        &mut self,
        preprocessor: S,
    ) -> &mut Self;
}
/// The JsRuntime facade is the main entry point to the JavaScript engine, it is thread safe and
/// handles the logic for transferring data from and to the JsRuntimeAdapter
pub trait JsRuntimeFacade {
    type JsRuntimeAdapterType: JsRuntimeAdapter;
    type JsRuntimeFacadeInnerType: JsRuntimeFacadeInner + Send + Sync;

    fn js_get_runtime_facade_inner(&self) -> Weak<Self::JsRuntimeFacadeInnerType>;

    fn js_realm_create(&mut self, name: &str) -> Result<(), JsError>;
    fn js_realm_destroy(&mut self, name: &str) -> Result<(), JsError>;
    fn js_realm_has(&mut self, name: &str) -> Result<bool, JsError>;

    fn js_loop_sync<
        R: Send + 'static,
        C: FnOnce(&Self::JsRuntimeAdapterType) -> R + Send + 'static,
    >(
        &self,
        consumer: C,
    ) -> R;
    fn js_loop<R: Send + 'static, C: FnOnce(&Self::JsRuntimeAdapterType) -> R + Send + 'static>(
        &self,
        consumer: C,
    ) -> Pin<Box<dyn Future<Output = R> + Send>>;
    fn js_loop_void<C: FnOnce(&Self::JsRuntimeAdapterType) + Send + 'static>(&self, consumer: C);

    // realm jobs

    fn js_loop_realm_sync<
        R: Send + 'static,
        C: FnOnce(
                &Self::JsRuntimeAdapterType,
                &<<Self as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType,
            ) -> R + Send
            + 'static ,
    >(
        &self,
        realm_name: Option<&str>,
        consumer: C,
    ) -> R;

    fn js_loop_realm<
        R: Send + 'static,
        C: FnOnce(
                &Self::JsRuntimeAdapterType,
                &<<Self as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType,
            ) -> R + Send + 'static,
    >(
        &self,
        realm_name: Option<&str>,
        consumer: C,
    ) -> Pin<Box<dyn Future<Output = R>>>;

    fn js_loop_realm_void<
        C: FnOnce(
                &Self::JsRuntimeAdapterType,
                &<<Self as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType,
            ) + Send
            + 'static,
    >(
        &self,
        realm_name: Option<&str>,
        consumer: C,
    );

    /// eval a script, please note that eval should not be used for production code, you should always
    /// use modules or functions and invoke them
    /// eval will always need to parse script and some engines like StarLight even require a different syntax (return(1); vs (1);)
    /// If None is passed as realm_name the default Realm wil be used
    #[allow(clippy::type_complexity)]
    fn js_eval(
        &self,
        realm_name: Option<&str>,
        script: Script,
    ) -> Pin<Box<dyn Future<Output = Result<Box<dyn JsValueFacade>, JsError>>>>;

    // function methods
    /// Invoke a function and block until the function is done
    /// If None is passed as realm_name the default Realm wil be used
    fn js_function_invoke_sync(
        &self,
        realm_name: Option<&str>,
        namespace: &[&str],
        method_name: &str,
        args: Vec<Box<dyn JsValueFacade>>,
    ) -> Result<Box<dyn JsValueFacade>, JsError>;

    /// Invoke a function
    /// this returns a Future which will fulfill when the function is done
    /// If None is passed as realm_name the default Realm wil be used
    #[allow(clippy::type_complexity)]
    fn js_function_invoke(
        &self,
        realm_name: Option<&str>,
        namespace: &[&str],
        method_name: &str,
        args: Vec<Box<dyn JsValueFacade>>,
    ) -> Pin<Box<dyn Future<Output = Result<Box<dyn JsValueFacade>, JsError>>>>;

    /// Invoke a function without waiting for a result
    /// If None is passed as realm_name the default Realm wil be used
    fn js_function_invoke_void(
        &self,
        realm_name: Option<&str>,
        namespace: &[&str],
        method_name: &str,
        args: Vec<Box<dyn JsValueFacade>>,
    );
}

/// The JsValueFacade is a Send-able representation of a variable in the Script engine

#[derive(PartialEq, Copy, Clone)]
pub enum JsValueType {
    I32,
    F64,
    String,
    Boolean,
    Object,
    Function,
    BigInt,
    Promise,
    Date,
    Null,
    Undefined,
    Array,
}

pub struct CachedJsObjectRef<R: JsRealmAdapter> {
    id: i32,
    rti: Weak<<<<R as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType>,
    realm: String,
}

impl<R: JsRealmAdapter> CachedJsObjectRef<R> {
    pub(crate) fn new(realm: &R, obj: &R::JsValueAdapterType) -> Self {
        let id = realm.js_cache_add(obj);
        let rti = realm.js_get_runtime_facade_inner();
        Self {
            id,
            rti,
            realm: realm.js_get_realm_id().to_string(),
        }
    }
    pub fn with_obj_sync<S: Send + 'static, C: FnOnce(&<<<<<<R as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType, &<<<<<<<R as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType as JsRealmAdapter>::JsValueAdapterType) -> S + Send + 'static>(
        &self,
        consumer: C,
    ) -> Result<S, JsError>{
        if let Some(rti) = self.rti.upgrade() {
            let id = self.id;
            let realm = self.realm.clone();
            rti.js_exe_rt_task_in_event_loop(move |rt| {
                if let Some(realm) = rt.js_get_realm(realm.as_str()) {
                    Ok(realm.js_cache_with(id, |obj| consumer(realm, obj)))
                } else {
                    Err(JsError::new_str("Realm was disposed"))
                }
            })
        } else {
            Err(JsError::new_str("Runtime was disposed"))
        }
    }
    pub fn with_obj_void<C: FnOnce(&<<<<<<R as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType, &<<<<<<<R as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType as JsRealmAdapter>::JsValueAdapterType) + Send + 'static>(&self, consumer: C){
        if let Some(rti) = self.rti.upgrade() {
            let id = self.id;
            let realm = self.realm.clone();
            rti.js_add_rt_task_to_event_loop_void(move |rt| {
                if let Some(realm) = rt.js_get_realm(realm.as_str()) {
                    realm.js_cache_with(id, |obj| consumer(realm, obj));
                }
            });
        }
    }
    pub fn with_obj<S: Send + 'static, C: FnOnce(&<<<<<<R as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType, &<<<<<<<R as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType as JsRealmAdapter>::JsValueAdapterType) -> S + Send + 'static>(
        &self,
        consumer: C,
    ) -> Pin<Box<dyn Future<Output = Result<S, JsError>>>>{
        if let Some(rti) = self.rti.upgrade() {
            let id = self.id;
            let realm = self.realm.clone();
            rti.js_add_rt_task_to_event_loop(move |rt| {
                if let Some(realm) = rt.js_get_realm(realm.as_str()) {
                    Ok(realm.js_cache_with(id, |obj| consumer(realm, obj)))
                } else {
                    Err(JsError::new_str("Realm was disposed"))
                }
            })
        } else {
            panic!("Runtime was disposed");
        }
    }
}

pub(crate) struct FromJsPromise<R: JsRealmAdapter> {
    pub(crate) obj: Arc<CachedJsObjectRef<R>>,
}

impl<R: JsRealmAdapter> JsValueFacade for FromJsPromise<R> {
    fn js_get_type(&self) -> JsValueType {
        JsValueType::Promise
    }

    fn js_stringify(&self) -> Result<String, JsError> {
        Ok("Promise<FromJsPromise>".to_string())
    }

    #[allow(clippy::type_complexity)]
    fn js_get_promise_result_sync(
        &self,
    ) -> Result<Result<Box<dyn JsValueFacade + 'static>, Box<dyn JsValueFacade + 'static>>, JsError>
    {
        let (tx, rx) = channel();
        let _ = self.obj.with_obj_sync(move |realm, obj| {
            // send results to tx
            let tx1 = tx.clone();
            let tx2 = tx.clone();

            let then_func = realm.js_function_create(
                "then",
                move |realm, _this, args| {
                    //
                    let resolution = &args[0];
                    let send_res = match realm.to_js_value_facade(resolution) {
                        Ok(vf) => tx1.send(Ok(Ok(vf))),
                        Err(conv_err) => tx1.send(Err(conv_err)),
                    };
                    send_res.map_err(|e| JsError::new_string(format!("could not send: {}", e)))?;
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
                        Ok(vf) => tx2.send(Ok(Err(vf))),
                        Err(conv_err) => tx2.send(Err(conv_err)),
                    };
                    send_res.map_err(|e| JsError::new_string(format!("could not send: {}", e)))?;
                    realm.js_undefined_create()
                },
                1,
            )?;

            let res = realm.js_promise_add_reactions(obj, Some(then_func), Some(catch_func), None);
            if let Some(add_reactions_err) = res.err() {
                tx.send(Err(add_reactions_err))
                    .map_err(|e| JsError::new_string(format!("could not send: {}", e)))
            } else {
                Ok(())
            }
        })?;

        // get result from rx
        rx.recv()
            .map_err(|e| JsError::new_string(format!("receive failed: {}", e)))?
    }

    #[allow(clippy::type_complexity)]
    fn js_get_promise_result(
        &self,
    ) -> Pin<
        Box<
            dyn Future<
                Output = Result<Result<Box<dyn JsValueFacade>, Box<dyn JsValueFacade>>, JsError>,
            >,
        >,
    > {
        let fut: ResolvableFuture<
            Result<Result<Box<dyn JsValueFacade>, Box<dyn JsValueFacade>>, JsError>,
        > = ResolvableFuture::new();
        let resolver = fut.get_resolver();
        let resolver1 = resolver.clone();
        let resolver2 = resolver.clone();

        self.obj.with_obj_void(move |realm, obj| {
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

        Box::pin(fut)
    }
}

impl<R: JsRealmAdapter> Drop for CachedJsObjectRef<R> {
    fn drop(&mut self) {
        if let Some(rti) = self.rti.upgrade() {
            let id = self.id;
            let realm = self.realm.clone();
            rti.js_exe_rt_task_in_event_loop(move |rt| {
                if let Some(realm) = rt.js_get_realm(realm.as_str()) {
                    realm.js_cache_dispose(id);
                }
            })
        }
    }
}

pub trait JsValueFacade: Send + Sync {
    fn js_is_null_or_undefined(&self) -> bool {
        false
    }
    fn js_as_i32(&self) -> i32 {
        panic!("not an i32");
    }
    fn js_as_f64(&self) -> f64 {
        panic!("not an f64");
    }
    fn js_as_str(&self) -> &str {
        panic!("not a String");
    }
    fn js_as_bool(&self) -> bool {
        panic!("not a bool");
    }
    fn js_get_type(&self) -> JsValueType;

    fn js_stringify(&self) -> Result<String, JsError>;
    fn js_get_array(&self) -> Result<&Vec<Box<dyn JsValueFacade>>, JsError> {
        panic!("not an Array")
    }
    fn js_get_object(&self) -> Result<&HashMap<String, Box<dyn JsValueFacade>>, JsError> {
        panic!("not an Object")
    }
    #[allow(clippy::type_complexity)]
    fn js_invoke_function(
        &self,
        _args: &[Box<dyn JsValueFacade>],
    ) -> Pin<Box<dyn Future<Output = Result<Box<dyn JsValueFacade>, JsError>>>> {
        panic!("not a Function")
    }
    fn js_invoke_function_sync(&self) -> Result<Box<dyn JsValueFacade>, JsError> {
        panic!("not a Function")
    }
    fn js_invoke_function_void(&self) {
        panic!("not a Function")
    }
    #[allow(clippy::type_complexity)]
    fn js_get_promise_result_sync(
        &self,
    ) -> Result<Result<Box<dyn JsValueFacade + 'static>, Box<dyn JsValueFacade + 'static>>, JsError>
    {
        panic!("not a Promise")
    }
    #[allow(clippy::type_complexity)]
    fn js_get_promise_result(
        &self,
    ) -> Pin<
        Box<
            dyn Future<
                Output = Result<Result<Box<dyn JsValueFacade>, Box<dyn JsValueFacade>>, JsError>,
            >,
        >,
    > {
        panic!("not a Promise")
    }
    fn js_resolve_promise(
        &self,
        _resolution: Result<Box<dyn JsValueFacade>, Box<dyn JsValueFacade>>,
    ) {
        panic!("not a resolvable Promise")
    }
}

pub struct JsNull {}
pub struct JsUndefined {}
pub struct JsPromise {}
pub struct JsFunction {}
pub type JsObject = HashMap<String, Box<dyn JsValueFacade>>;
pub type JsArray = Vec<Box<dyn JsValueFacade>>;

impl JsPromise {
    pub fn new() -> Self {
        Self {}
    }
    pub fn new_resolving<F: FnOnce() -> Result<Box<dyn JsValueFacade>, Box<dyn JsValueFacade>>>(
        _resolver: F,
    ) -> Self {
        todo!();
    }
    pub fn new_async<R>(_resolver: R) -> Self
    where
        R: Future<Output = Result<Box<dyn JsValueFacade>, Box<dyn JsValueFacade>>> + Send + 'static,
    {
        todo!();
    }
}

impl Default for JsPromise {
    fn default() -> Self {
        Self::new()
    }
}

impl JsFunction {
    pub fn new<F: Fn() -> Result<Box<dyn JsValueFacade>, Box<dyn JsValueFacade>>>(
        _callback: F,
    ) -> Self {
        todo!();
    }
}

impl JsValueFacade for JsNull {
    fn js_is_null_or_undefined(&self) -> bool {
        true
    }

    fn js_get_type(&self) -> JsValueType {
        JsValueType::Null
    }

    fn js_stringify(&self) -> Result<String, JsError> {
        Ok("null".to_string())
    }
}

impl JsValueFacade for JsUndefined {
    fn js_is_null_or_undefined(&self) -> bool {
        true
    }

    fn js_get_type(&self) -> JsValueType {
        JsValueType::Undefined
    }

    fn js_stringify(&self) -> Result<String, JsError> {
        Ok("undefined".to_string())
    }
}

impl JsValueFacade for JsPromise {
    fn js_get_type(&self) -> JsValueType {
        todo!()
    }

    fn js_stringify(&self) -> Result<String, JsError> {
        todo!()
    }

    fn js_resolve_promise(
        &self,
        _resolution: Result<Box<dyn JsValueFacade>, Box<dyn JsValueFacade>>,
    ) {
        todo!()
    }
}

impl JsValueFacade for JsFunction {
    fn js_get_type(&self) -> JsValueType {
        todo!()
    }

    fn js_stringify(&self) -> Result<String, JsError> {
        todo!()
    }
}

impl JsValueFacade for Vec<Box<dyn JsValueFacade>> {
    fn js_get_type(&self) -> JsValueType {
        JsValueType::Array
    }

    fn js_stringify(&self) -> Result<String, JsError> {
        todo!()
    }

    fn js_get_array(&self) -> Result<&Vec<Box<dyn JsValueFacade>>, JsError> {
        Ok(self)
    }
}

impl JsValueFacade for HashMap<String, Box<dyn JsValueFacade>> {
    fn js_get_type(&self) -> JsValueType {
        JsValueType::Object
    }

    fn js_stringify(&self) -> Result<String, JsError> {
        todo!()
    }

    fn js_get_object(&self) -> Result<&HashMap<String, Box<dyn JsValueFacade>>, JsError> {
        Ok(self)
    }
}

impl JsValueFacade for i32 {
    fn js_as_i32(&self) -> i32 {
        *self
    }

    fn js_get_type(&self) -> JsValueType {
        JsValueType::I32
    }

    fn js_stringify(&self) -> Result<String, JsError> {
        Ok(format!("{}", self.js_as_i32()))
    }
}

impl JsValueFacade for f64 {
    fn js_as_f64(&self) -> f64 {
        *self
    }

    fn js_get_type(&self) -> JsValueType {
        JsValueType::F64
    }
    fn js_stringify(&self) -> Result<String, JsError> {
        Ok(format!("{}", self.js_as_f64()))
    }
}

impl JsValueFacade for bool {
    fn js_as_bool(&self) -> bool {
        *self
    }

    fn js_get_type(&self) -> JsValueType {
        JsValueType::Boolean
    }
    fn js_stringify(&self) -> Result<String, JsError> {
        Ok(format!("{}", self.js_as_bool()))
    }
}

impl JsValueFacade for String {
    fn js_as_str(&self) -> &str {
        self.as_str()
    }

    fn js_get_type(&self) -> JsValueType {
        JsValueType::String
    }
    fn js_stringify(&self) -> Result<String, JsError> {
        Ok(format!("\"{}\"", self.js_as_str().replace("\"", "\\\"")))
    }
}
