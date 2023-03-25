use crate::js_utils::adapters::{JsPromiseAdapter, JsRealmAdapter, JsRuntimeAdapter};
use crate::js_utils::facades::async_utils::{add_helper_task, add_helper_task_async};
use crate::js_utils::facades::{JsRuntimeFacade, JsRuntimeFacadeInner};
use crate::js_utils::JsError;
use futures::Future;

#[allow(clippy::type_complexity)]
/// create a new promise with a producer and a mapper
/// the producer will run in a helper thread(in the tokio thread pool) and thus get a result asynchronously
/// the resulting value will then be mapped to a JSValueRef by the mapper in the EventQueue thread
/// the promise which was returned is then resolved with the value which is returned by the mapper
pub fn new_resolving_promise<P, R, M, T>(
    realm: &T,
    producer: P,
    mapper: M,
) -> Result<T::JsValueAdapterType, JsError>
    where
        T: JsRealmAdapter + 'static,
        R: Send + 'static,
        P: FnOnce() -> Result<R, JsError> + Send + 'static,
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
        if let Some(rti) = rti_ref.upgrade() {
            rti.js_add_rt_task_to_event_loop_void(move |rt| {
                if let Some(realm) = rt.js_get_realm(realm_id.as_str()) {

                    // in q_js_rt worker thread, resolve promise
                    // retrieve promise
                    let prom_ref_opt: Option<Box<(dyn JsPromiseAdapter<<<<<<<T as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType> + 'static)>> = realm.js_promise_cache_consume(id);
                    if let Some(prom_ref) = prom_ref_opt {
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
                                            .expect("prom resolution failed");
                                    }
                                    Err(err) => {
                                        let err_ref = realm
                                            .js_error_create(err.get_name(), err.get_message(), err.get_stack())
                                            .expect("could not create str");
                                        prom_ref
                                            .js_promise_reject(realm, &err_ref)
                                            .expect("prom rejection failed");
                                    }
                                }
                            }
                            Err(err) => {
                                // todo use error:new_error(err)
                                let err_ref = realm
                                    .js_error_create(err.get_name(), err.get_message(), err.get_stack())
                                    .expect("could not create str");
                                prom_ref
                                    .js_promise_reject(realm, &err_ref)
                                    .expect("prom rejection failed");
                            }
                        }
                    } else {
                        log::error!("async promise running for dropped realm: {} promise_id:{}", realm_id, id);
                    }
                } else {
                    log::error!("async promise running for dropped realm: {}", realm_id);
                }
            });
        } else {
            log::error!("async promise running for dropped runtime");
        }
    });

    Ok(return_ref)
}

#[allow(clippy::type_complexity)]
/// create a new promise with an async producer and a mapper
/// the producer will be awaited asynchronously and
/// the resulting value will then be mapped to a JSValueRef by the mapper in the EventQueue thread
/// the promise which was returned is then resolved with the value which is returned by the mapper
pub(crate) fn new_resolving_promise_async<P, R, M, T>(
    realm: &T,
    producer: P,
    mapper: M,
) -> Result<T::JsValueAdapterType, JsError>
    where
        T: JsRealmAdapter + 'static,
        R: Send + 'static,
        P: Future<Output=Result<R, JsError>> + Send + 'static,
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
    let _ignore_result = add_helper_task_async(async move {
        // in helper thread, produce result
        let produced_result = producer.await;
        if let Some(rti) = rti_ref.upgrade() {
            rti.js_add_rt_task_to_event_loop_void(move |rt| {
                if let Some(realm) = rt.js_get_realm(realm_id.as_str()) {
                    // in q_js_rt worker thread, resolve promise
                    // retrieve promise
                    let prom_ref_opt: Option<Box<(dyn JsPromiseAdapter<<<<<<<T as JsRealmAdapter>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeFacadeInnerType as JsRuntimeFacadeInner>::JsRuntimeFacadeType as JsRuntimeFacade>::JsRuntimeAdapterType as JsRuntimeAdapter>::JsRealmAdapterType> + 'static)>> = realm.js_promise_cache_consume(id);
                    if let Some(prom_ref) = prom_ref_opt {
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
                                            .expect("prom resolution failed");
                                    }
                                    Err(err) => {
                                        let err_ref = realm
                                            .js_error_create(err.get_name(), err.get_message(), err.get_stack())
                                            .expect("could not create err");
                                        prom_ref
                                            .js_promise_reject(realm, &err_ref)
                                            .expect("prom rejection failed");
                                    }
                                }
                            }
                            Err(err) => {
                                // todo use error:new_error(err)
                                let err_ref = realm
                                    .js_error_create(err.get_name(), err.get_message(), err.get_stack())
                                    .expect("could not create str");
                                prom_ref
                                    .js_promise_reject(realm, &err_ref)
                                    .expect("prom rejection failed");
                            }
                        }
                    } else {
                        log::error!("async promise running on dropped realm: {} promise_id:{}", realm_id, id);
                    }
                } else {
                    log::error!("async promise running on dropped realm: {}", realm_id);
                }
            });
        } else {
            log::error!("async promise running on dropped runtime");
        }
    });
    Ok(return_ref)
}
