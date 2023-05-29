

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use futures::executor::block_on;

use hirofa_utils::eventloop::EventLoop;
use hirofa_utils::resolvable_future::ResolvableFuture;
use hirofa_utils::task_manager::TaskManager;

fn test_eventloop_exe(){

    let event_loop = EventLoop::new();
    for x in 0..5000 {
        let y = x;
        event_loop.exe(move || {
            black_box(y);
        });
    }

}

fn test_res_fut(){

    let tm = TaskManager::new(4);

    for _x in 0..5000 {

        let rf = ResolvableFuture::new();
        let resolver = rf.get_resolver();
        tm.add_task(move || {
            resolver.resolve("hi".to_string()).expect("wtf");
        });
        let s= block_on(rf);
        assert_eq!(s.len(), 2);

    }

}


pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("eventLoop.exe", |b| b.iter(|| test_eventloop_exe()));
    c.bench_function("test_res_fut", |b| b.iter(|| test_res_fut()));

}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);