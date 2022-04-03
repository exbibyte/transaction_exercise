//! execute following before running bench:
//!  cargo run --release --bin generate_data

use criterion::{criterion_group, criterion_main, Criterion};
// use std::io;
use std::path::Path;

fn run() {
    let file = "./sample_input.txt";
    let path = Path::new(&file);

    let mut reader = csv::Reader::from_path(path).unwrap();

    let mut executor = transaction::Executor::default();

    // let mut writer = csv::Writer::from_writer(io::stdout());

    for result in reader.deserialize() {
        // We must tell Serde what type we want to deserialize into.
        let input: transaction::Input = result.unwrap();
        executor.process(input);
    }

    // for i in executor.output() {
    //     writer.serialize(i).unwrap();
    // }

    // writer.flush().unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("transaction singlecore", |b| b.iter(run));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
