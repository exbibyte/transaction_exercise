use std::collections::{HashMap, HashSet};

use crate::core::*;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ClientData {
    pub avai: Amount,
    pub held: Amount,
    pub total: Amount,
    pub locked: bool,
}

impl From<(&Client, &ClientData)> for Output {
    fn from((client_, data): (&Client, &ClientData)) -> Self {
        Self {
            client: *client_,
            available: data.avai,
            held: data.held,
            total: data.total,
            locked: data.locked,
        }
    }
}

/// Executor for inputs
///
/// Can also possibly instantiate multiple executors on threads and
/// partition work by mapping user id -> executor and use lockfree
/// spsc queues (eg: crossbeam), but will keep it simple for now
#[derive(Default)]
pub struct Executor {
    client_data: HashMap<Client, ClientData>,
    client_record: HashMap<Client, HashSet<Tx>>, //record for only deposits and withdrawls
    record: HashMap<Tx, InputInternal>,          //record for only deposits and withdrawls
}

impl Executor {
    ///process an input
    pub fn process(&mut self, input: Input) {
        let input = InputInternal::from(input);
        match input {
            InputInternal::Deposit(client, tx, amount, _eligible) => {
                let txs = self.client_record.entry(client).or_default();
                //duplicate tx shouldn't occur in input file anyways, so ignore it if it already exists
                if !txs.contains(&tx) {
                    txs.insert(tx);
                    let mut data = self.client_data.entry(client).or_default();
                    if !data.locked {
                        data.avai.0 += amount.0;
                        data.total.0 += amount.0;
                        self.record.insert(tx, input);
                    }
                }
            }
            InputInternal::Withdrawl(client, tx, amount, _eligible) => {
                let txs = self.client_record.entry(client).or_default();
                //duplicate tx shouldn't occur in input file anyways, so ignore it if it already exists
                if !txs.contains(&tx) {
                    let mut data = self.client_data.entry(client).or_default();
                    if data.avai.0 < amount.0 || data.locked {
                        //fail
                    } else {
                        txs.insert(tx);
                        data.avai.0 -= amount.0;
                        data.total.0 -= amount.0;
                        self.record.insert(tx, input);
                    }
                }
            }
            InputInternal::Dispute(client, tx) => {
                //only take first dispute of tx if there are multiple

                if let Some(x) = self.record.get_mut(&tx) {
                    match x {
                        InputInternal::Deposit(client_, _tx, amount, dispute_status) => {
                            if client == *client_
                                && *dispute_status == DisputeStatus::Eligible
                                && !self.client_data.get(&client).unwrap().locked
                            {
                                //undo a deposit may make the balance go into negative territory, just do it anyways since there isn't an explicit rule about it

                                let data = self.client_data.entry(client).or_default();
                                data.avai.0 -= amount.0;
                                data.held.0 += amount.0;
                                *dispute_status = DisputeStatus::Pending;
                            }
                        }
                        InputInternal::Withdrawl(client_, _tx, amount, dispute_status) => {
                            if client == *client_
                                && *dispute_status == DisputeStatus::Eligible
                                && !self.client_data.get(&client).unwrap().locked
                            {
                                let data = self.client_data.entry(client).or_default();
                                data.avai.0 += amount.0;
                                data.held.0 -= amount.0;
                                *dispute_status = DisputeStatus::Pending;
                            }
                        }
                        _ => { //ignore
                        }
                    }
                }
            }
            InputInternal::Resolve(client, tx) => {
                if let Some(x) = self.record.get_mut(&tx) {
                    match x {
                        InputInternal::Deposit(client_, _tx, amount, dispute_status) => {
                            if client == *client_
                                && *dispute_status == DisputeStatus::Pending
                                && !self.client_data.get(&client).unwrap().locked
                            {
                                let data = self.client_data.entry(client).or_default();
                                data.avai.0 += amount.0;
                                data.held.0 -= amount.0;
                                *dispute_status = DisputeStatus::Eligible;
                            }
                        }
                        InputInternal::Withdrawl(client_, _tx, amount, dispute_status) => {
                            if client == *client_
                                && *dispute_status == DisputeStatus::Pending
                                && !self.client_data.get(&client).unwrap().locked
                            {
                                let data = self.client_data.entry(client).or_default();
                                data.avai.0 -= amount.0;
                                data.held.0 += amount.0;
                                *dispute_status = DisputeStatus::Eligible;
                            }
                        }
                        _ => { //ignore
                        }
                    }
                }
            }
            InputInternal::Chargeback(client, tx) => {
                //undo deposit or withdrawl
                if let Some(x) = self.record.get_mut(&tx) {
                    match x {
                        InputInternal::Deposit(client_, _tx, amount, dispute_status) => {
                            if client == *client_
                                && *dispute_status == DisputeStatus::Pending
                                && !self.client_data.get(&client).unwrap().locked
                            {
                                //undo a deposit may make the balance go into negative territory, just do it anyways since there isn't an explicit rule about it

                                let data = self.client_data.entry(client).or_default();
                                data.held.0 -= amount.0;
                                data.total.0 -= amount.0;
                                *dispute_status = DisputeStatus::Complete;
                                self.client_data.get_mut(&client).unwrap().locked = true;
                            }
                        }
                        InputInternal::Withdrawl(client_, _tx, amount, dispute_status) => {
                            if client == *client_
                                && *dispute_status == DisputeStatus::Pending
                                && !self.client_data.get(&client).unwrap().locked
                            {
                                let data = self.client_data.entry(client).or_default();
                                data.held.0 += amount.0;
                                data.total.0 += amount.0;
                                *dispute_status = DisputeStatus::Complete;
                                self.client_data.get_mut(&client).unwrap().locked = true;
                            }
                        }
                        _ => {
                            //ignore
                        }
                    }
                }
            }
        }
    }

    ///return clients' data
    pub fn output(&mut self) -> impl Iterator<Item = Output> + '_ {
        self.client_data
            .iter()
            .map(|(client, data)| Output::from((client, data)))
    }
}
