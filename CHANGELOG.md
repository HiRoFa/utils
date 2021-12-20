# 0.4.0

* added CompiledModuleLoader
* added js_to_str for JSValueAdapter

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
