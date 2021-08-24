use crate::js_utils::adapters::{JsPromiseAdapter, JsRealmAdapter, JsRuntimeAdapter};
use crate::js_utils::facades::async_utils::{add_helper_task, add_helper_task_async};
use crate::js_utils::facades::{JsRuntimeFacade, JsRuntimeFacadeInner};
use crate::js_utils::JsError;
use futures::Future;

#[allow(clippy::type_complexity)]
pub fn new_resolving_promise<P, R, M, T>(
    realm: &T,
    producer: P,
    mapper: M,
) -> Result<T::JsValueAdapterType, JsError>
where
    T: JsRealmAdapter + 'static,
    R: Send + 'static,
    P: FnOnce() -> Result<R, String> + Send + 'static,
    M: FnOnce(&<<<<<<T as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType, R) -> Result<<<<<<<<T as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType as JsRealmAdapter>::JsValueAdapterType, JsError> + Send + 'static,
{
    // create promise
    let promise_ref = realm.js_promise_create()?;
    let return_ref = promise_ref.js_promise_get_value(realm);

    // add to map and keep id
    let id = realm.js_promise_cache_add(promise_ref);

    let rti_ref = realm.js_get_runtime_facade_inner();

    let realm_id = realm.js_get_realm_id().to_string();
    // go async
    add_helper_task(move || {
        // in helper thread, produce result
        let produced_result = producer();
        let rti = rti_ref.upgrade().expect("invalid state"); // todo ignore this async result then
        rti.js_add_rt_task_to_event_loop_void(move |rt| {
            let realm = rt.js_get_realm(realm_id.as_str()).expect("no such realm");
            // in q_js_rt worker thread, resolve promise
            // retrieve promise
            let prom_ref: Box<(dyn JsPromiseAdapter<<<<<<<T as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType> + 'static)> = realm.js_promise_cache_consume(id);
            //let prom_ref = realm.js_promise_cache_consume(id);
            match produced_result {
                Ok(ok_res) => {
                    // map result to JSValueRef
                    let raw_res = mapper(realm, ok_res);

                    // resolve or reject promise
                    match raw_res {
                        Ok(val_ref) => {
                            prom_ref
                                .js_promise_resolve(realm, &val_ref)
                                .ok()
                                .expect("prom resolution failed");
                        }
                        Err(err) => {
                            let err_ref = realm
                                .js_string_create(err.get_message())
                                .ok()
                                .expect("could not create str");
                            prom_ref
                                .js_promise_reject(realm, &err_ref)
                                .ok()
                                .expect("prom rejection failed");
                        }
                    }
                }
                Err(err) => {
                    // todo use error:new_error(err)
                    let err_ref = realm
                        .js_string_create(err.as_str())
                        .ok()
                        .expect("could not create str");
                    prom_ref
                        .js_promise_reject(realm, &err_ref)
                        .ok()
                        .expect("prom rejection failed");
                }
            }
        });
    });

    Ok(return_ref)
}

#[allow(clippy::type_complexity)]
pub(crate) fn new_resolving_promise_async<P, R, M, T>(
    realm: &T,
    producer: P,
    mapper: M,
) -> Result<T::JsValueAdapterType, JsError>
    where
        T: JsRealmAdapter + 'static,
        R: Send + 'static,
        P: Future<Output = Result<R, String>> + Send + 'static,
        M: FnOnce(&<<<<<<T as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType, R) -> Result<<<<<<<<T as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType as JsRealmAdapter>::JsValueAdapterType, JsError> + Send + 'static,
{
    // create promise
    let promise_ref = realm.js_promise_create()?;
    let return_ref = promise_ref.js_promise_get_value(realm);

    // add to map and keep id
    let id = realm.js_promise_cache_add(promise_ref);

    let rti_ref = realm.js_get_runtime_facade_inner();

    let realm_id = realm.js_get_realm_id().to_string();
    // go async
    let _ = add_helper_task_async(async move {
        // in helper thread, produce result
        let produced_result = producer.await;
        let rti = rti_ref.upgrade().expect("invalid state"); // todo ignore this async result then
        rti.js_add_rt_task_to_event_loop_void(move |rt| {
            let realm = rt.js_get_realm(realm_id.as_str()).expect("no such realm");
            // in q_js_rt worker thread, resolve promise
            // retrieve promise
            let prom_ref: Box<(dyn JsPromiseAdapter<<<<<<<T as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType> + 'static)> = realm.js_promise_cache_consume(id);
            //let prom_ref = realm.js_promise_cache_consume(id);
            match produced_result {
                Ok(ok_res) => {
                    // map result to JSValueRef
                    let raw_res = mapper(realm, ok_res);

                    // resolve or reject promise
                    match raw_res {
                        Ok(val_ref) => {
                            prom_ref
                                .js_promise_resolve(realm, &val_ref)
                                .ok()
                                .expect("prom resolution failed");
                        }
                        Err(err) => {
                            let err_ref = realm
                                .js_string_create(err.get_message())
                                .ok()
                                .expect("could not create str");
                            prom_ref
                                .js_promise_reject(realm, &err_ref)
                                .ok()
                                .expect("prom rejection failed");
                        }
                    }
                }
                Err(err) => {
                    // todo use error:new_error(err)
                    let err_ref = realm
                        .js_string_create(err.as_str())
                        .ok()
                        .expect("could not create str");
                    prom_ref
                        .js_promise_reject(realm, &err_ref)
                        .ok()
                        .expect("prom rejection failed");
                }
            }
        });
    });
    Ok(return_ref)
}