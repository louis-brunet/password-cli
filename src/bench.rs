use std::time::SystemTime;

pub fn bench(func: impl Fn(), bench_name: &str) {
    return bench_with_counts(func, bench_name, &[1, 10, 100, 1000, 10000]);
}

pub fn bench_with_counts(func: impl Fn(), bench_name: &str, iter_counts: &[usize]) {
    let start = SystemTime::now();
    for &iter_count in iter_counts {
        // let iter_count = 10000;
        for _ in 0..iter_count {
            func();
        }
        println!(
            "Bench {:?}: {:?}ms / iter for {:?} iterations",
            bench_name,
            start.elapsed().unwrap().as_micros() as f64 / (iter_count as f64 * 1000f64),
            iter_count
        );
    }
}
