use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
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
    let mut group = c.benchmark_group("bitcask-set-bench");
    for size in [128, 256, 512, 1024, 2048, 4096].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
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
                            .set(key, generate_random_bytes(size - key_len))
                            .unwrap();
                    }
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn get_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("bitcask-get-bench");
    for size in [128, 256, 512, 1024, 2048, 4096].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    let temp_dir = TempDir::new().unwrap();
                    let mut store = KvStore::open(temp_dir.path()).unwrap();
                    for i in 1..10000 {
                        let key = format!("key{}", i);
                        let key_len = key.len();

                        store
                            .set(key, generate_random_bytes(size - key_len))
                            .unwrap();
                    }

                    (store, temp_dir)
                },
                |(mut store, _temp_dir)| {
                    for i in 1..10000 {
                        let key = format!("key{}", i);

                        store.get(key);
                    }
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

criterion_group!(benches, set_bench, get_bench);
criterion_main!(benches);
