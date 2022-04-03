use serde::{Deserialize, Serialize};
use std::option::Option;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, Eq, PartialEq)]
pub struct Client(pub u16);

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, Eq, PartialEq)]
pub struct Tx(pub u32);

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
pub struct Amount(pub f32);

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum InputType {
    Deposit,
    Withdrawl,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    #[serde(rename = "type")]
    pub ty: InputType,
    pub client: Client,
    pub tx: Tx,
    pub amount: Option<Amount>,
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum DisputeStatus {
    Eligible, //tx can be disputed, transition to pending
    Pending, //tx can be resolved (transition to eligible), or can be chargeback (transition to complete)
    Complete, //tx can no longer be disputed
}

///internal representation of input
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InputInternal {
    Deposit(Client, Tx, Amount, DisputeStatus),
    Withdrawl(Client, Tx, Amount, DisputeStatus),
    Dispute(Client, Tx),
    Resolve(Client, Tx),
    Chargeback(Client, Tx),
}

impl From<Input> for InputInternal {
    fn from(input: Input) -> Self {
        match input.ty {
            InputType::Deposit => Self::Deposit(
                input.client,
                input.tx,
                input.amount.expect("amount not present"),
                DisputeStatus::Eligible,
            ),
            InputType::Withdrawl => Self::Withdrawl(
                input.client,
                input.tx,
                input.amount.expect("amount not present"),
                DisputeStatus::Eligible,
            ),
            InputType::Dispute => Self::Dispute(input.client, input.tx),
            InputType::Resolve => Self::Resolve(input.client, input.tx),
            InputType::Chargeback => Self::Chargeback(input.client, input.tx),
        }
    }
}

///output client data
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Output {
    pub client: Client,
    pub available: Amount,
    pub held: Amount,
    pub total: Amount,
    pub locked: bool,
}
