# 0.1.2

* fix for nested timeout/interval related actions inside a running timeout/interval

# 0.1.1

* fix for EventLoop which would just run a task in exe_task if it was added from a worker thread, need to check if it is the worke rthread associated by this eventloop or tasks will run in worong thread (happens when nesting EventLoops)

# 0.1.0

initial version
