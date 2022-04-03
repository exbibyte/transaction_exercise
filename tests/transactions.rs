#[cfg(test)]
use std::collections::HashMap;

#[test]
fn transaction_deposits() {
    use transaction::*;

    let inputs = [
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(1),
            amount: Some(Amount(5.)),
        },
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(2),
            amount: Some(Amount(7.)),
        },
    ];
    let mut executor = Executor::default();
    for i in inputs {
        executor.process(i);
    }
    let out: Vec<_> = executor.output().collect();
    assert_eq!(out.len(), 1);
    let item = out.get(0).unwrap();

    let expected = Output {
        client: Client(1),
        available: Amount(12.),
        held: Amount(0.),
        total: Amount(12.),
        locked: false,
    };
    assert_eq!(item, &expected);
}

#[test]
fn transaction_deposit_withdrawl() {
    use transaction::*;

    let inputs = [
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(1),
            amount: Some(Amount(5.)),
        },
        Input {
            ty: InputType::Withdrawl,
            client: Client(1),
            tx: Tx(2),
            amount: Some(Amount(3.)),
        },
    ];
    let mut executor = Executor::default();
    for i in inputs {
        executor.process(i);
    }
    let out: Vec<_> = executor.output().collect();
    assert_eq!(out.len(), 1);
    let item = out.get(0).unwrap();
    let expected = Output {
        client: Client(1),
        available: Amount(2.),
        held: Amount(0.),
        total: Amount(2.),
        locked: false,
    };
    assert_eq!(item, &expected);
}

#[test]
fn transaction_deposit_withdrawl_insufficient_fund() {
    use transaction::*;

    let inputs = [
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(1),
            amount: Some(Amount(5.)),
        },
        Input {
            ty: InputType::Withdrawl,
            client: Client(1),
            tx: Tx(2),
            amount: Some(Amount(8.)),
        },
    ];
    let mut executor = Executor::default();
    for i in inputs {
        executor.process(i);
    }
    let out: Vec<_> = executor.output().collect();
    assert_eq!(out.len(), 1);
    let item = out.get(0).unwrap();
    let expected = Output {
        client: Client(1),
        available: Amount(5.),
        held: Amount(0.),
        total: Amount(5.),
        locked: false,
    };
    assert_eq!(item, &expected);
}

#[test]
fn transaction_deposit_withdrawl_insufficient_fund_2() {
    use transaction::*;

    let inputs = [
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(1),
            amount: Some(Amount(5.)),
        },
        Input {
            ty: InputType::Withdrawl,
            client: Client(2),
            tx: Tx(2),
            amount: Some(Amount(8.)),
        },
    ];
    let mut executor = Executor::default();
    for i in inputs {
        executor.process(i);
    }
    let out: Vec<_> = executor.output().collect();
    assert_eq!(out.len(), 2);

    let mut hm = HashMap::new();

    for i in out {
        hm.insert(i.client, i);
    }
    assert!(hm.contains_key(&Client(1)));
    assert!(hm.contains_key(&Client(2)));

    assert_eq!(
        hm.get(&Client(1)).unwrap(),
        &Output {
            client: Client(1),
            available: Amount(5.),
            held: Amount(0.),
            total: Amount(5.),
            locked: false,
        }
    );
    assert_eq!(
        hm.get(&Client(2)).unwrap(),
        &Output {
            client: Client(2),
            available: Amount(0.),
            held: Amount(0.),
            total: Amount(0.),
            locked: false,
        }
    );
}

#[test]
fn transaction_withdrawl_invalid() {
    use transaction::*;

    let inputs = [Input {
        ty: InputType::Withdrawl,
        client: Client(1),
        tx: Tx(1),
        amount: Some(Amount(8.)),
    }];
    let mut executor = Executor::default();
    for i in inputs {
        executor.process(i);
    }
    let out: Vec<_> = executor.output().collect();
    assert_eq!(out.len(), 1);
    let item = out.get(0).unwrap();
    let expected = Output {
        client: Client(1),
        available: Amount(0.),
        held: Amount(0.),
        total: Amount(0.),
        locked: false,
    };
    assert_eq!(item, &expected);
}

#[test]
fn transaction_dispute_for_withdrawl() {
    use transaction::*;

    let inputs = [
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(1),
            amount: Some(Amount(5.)),
        },
        Input {
            ty: InputType::Withdrawl,
            client: Client(1),
            tx: Tx(2),
            amount: Some(Amount(3.)),
        },
        Input {
            ty: InputType::Dispute,
            client: Client(1),
            tx: Tx(2),
            amount: None,
        },
    ];
    let mut executor = Executor::default();
    for i in inputs {
        executor.process(i);
    }
    let out: Vec<_> = executor.output().collect();
    assert_eq!(out.len(), 1);
    let item = out.get(0).unwrap();
    let expected = Output {
        client: Client(1),
        available: Amount(5.),
        held: Amount(-3.),
        total: Amount(2.),
        locked: false,
    };
    assert_eq!(item, &expected);
}

