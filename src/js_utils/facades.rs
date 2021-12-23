use crate::js_utils::adapters::JsRuntimeAdapter;
use crate::js_utils::facades::values::JsValueFacade;
use crate::js_utils::modules::{CompiledModuleLoader, NativeModuleLoader, ScriptModuleLoader};
use crate::js_utils::{JsError, Script, ScriptPreProcessor};
use std::future::Future;
use std::pin::Pin;
use std::sync::Weak;

pub mod async_utils;
pub mod values;

pub struct JsProxy {}

pub trait JsRuntimeFacadeInner {
    type JsRuntimeFacadeType: JsRuntimeFacade;
    /// run a closure in the EventLoop with a &JsRuntimeAdapter as only param and await the result blocking
    fn js_exe_rt_task_in_event_loop<
        R: Send + 'static,
        J: FnOnce(&<<Self as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType) -> R + Send + 'static,
    >(
        &self,
        task: J,
    ) -> R;
    /// run a closure in the EventLoop with a &JsRuntimeAdapter as only param and return a future which will fulfill after the closure has been called
    fn js_add_rt_task_to_event_loop<
        R: Send + 'static,
        J: FnOnce(&<<Self as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType) -> R + Send + 'static,
    >(
        &self,
        task: J,
    ) -> Pin<Box<dyn Future<Output = R>>>;
    /// run a closure in the EventLoop with a &JsRuntimeAdapter as only param
    fn js_add_rt_task_to_event_loop_void<J: FnOnce(&<<Self as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType) + Send + 'static>(
        &self,
        task: J,
    );
}

pub trait JsRuntimeBuilder {
    type JsRuntimeFacadeType: JsRuntimeFacade;
    /// construct a JsRuntimeFacade based on this builders options
    fn js_build(self) -> Self::JsRuntimeFacadeType;
    /// add a runtime init hook, this closure will be invoked when a JsRuntimeFacade is constructed (e.g. builder.build is called)
    fn js_runtime_init_hook<
        H: FnOnce(&Self::JsRuntimeFacadeType) -> Result<(), JsError> + Send + 'static,
    >(
        self,
        hook: H,
    ) -> Self;
    /// add a realm adapter init hook, this will be called every time a realm is initialized
    fn js_realm_adapter_init_hook<
        H: FnOnce(&<<Self as JsRuntimeBuilder>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType, &<<<Self as JsRuntimeBuilder>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType) -> Result<(), JsError> + Send + 'static,
    >(
        self,
        hook: H,
    ) -> Self;
    fn js_runtime_adapter_init_hook<
        H: FnOnce(&<<Self as JsRuntimeBuilder>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType) -> Result<(), JsError> + Send + 'static,
    >(
        self,
        hook: H,
    ) -> Self;
    /// add a script preprocessor
    fn js_script_pre_processor<S: ScriptPreProcessor + Send + 'static>(
        self,
        preprocessor: S,
    ) -> Self;
    /// add a module loader
    fn js_script_module_loader<S: ScriptModuleLoader<<<<Self as JsRuntimeBuilder>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType> + Send + 'static>(
        self,
        module_loader: S,
    ) -> Self;
    /// add a compiled_module loader
    fn js_compiled_module_loader<
        S: CompiledModuleLoader<<<<Self as JsRuntimeBuilder>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType>
            + Send
            + 'static
    >(
        self,
        module_loader: S,
    ) -> Self;
    /// add a native module loader (e.g. with proxy classes instead of a script)
    fn js_native_module_loader<
        S: NativeModuleLoader<<<<Self as JsRuntimeBuilder>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType>
            + Send
            + 'static,
    >(
        self,
        module_loader: S,
    ) -> Self
    where
        Self: Sized;
}
/// The JsRuntime facade is the main entry point to the JavaScript engine, it is thread safe and
/// handles the logic for transferring data from and to the JsRuntimeAdapter
pub trait JsRuntimeFacade {
    type JsRuntimeAdapterType: JsRuntimeAdapter;
    type JsRuntimeFacadeInnerType: JsRuntimeFacadeInner + Send + Sync;

    /// obtain a Weak reference to the JsRuntimeFacadeInner, this is often used to add jobs to the EventLoop from async tasks
    fn js_get_runtime_facade_inner(&self) -> Weak<Self::JsRuntimeFacadeInnerType>;

    /// create a new JavaScript realm or context
    fn js_realm_create(&mut self, name: &str) -> Result<(), JsError>;

