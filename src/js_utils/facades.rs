use crate::js_utils::adapters::{JsContextAdapter, JsRuntimeAdapter};
use crate::js_utils::facades::JsValueType::*;
use crate::js_utils::{JsError, Script};
use std::future::Future;
use std::pin::Pin;

pub struct JsProxy {}

pub trait JsRuntimeBuilder {
    type JsRuntimeFacadeType: JsRuntimeFacade;
    fn build(self) -> Self::JsRuntimeFacadeType;
}
/// The JsRuntime facade is the main entry point to the JavaScript engine, it is thread safe and
/// handles the logic for transferring data from and to the JsRuntimeAdapter
pub trait JsRuntimeFacade {
    type JsRuntimeAdapterType: JsRuntimeAdapter;
    type JsContextFacadeType: JsContextFacade;

    fn js_create_context(&self, name: &str) -> Self::JsContextFacadeType;
    fn js_get_main_context(&self) -> &Self::JsContextFacadeType;
    fn js_get_context(&self, name: &str) -> &Self::JsContextFacadeType;
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
    ) -> Box<dyn Future<Output = R>>;
    fn js_loop_void<C: FnOnce(&Self::JsRuntimeAdapterType) + Send + 'static>(&self, consumer: C);
}

pub trait JsContextFacade {
    type JsRuntimeFacadeType: JsRuntimeFacade;
    type JsContextAdapterType: JsContextAdapter;

    fn js_install_proxy(&self, js_proxy: JsProxy);

    /// eval a script, please note that eval should not be used for production code, you should always
    /// use modules or functions and invoke them
    /// eval will always need to parse script and some engines like StarLight even require a different syntax (return(1); vs (1);)
    #[allow(clippy::type_complexity)]
    fn js_eval(
        &self,
        script: Script,
    ) -> Pin<Box<dyn Future<Output = Result<Box<dyn JsValueFacade>, JsError>>>>;

    // function methods
    fn js_function_invoke(
        &self,
        namespace: &[&'static str],
        method_name: &'static str,
        args: Vec<Box<dyn JsValueFacade>>,
    ) -> Result<Box<dyn JsValueFacade>, JsError>;

    fn js_loop_sync<
        R : Send + 'static,
        C: FnOnce(&<<Self as JsContextFacade>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType, &Self::JsContextAdapterType) -> R + Send + 'static,
    >(
        &self,
        consumer: C,
    ) -> R;
    fn js_loop<R : Send + 'static, C>(&self, consumer: C) -> Pin<Box<dyn Future<Output = Result<R, JsError>>>>
    where
        C: FnOnce(&<<Self as JsContextFacade>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType, &Self::JsContextAdapterType) -> Result<R, JsError> + Send + 'static;
    fn js_loop_void<C>(&self, consumer: C) where C: FnOnce(&<<Self as JsContextFacade>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType, &Self::JsContextAdapterType) + Send + 'static;
}

/// The JsValueFacade is a thread safe Sendable representation of a variable in the Script engine

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
}

pub trait JsValueFacade: Send + Sync {
    fn js_is_i32(&self) -> bool {
        false
    }
    fn js_is_bool(&self) -> bool {
        false
    }
    fn js_is_null(&self) -> bool {
        false
    }
    fn js_is_undefined(&self) -> bool {
        false
    }
    fn js_is_null_or_undefined(&self) -> bool {
        self.js_is_null() || self.js_is_undefined()
    }
    fn js_as_i32(&self) -> i32 {
        panic!("not an i32");
    }
    fn js_as_bool(&self) -> bool {
        panic!("not a bool");
    }
    fn js_get_type(&self) -> JsValueType;
}

pub struct JsNull {}
pub struct JsUndefined {}

impl JsValueFacade for JsNull {
    fn js_is_null(&self) -> bool {
        true
    }

    fn js_get_type(&self) -> JsValueType {
        Null
    }
}

impl JsValueFacade for JsUndefined {
    fn js_is_undefined(&self) -> bool {
        true
    }
    fn js_get_type(&self) -> JsValueType {
        Undefined
    }
}

impl JsValueFacade for i32 {
    fn js_is_i32(&self) -> bool {
        true
    }

    fn js_as_i32(&self) -> i32 {
        *self
    }

    fn js_get_type(&self) -> JsValueType {
        I32
    }
}

impl JsValueFacade for bool {
    fn js_is_bool(&self) -> bool {
        true
    }

    fn js_as_bool(&self) -> bool {
        *self
    }

    fn js_get_type(&self) -> JsValueType {
        Boolean
    }
}