#[test]
fn transaction_dispute_for_deposit() {
    use transaction::*;

    let inputs = [
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(1),
            amount: Some(Amount(5.)),
        },
        Input {
            ty: InputType::Withdrawl,
            client: Client(1),
            tx: Tx(2),
            amount: Some(Amount(3.)),
        },
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(3),
            amount: Some(Amount(10.)),
        },
        Input {
            ty: InputType::Dispute,
            client: Client(1),
            tx: Tx(1),
            amount: None,
        },
    ];
    let mut executor = Executor::default();
    for i in inputs {
        executor.process(i);
    }
    let out: Vec<_> = executor.output().collect();
    assert_eq!(out.len(), 1);
    let item = out.get(0).unwrap();
    let expected = Output {
        client: Client(1),
        available: Amount(7.),
        held: Amount(5.),
        total: Amount(12.),
        locked: false,
    };
    assert_eq!(item, &expected);
}

#[test]
fn transaction_multiple_disputes_for_same_transaction() {
    use transaction::*;

    let inputs = [
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(1),
            amount: Some(Amount(5.)),
        },
        Input {
            ty: InputType::Withdrawl,
            client: Client(1),
            tx: Tx(2),
            amount: Some(Amount(3.)),
        },
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(3),
            amount: Some(Amount(10.)),
        },
        Input {
            ty: InputType::Dispute,
            client: Client(1),
            tx: Tx(1),
            amount: None,
        },
        Input {
            //duplicate dispute should be idempotent
            ty: InputType::Dispute,
            client: Client(1),
            tx: Tx(1),
            amount: None,
        },
    ];
    let mut executor = Executor::default();
    for i in inputs {
        executor.process(i);
    }
    let out: Vec<_> = executor.output().collect();
    assert_eq!(out.len(), 1);
    let item = out.get(0).unwrap();
    let expected = Output {
        client: Client(1),
        available: Amount(7.),
        held: Amount(5.),
        total: Amount(12.),
        locked: false,
    };
    assert_eq!(item, &expected);
}

#[test]
fn transaction_resolve_for_withdrawl() {
    use transaction::*;

    let inputs = [
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(1),
            amount: Some(Amount(5.)),
        },
        Input {
            ty: InputType::Withdrawl,
            client: Client(1),
            tx: Tx(2),
            amount: Some(Amount(3.)),
        },
        Input {
            ty: InputType::Dispute,
            client: Client(1),
            tx: Tx(2),
            amount: None,
        },
        Input {
            ty: InputType::Resolve,
            client: Client(1),
            tx: Tx(2),
            amount: None,
        },
    ];
    let mut executor = Executor::default();
    for i in inputs {
        executor.process(i);
    }
    let out: Vec<_> = executor.output().collect();
    assert_eq!(out.len(), 1);
    let item = out.get(0).unwrap();
    let expected = Output {
        client: Client(1),
        available: Amount(2.),
        held: Amount(0.),
        total: Amount(2.),
        locked: false,
    };
    assert_eq!(item, &expected);
}

#[test]
fn transaction_resolve_for_deposit() {
    use transaction::*;

    let inputs = [
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(1),
            amount: Some(Amount(5.)),
        },
        Input {
            ty: InputType::Withdrawl,
            client: Client(1),
            tx: Tx(2),
            amount: Some(Amount(3.)),
        },
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(3),
            amount: Some(Amount(10.)),
        },
        Input {
            ty: InputType::Dispute,
            client: Client(1),
            tx: Tx(1),
            amount: None,
        },
        Input {
            ty: InputType::Resolve,
            client: Client(1),
            tx: Tx(1),
            amount: None,
        },
    ];
    let mut executor = Executor::default();
    for i in inputs {
        executor.process(i);
    }
    let out: Vec<_> = executor.output().collect();
    assert_eq!(out.len(), 1);
    let item = out.get(0).unwrap();
    let expected = Output {
        client: Client(1),
        available: Amount(12.),
        held: Amount(0.),
        total: Amount(12.),
        locked: false,
    };
    assert_eq!(item, &expected);
}

