use criterion::{black_box, criterion_group, criterion_main, Criterion};
// use jr::eval;
use jr::test_impls::{run_j, scan_eval};

pub fn maths1_benchmark(c: &mut Criterion) {
    c.bench_function("2 * i. 1e6", |b| b.iter(|| scan_eval("2 * i. 1e6")));
}
criterion_group!(benches, maths1_benchmark);
criterion_main!(benches);
