use std::collections::{BTreeSet, HashSet};
use std::time::{Duration, Instant};

const TEXT_EXTENSIONS: &[&str] = &[
    "bash", "conf", "css", "csv", "htm", "html", "js", "json", "md", "py", "rs", "sh", "toml",
    "txt", "xml", "yaml", "yml",
];

const ITERATIONS: usize = 1000;
const TEST_EXTENSIONS: &[&str] = &[
    // Mix of existing and non-existing extensions to test both cases
    "txt",
    "nonexistent",
    "md",
    "fake",
    "py",
    "missing",
    "rs",
    "nope",
    "html",
    "wrong",
    "json",
    "invalid",
    "yaml",
    "nothere",
    "xml",
    "absent",
];

fn benchmark_array() -> Duration {
    let start = Instant::now();

    for _ in 0..ITERATIONS {
        for &ext in TEST_EXTENSIONS {
            let _ = TEXT_EXTENSIONS.contains(&ext);
        }
    }

    start.elapsed()
}

fn benchmark_hashset() -> Duration {
    let extensions: HashSet<_> = TEXT_EXTENSIONS.iter().copied().collect();
    let start = Instant::now();

    for _ in 0..ITERATIONS {
        for &ext in TEST_EXTENSIONS {
            let _ = extensions.contains(ext);
        }
    }

    start.elapsed()
}

fn benchmark_btreeset() -> Duration {
    let extensions: BTreeSet<_> = TEXT_EXTENSIONS.iter().copied().collect();
    let start = Instant::now();

    for _ in 0..ITERATIONS {
        for &ext in TEST_EXTENSIONS {
            let _ = extensions.contains(ext);
        }
    }

    start.elapsed()
}

fn main() {
    // Run each benchmark multiple times to get more reliable results
    const RUNS: usize = 10;

    let mut array_times = Vec::with_capacity(RUNS);
    let mut hashset_times = Vec::with_capacity(RUNS);
    let mut btreeset_times = Vec::with_capacity(RUNS);

    for _ in 0..RUNS {
        array_times.push(benchmark_array());
        hashset_times.push(benchmark_hashset());
        btreeset_times.push(benchmark_btreeset());
    }

    // Calculate and print average times
    let avg_array = array_times.iter().sum::<Duration>() / RUNS as u32;
    let avg_hashset = hashset_times.iter().sum::<Duration>() / RUNS as u32;
    let avg_btreeset = btreeset_times.iter().sum::<Duration>() / RUNS as u32;

    println!(
        "Benchmark Results ({} iterations, {} runs):",
        ITERATIONS, RUNS
    );
    println!("Array:     {:?}", avg_array);
    println!("HashSet:   {:?}", avg_hashset);
    println!("BTreeSet:  {:?}", avg_btreeset);

    // Calculate relative performance
    let fastest = avg_array.min(avg_hashset).min(avg_btreeset);
    println!("\nRelative Performance (lower is better):");
    println!(
        "Array:     {:.2}x",
        avg_array.as_nanos() as f64 / fastest.as_nanos() as f64
    );
    println!(
        "HashSet:   {:.2}x",
        avg_hashset.as_nanos() as f64 / fastest.as_nanos() as f64
    );
    println!(
        "BTreeSet:  {:.2}x",
        avg_btreeset.as_nanos() as f64 / fastest.as_nanos() as f64
    );
}
