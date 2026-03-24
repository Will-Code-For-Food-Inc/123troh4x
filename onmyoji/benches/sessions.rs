use criterion::{black_box, criterion_group, criterion_main, Criterion};
use onmyoji::sessions::{Session, SessionStore};

fn make_session(i: usize) -> Session {
    Session { container_id: format!("ctr-{i}"), platform: "ds".into() }
}

fn bench_session_insert(c: &mut Criterion) {
    c.bench_function("session_insert_sequential", |b| {
        b.iter(|| {
            let store = SessionStore::default();
            for i in 0..100 {
                store.insert(format!("session-{i}"), make_session(black_box(i)));
            }
        })
    });
}

fn bench_session_get(c: &mut Criterion) {
    let store = SessionStore::default();
    for i in 0..100 {
        store.insert(format!("session-{i}"), make_session(i));
    }

    c.bench_function("session_get_sequential", |b| {
        b.iter(|| {
            for i in 0..100 {
                let _ = store.get(black_box(&format!("session-{i}")));
            }
        })
    });
}

fn bench_session_concurrent(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    c.bench_function("session_concurrent_8_threads", |b| {
        b.iter(|| {
            let store = Arc::new(SessionStore::default());
            let mut handles = vec![];
            for t in 0..8 {
                let s = store.clone();
                handles.push(thread::spawn(move || {
                    for i in 0..50 {
                        let key = format!("t{t}-s{i}");
                        s.insert(key.clone(), make_session(i));
                        let _ = s.get(&key);
                    }
                }));
            }
            for h in handles { h.join().unwrap(); }
        })
    });
}

criterion_group!(benches, bench_session_insert, bench_session_get, bench_session_concurrent);
criterion_main!(benches);
