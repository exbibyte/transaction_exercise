//! entry point for multi core impl
//!
//! execute following before running bench:
//!  cargo run --release --bin generate_data

extern crate crossbeam;
extern crate num_cpus;
extern crate transaction;

use crossbeam::channel::unbounded;
use crossbeam::thread;
use std::env;
use std::error::Error;
use std::io;
use std::path::Path;
use std::process;

pub enum Msg {
    Item(transaction::Input),
    End,
}

fn run(path: &Path) -> Result<(), Box<dyn Error>> {
    let mut reader = csv::Reader::from_path(path)?;

    // let num_workers: usize = 3;
    let num_workers: usize = num_cpus::get();
    // println!("number of cores: {}", num_workers);

    //concurrent msg channels for workers
    let mut channels_sender = vec![];
    let mut channels_receiver = vec![];

    let mut executors = vec![];

    for _ in 0..num_workers {
        let (sender, receiver) = unbounded();
        channels_sender.push(sender);
        channels_receiver.push(receiver);
        executors.push(transaction::Executor::default());
    }

    let executors_finished: Vec<_> = thread::scope(|s| {
        let handle_reader = s.spawn(|_| {
            for result in reader.deserialize() {
                // We must tell Serde what type we want to deserialize into.
                let input: transaction::Input = result.expect("failed to get input");
                //client must be mapped to a same worker in order for result to be correct
                let sender = &channels_sender[input.client.0 as usize % num_workers];
                sender.send(Msg::Item(input)).unwrap();
            }
            for i in &channels_sender {
                i.send(Msg::End).unwrap();
            }
        });

        let mut handles_executors = vec![];

        for (idx, executor) in executors.iter_mut().enumerate() {
            let receiver = channels_receiver[idx].clone();
            let h_executor = s.spawn(move |_| {
                loop {
                    match receiver.recv() {
                        Ok(x) => match x {
                            Msg::Item(item) => {
                                executor.process(item);
                            }
                            _ => {
                                break;
                            }
                        },
                        _ => {
                            panic!("receiver failure");
                        }
                    }
                }
                executor
            });
            handles_executors.push(h_executor);
        }

        handle_reader.join().unwrap();

        handles_executors
            .into_iter()
            .map(|x| x.join().unwrap())
            .collect()
    })
    .unwrap();
    //sync point

    //now write result from executors
    let mut writer = csv::Writer::from_writer(io::stdout());
    for i in executors_finished.into_iter().flat_map(|x| x.output()) {
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
