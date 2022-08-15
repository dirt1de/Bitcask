use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use kvs::KvStore;
use rand::distributions::Alphanumeric;
use rand::Rng;
use tempfile::TempDir;

fn generate_random_bytes(num_chars: usize) -> String {
    let res: String = std::iter::repeat(())
        .map(|()| rand::thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(num_chars)
        .collect();
    // res.into_bytes()
    res
}

fn set_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_bench");
    group.bench_function("bitcask-set-128b", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                (KvStore::open(temp_dir.path()).unwrap(), temp_dir)
            },
            |(mut store, _temp_dir)| {
                for i in 1..10000 {
                    let key = format!("key{}", i);
                    let key_len = key.len();

                    store
                        .set(key, generate_random_bytes(128 - key_len))
                        .unwrap();
                }
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("bitcask-set-256b", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                (KvStore::open(temp_dir.path()).unwrap(), temp_dir)
            },
            |(mut store, _temp_dir)| {
                for i in 1..10000 {
                    let key = format!("key{}", i);
                    let key_len = key.len();

                    store
                        .set(key, generate_random_bytes(256 - key_len))
                        .unwrap();
                }
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("bitcask-set-512b", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                (KvStore::open(temp_dir.path()).unwrap(), temp_dir)
            },
            |(mut store, _temp_dir)| {
                for i in 1..10000 {
                    let key = format!("key{}", i);
                    let key_len = key.len();

                    store
                        .set(key, generate_random_bytes(512 - key_len))
                        .unwrap();
                }
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("bitcask-set-1kb", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                (KvStore::open(temp_dir.path()).unwrap(), temp_dir)
            },
            |(mut store, _temp_dir)| {
                for i in 1..10000 {
                    let key = format!("key{}", i);
                    let key_len = key.len();

                    store
                        .set(key, generate_random_bytes(1000 - key_len))
                        .unwrap();
                }
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("bitcask-set-2kb", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                (KvStore::open(temp_dir.path()).unwrap(), temp_dir)
            },
            |(mut store, _temp_dir)| {
                for i in 1..10000 {
                    let key = format!("key{}", i);
                    let key_len = key.len();

                    store
                        .set(key, generate_random_bytes(2000 - key_len))
                        .unwrap();
                }
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(benches, set_bench);
criterion_main!(benches);
