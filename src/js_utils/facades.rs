use crate::eventloop::EventLoop;
use crate::js_utils::adapters::{JsContextAdapter, JsRuntimeAdapter, JsValueAdapter};
use crate::js_utils::{JsError, Script};
use std::future::Future;
use std::marker::PhantomData;

pub struct JsProxy {}

pub trait JsRuntimeBuilder {
    type JsRuntimeAdapterType: JsRuntimeAdapter;
    fn build(&self);
    fn with_rt<R, F: FnOnce(&Self::JsRuntimeAdapterType) -> R>(&self, consumer: F) -> R;
}

pub struct JsRuntimeFacade<S: JsRuntimeAdapter> {
    event_loop: EventLoop,
    _phantom: PhantomData<S>,
}

impl<S: JsRuntimeAdapter> JsRuntimeFacade<S> {
    pub fn new(builder: impl JsRuntimeBuilder<JsRuntimeAdapterType = S> + Send + 'static) -> Self {
        let ret = Self {
            event_loop: EventLoop::new(),
            _phantom: Default::default(),
        };

        ret.event_loop.exe(move || {
            // init thread local with builder.build
            // todo move builder to tl var, builder should have a with_rt() fn
            builder.build();
        });

        ret
    }
    pub fn js_create_context(&self, _name: &str) -> JsContextFacade {
        todo!()
    }
    pub fn js_get_main_context(&self) -> &JsContextFacade {
        todo!()
    }
    pub fn js_get_context(&self, _name: &str) -> JsContextFacade {
        todo!()
    }
    pub fn js_loop_exe<R: Send + 'static, C: FnOnce(&S) -> R + Send + 'static>(
        &self,
        _consumer: C,
    ) -> R {
        todo!()
    }
    pub fn js_loop(&self) {
        todo!()
    }
    pub fn js_loop_sync(&self) {
        todo!()
    }
}

pub struct JsContextFacade {}

impl JsContextFacade {
    pub fn js_install_proxy(&self, _js_proxy: JsProxy) {
        todo!()
    }
    pub fn js_eval(
        &self,
        _script: Script,
    ) -> Box<dyn Future<Output = Result<JsValueFacade, JsError>>> {
        todo!()
    }
    /*
    pub fn js_loop_sync<R, C: FnOnce(&dyn JsRuntimeAdapter, &dyn JsContextAdapter) -> R>(
        &self,
        consumer: C,
    ) -> R {
        todo!()
    }
    pub fn js_loop<R, C>(&self) -> Box<dyn Future<Output = Result<R, JsError>>>
    where
        C: FnOnce(&dyn JsRuntimeAdapter, &dyn JsContextAdapter) -> Result<R, JsError>,
    {
        todo!()
    }
    */
    pub fn js_loop_void(&self) {
        todo!()
    }
}

pub struct JsValueFacade {}

impl JsValueFacade {
    pub fn from_js_value_adapter<C: JsContextAdapter + Sized, V: JsValueAdapter + Sized>(
        _ctx: C,
        _value: V,
    ) -> JsValueFacade {
        unimplemented!()
    }
}
