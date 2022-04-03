//! used to generate test data

use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashSet;
use std::error::Error;
use std::ffi::OsString;
use std::ops::Range;
use std::path::Path;
use std::process;

extern crate transaction;

pub static ALL_INPUT_TYPES: [transaction::InputType; 5] = [
    transaction::InputType::Deposit,
    transaction::InputType::Withdrawl,
    transaction::InputType::Dispute,
    transaction::InputType::Resolve,
    transaction::InputType::Chargeback,
];

/// convenience for creating an input
pub struct InputBuilder {
    bound_client: Range<transaction::Client>,
    bound_tx: Range<transaction::Tx>,
    bound_amount: Range<transaction::Amount>,
    tx_used: HashSet<transaction::Tx>,
}

impl InputBuilder {
    pub fn new(
        client_range: Range<transaction::Client>,
        tx_range: Range<transaction::Tx>,
        amount_range: Range<transaction::Amount>,
    ) -> InputBuilder {
        InputBuilder {
            bound_client: client_range,
            bound_tx: tx_range,
            bound_amount: amount_range,
            tx_used: Default::default(),
        }
    }
    pub fn sample_random(&mut self) -> transaction::Input {
        let mut rng = rand::thread_rng();

        let client = rng.gen_range(self.bound_client.start.0..self.bound_client.end.0);

        let ty = ALL_INPUT_TYPES.choose(&mut rng).unwrap();

        let (tx, amnt) = match ty {
            transaction::InputType::Deposit | transaction::InputType::Withdrawl => {
                //get a unique tx id
                let obtained = loop {
                    let candidate =
                        transaction::Tx(rng.gen_range(self.bound_tx.start.0..self.bound_tx.end.0));
                    if !self.tx_used.contains(&candidate) {
                        break candidate;
                    }
                };
                self.tx_used.insert(obtained);
                (
                    obtained,
                    Some(transaction::Amount(rng.gen_range(
                        self.bound_amount.start.0..self.bound_amount.end.0,
                    ))),
                )
            }
            _ => (
                transaction::Tx(rng.gen_range(self.bound_tx.start.0..self.bound_tx.end.0)),
                None,
            ),
        };

        transaction::Input {
            ty: *ty,
            client: transaction::Client(client),
            tx,
            amount: amnt, //don't care for resolve or chargeback
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    //create a write to a file
    let path: OsString = Path::new("./sample_input.txt").into();
    let mut wtr = csv::Writer::from_path(path)?;

    //bound this for testing
    let mut input_builder = InputBuilder::new(
        transaction::Client(0)..transaction::Client(1000),
        transaction::Tx(0)..transaction::Tx(u32::MAX),
        transaction::Amount(-999.)..transaction::Amount(999.),
    );

    let num_inputs = 10_000_000;

    for _ in 0..num_inputs {
        let input = input_builder.sample_random();
        wtr.serialize(input)?;
    }
    wtr.flush()?;
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        println!("failure: {:?}", err);
        process::exit(1);
    }
}
