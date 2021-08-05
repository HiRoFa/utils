use crate::js_utils::adapters::JsRuntimeAdapter;
use crate::js_utils::{JsError, Script};
use std::future::Future;
use std::pin::Pin;
use std::sync::Weak;

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
}
/// The JsRuntime facade is the main entry point to the JavaScript engine, it is thread safe and
/// handles the logic for transferring data from and to the JsRuntimeAdapter
pub trait JsRuntimeFacade {
    type JsRuntimeAdapterType: JsRuntimeAdapter;
    type JsRuntimeFacadeInnerType: JsRuntimeFacadeInner;

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

#[derive(PartialEq)]
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

pub trait JsValueFacade: Send + Sync {
    fn js_is_null_or_undefined(&self) -> bool {
        self.js_get_type() == JsValueType::Null || self.js_get_type() == JsValueType::Undefined
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
}

pub struct JsNull {}
pub struct JsUndefined {}

impl JsValueFacade for JsNull {
    fn js_get_type(&self) -> JsValueType {
        JsValueType::Null
    }
}

impl JsValueFacade for JsUndefined {
    fn js_get_type(&self) -> JsValueType {
        JsValueType::Undefined
    }
}

impl JsValueFacade for i32 {
    fn js_as_i32(&self) -> i32 {
        *self
    }

    fn js_get_type(&self) -> JsValueType {
        JsValueType::I32
    }
}

impl JsValueFacade for f64 {
    fn js_as_f64(&self) -> f64 {
        *self
    }

    fn js_get_type(&self) -> JsValueType {
        JsValueType::F64
    }
}

impl JsValueFacade for bool {
    fn js_as_bool(&self) -> bool {
        *self
    }

    fn js_get_type(&self) -> JsValueType {
        JsValueType::Boolean
    }
}

impl JsValueFacade for String {
    fn js_as_str(&self) -> &str {
        self.as_str()
    }

    fn js_get_type(&self) -> JsValueType {
        JsValueType::String
    }
}