    /// remove a JavaScript realm or context
    fn js_realm_destroy(&mut self, name: &str) -> Result<(), JsError>;

    /// check if a realm is present
    fn js_realm_has(&self, name: &str) -> Result<bool, JsError>;

    /// util method to add a job to the EventLoop, usually this is passed to the JsRuntimeFacadeInner.js_loop_sync
    fn js_loop_sync<
        R: Send + 'static,
        C: FnOnce(&Self::JsRuntimeAdapterType) -> R + Send + 'static,
    >(
        &self,
        consumer: C,
    ) -> R;

    /// util method to add a job to the EventLoop, usually this is passed to the JsRuntimeFacadeInner.js_loop_sync
    fn js_loop_sync_mut<
        R: Send + 'static,
        C: FnOnce(&mut Self::JsRuntimeAdapterType) -> R + Send + 'static,
    >(
        &self,
        consumer: C,
    ) -> R;
    /// util method to add a job to the EventLoop, usually this is passed to the JsRuntimeFacadeInner.js_loop_sync
    fn js_loop<R: Send + 'static, C: FnOnce(&Self::JsRuntimeAdapterType) -> R + Send + 'static>(
        &self,
        consumer: C,
    ) -> Pin<Box<dyn Future<Output = R> + Send>>;
    /// util method to add a job to the EventLoop, usually this is passed to the JsRuntimeFacadeInner.js_loop_void
    fn js_loop_void<C: FnOnce(&Self::JsRuntimeAdapterType) + Send + 'static>(&self, consumer: C);

    // realm jobs
    /// util method to add a job to the EventLoop, usually this is passed to the JsRuntimeFacadeInner.js_loop_realm_sync
    /// if the realm does not exist it should be initialized, in order to customize its initialization you could add a realm_init_hook to the Builder
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
    /// util method to add a job to the EventLoop, usually this is passed to the JsRuntimeFacadeInner.js_loop_realm
    /// if the realm does not exist it should be initialized, in order to customize its initialization you could add a realm_init_hook to the Builder
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
    /// util method to add a job to the EventLoop, usually this is passed to the JsRuntimeFacadeInner.js_loop_realm_void
    /// if the realm does not exist it should be initialized, in order to customize its initialization you could add a realm_init_hook to the Builder
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

    /// evaluate a script, please note that eval should not be used for production code, you should always
    /// use modules or functions and invoke them
    /// eval will always need to parse script and some engines like StarLight even require a different syntax (return(1); vs (1);)
    /// If None is passed as realm_name the main Realm wil be used
    #[allow(clippy::type_complexity)]
    fn js_eval(
        &self,
        realm_name: Option<&str>,
        script: Script,
    ) -> Pin<Box<dyn Future<Output = Result<JsValueFacade, JsError>>>>;

    /// eval a script, please note that eval should not be used for production code, you should always
    /// use modules or functions and invoke them
    /// eval will always need to parse script and some engines like StarLight even require a different syntax (return(1); vs (1);)
    /// If None is passed as realm_name the main Realm wil be used
    #[allow(clippy::type_complexity)]
    fn js_eval_module(
        &self,
        realm_name: Option<&str>,
        script: Script,
    ) -> Pin<Box<dyn Future<Output = Result<(), JsError>>>>;

    // function methods
    /// Invoke a function and block until the function is done
    /// If None is passed as realm_name the main Realm wil be used
    fn js_function_invoke_sync(
        &self,
        realm_name: Option<&str>,
        namespace: &[&str],
        method_name: &str,
        args: Vec<JsValueFacade>,
    ) -> Result<JsValueFacade, JsError>;

    /// Invoke a function
    /// this returns a Future which will fulfill when the function is done
    /// If None is passed as realm_name the main Realm wil be used
    #[allow(clippy::type_complexity)]
    fn js_function_invoke(
        &self,
        realm_name: Option<&str>,
        namespace: &[&str],
        method_name: &str,
        args: Vec<JsValueFacade>,
    ) -> Pin<Box<dyn Future<Output = Result<JsValueFacade, JsError>>>>;

    /// Invoke a function without waiting for a result
    /// This method may be used instead of js_function_invoke when you don't want to block_on the future or can't .await it
    /// If None is passed as realm_name the main Realm wil be used
    fn js_function_invoke_void(
        &self,
        realm_name: Option<&str>,
        namespace: &[&str],
        method_name: &str,
        args: Vec<JsValueFacade>,
    );
}

/// the JsValueType represents the type of value for a JSValue
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
    Error,
}
