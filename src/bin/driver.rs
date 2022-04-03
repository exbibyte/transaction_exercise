//! entry point for single core impl
//!
//! execute following before running bench:
//!  cargo run --release --bin generate_data

use std::env;
use std::error::Error;
use std::io;
use std::path::Path;
use std::process;

fn run(path: &Path) -> Result<(), Box<dyn Error>> {
    let mut reader = csv::Reader::from_path(path)?;

    let mut executor = transaction::Executor::default();
    for result in reader.deserialize() {
        // We must tell Serde what type we want to deserialize into.
        let input: transaction::Input = result?;
        executor.process(input);
    }

    let mut writer = csv::Writer::from_writer(io::stdout());

    for i in executor.output() {
        writer.serialize(i)?;
    }

    writer.flush()?;

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 2, "please provide input file path");
    let file = &args[1];
    let path = Path::new(&file);
    if let Err(err) = run(path) {
        println!("failure: {:?}", err);
        process::exit(1);
    }
}
