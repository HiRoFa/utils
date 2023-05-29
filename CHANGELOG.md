# 0.7.1

* use flume for channels
* use parking_lot for Mutex

# 0.7.0

* removed js_utils

# 0.6.1

* updated autoidmap so it is Send again

# 0.6.0

* AutoIdMap IDs are now random instead of sequential
* JSRealmAdapter::promise_cache_consume now returns an Option instead of panic on not found

# 0.5.7

* impl JsValueConvertable::to_jsValue_facade for serde::Value
* impl JsValueFacade::to_json_string for return types like JsValueFacade::JsObject (not input types like JsValueFacade::Object>)
* impl JsValueFacade::to_serde_value

# 0.5.6

* impl from_serializable for JsValueFacade for creating a JsValueFacade::JsonStr from a Serializable struct
* impl JsValueFacade::SerdeValue for creating a JSValue from a serde::Value

# 0.5.5

* impl Error for JsError

# 0.5.4

* use owned string for DefaultAtom instantiation 

# 0.5.3

* use DefaultAtom for string in JsValueFacade
* added remove_opt to AutoIdMap
* fixed race condition in ResolvableFuture which could lead to Futures/Promises never resolving

# 0.5.2

* added JsValueFacade::JsonStr to pass a json string which is parsed when converted to JsValueAdapter

# 0.5.1

* added code to transform typed array to JsValueFacade

# 0.5.0

* proxy instance info functions
* altered realm_init_hook signature to Fn from FnOnce

# 0.4.1 

added realm.js_function_create_async

# 0.4.0

* added CompiledModuleLoader
* added js_to_str for JSValueAdapter
* added typed array support (UInt8 only for now)

# 0.3

* bumped version for publish

# 0.2.2 

* small optimization to add_void in EventLoop
* added js_get_script_or_module_name to JsRealmAdapter
* removed fetch api (moved to GreCo)
* added script_preproc to builder
* added js_load_module_script to runtime

# 0.2.1

* removed default impl for js_loop_realm* 
* fixed typedef for to_js_value_facade()

# 0.2.0

* working on facades and adapters for JsEngines
  * renamed Context to Realm
  * replaced ContextFacade with methods in RuntimeFacade

# 0.1.3

* fix for inaccurate timing when adding timeouts from timeouts

# 0.1.2

* fix for nested timeout/interval related actions inside a running timeout/interval

# 0.1.1

* fix for EventLoop which would just run a task in exe_task if it was added from a worker thread, need to check if it is the worke rthread associated by this eventloop or tasks will run in worong thread (happens when nesting EventLoops)

# 0.1.0

initial version