#[test]
fn transaction_chargeback_for_withdrawl() {
    use transaction::*;

    let inputs = [
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(1),
            amount: Some(Amount(5.)),
        },
        Input {
            ty: InputType::Withdrawl,
            client: Client(1),
            tx: Tx(2),
            amount: Some(Amount(3.)),
        },
        Input {
            ty: InputType::Dispute,
            client: Client(1),
            tx: Tx(2),
            amount: None,
        },
        Input {
            ty: InputType::Chargeback,
            client: Client(1),
            tx: Tx(2),
            amount: None,
        },
    ];
    let mut executor = Executor::default();
    for i in inputs {
        executor.process(i);
    }
    let out: Vec<_> = executor.output().collect();
    assert_eq!(out.len(), 1);
    let item = out.get(0).unwrap();
    let expected = Output {
        client: Client(1),
        available: Amount(5.),
        held: Amount(0.),
        total: Amount(5.),
        locked: true,
    };
    assert_eq!(item, &expected);
}

#[test]
fn transaction_resolve_and_chargeback_for_withdrawl() {
    use transaction::*;

    let inputs = [
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(1),
            amount: Some(Amount(5.)),
        },
        Input {
            ty: InputType::Withdrawl,
            client: Client(1),
            tx: Tx(2),
            amount: Some(Amount(3.)),
        },
        Input {
            ty: InputType::Dispute,
            client: Client(1),
            tx: Tx(2),
            amount: None,
        },
        Input {
            ty: InputType::Resolve,
            client: Client(1),
            tx: Tx(2),
            amount: None,
        },
        //dispute it again
        Input {
            ty: InputType::Dispute,
            client: Client(1),
            tx: Tx(2),
            amount: None,
        },
        Input {
            ty: InputType::Chargeback,
            client: Client(1),
            tx: Tx(2),
            amount: None,
        },
    ];
    let mut executor = Executor::default();
    for i in inputs {
        executor.process(i);
    }
    let out: Vec<_> = executor.output().collect();
    assert_eq!(out.len(), 1);
    let item = out.get(0).unwrap();
    let expected = Output {
        client: Client(1),
        available: Amount(5.),
        held: Amount(0.),
        total: Amount(5.),
        locked: true,
    };
    assert_eq!(item, &expected);
}

#[test]
fn transaction_chargeback_for_deposit() {
    use transaction::*;

    let inputs = [
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(1),
            amount: Some(Amount(5.)),
        },
        Input {
            ty: InputType::Withdrawl,
            client: Client(1),
            tx: Tx(2),
            amount: Some(Amount(3.)),
        },
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(3),
            amount: Some(Amount(10.)),
        },
        Input {
            ty: InputType::Dispute,
            client: Client(1),
            tx: Tx(1),
            amount: None,
        },
        Input {
            ty: InputType::Chargeback,
            client: Client(1),
            tx: Tx(1),
            amount: None,
        },
    ];
    let mut executor = Executor::default();
    for i in inputs {
        executor.process(i);
    }
    let out: Vec<_> = executor.output().collect();
    assert_eq!(out.len(), 1);
    let item = out.get(0).unwrap();
    let expected = Output {
        client: Client(1),
        available: Amount(7.),
        held: Amount(0.),
        total: Amount(7.),
        locked: true,
    };
    assert_eq!(item, &expected);
}

#[test]
fn transaction_activity_after_locked() {
    use transaction::*;

    let inputs = [
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(1),
            amount: Some(Amount(5.)),
        },
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(2),
            amount: Some(Amount(10.)),
        },
        Input {
            ty: InputType::Dispute,
            client: Client(1),
            tx: Tx(1),
            amount: None,
        },
        Input {
            ty: InputType::Chargeback,
            client: Client(1),
            tx: Tx(1),
            amount: None,
        },
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(3),
            amount: Some(Amount(20.)),
        },
    ];
    let mut executor = Executor::default();
    for i in inputs {
        executor.process(i);
    }
    let out: Vec<_> = executor.output().collect();
    assert_eq!(out.len(), 1);
    let item = out.get(0).unwrap();
    let expected = Output {
        client: Client(1),
        available: Amount(10.),
        held: Amount(0.),
        total: Amount(10.),
        locked: true,
    };
    assert_eq!(item, &expected);
}

#[test]
fn transaction_duplicate_tx_id() {
    use transaction::*;

    let inputs = [
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(1),
            amount: Some(Amount(5.)),
        },
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(2),
            amount: Some(Amount(10.)),
        },
        Input {
            ty: InputType::Deposit,
            client: Client(1),
            tx: Tx(1), //duplcate id shouldn't be in input data, so should ignore it
            amount: Some(Amount(20.)),
        },
    ];
    let mut executor = Executor::default();
    for i in inputs {
        executor.process(i);
    }
    let out: Vec<_> = executor.output().collect();
    assert_eq!(out.len(), 1);
    let item = out.get(0).unwrap();
    let expected = Output {
        client: Client(1),
        available: Amount(15.),
        held: Amount(0.),
        total: Amount(15.),
        locked: false,
    };
    assert_eq!(item, &expected);
}
