#![allow(dead_code)]

use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug)]
struct PaymentsEngine {
    client_list: HashMap<ClientId, Client>,
}

impl PaymentsEngine {
    fn recv_tx(&mut self, transaction: Transaction) {
        match transaction.transaction_type {
            TransactionType::Deposit => {
                self.client_list
                    .entry(transaction.client_id)
                    .and_modify(|client| {
                        let amount = transaction.amount.unwrap();
                        client.available += amount;
                        client
                            .transaction_list
                            .insert(transaction.transaction_id, transaction);
                    })
                    .or_insert(Client::new_with_deposit(transaction));
            }

            TransactionType::Withdraw => {
                self.client_list
                    .entry(transaction.client_id)
                    .and_modify(|client| {
                        let amount = transaction.amount.unwrap();
                        if amount > client.available {
                            client
                                .transaction_list
                                .insert(transaction.transaction_id, transaction);
                            eprintln!("Withdraw amount is bigger than available amount...");
                        } else {
                            client.available -= amount;
                            client
                                .transaction_list
                                .insert(transaction.transaction_id, transaction);
                        }
                    });
            }

            TransactionType::Dispute(tx) => {
                self.client_list
                    .entry(transaction.client_id)
                    .and_modify(|client| {
                        let target_transaction = client
                            .transaction_list
                            .get(&tx)
                            .expect("transaction id doesn't exist");
                        if target_transaction.transaction_type != TransactionType::Deposit {
                            eprintln!("cannot dispute a non deposit transaction...");
                        } else {
                            let amount = target_transaction.amount.unwrap();
                            client.available -= amount;
                            client.held += amount;
                            client.disputed = true;
                            client
                                .transaction_list
                                .insert(transaction.transaction_id, transaction);
                        }
                    });
            }

            TransactionType::Resolve(tx) => {
                self.client_list
                    .entry(transaction.client_id)
                    .and_modify(|client| {
                        if !client.disputed {
                            eprintln!("cannot resolve a transaction that is not disputed...");
                        } else {
                            let target_transaction = client
                                .transaction_list
                                .get(&tx)
                                .expect("transaction id doesn't exist");
                            let amount = target_transaction.amount.unwrap();
                            client.available += amount;
                            client.held -= amount;
                            client.disputed = false;
                            client
                                .transaction_list
                                .insert(transaction.transaction_id, transaction);
                        }
                    });
            }

            TransactionType::Chargeback(tx) => {
                self.client_list
                    .entry(transaction.client_id)
                    .and_modify(|client| {
                        if !client.disputed {
                            eprintln!("cannot chargeback a transaction that is not disputed...");
                        } else {
                            let target_transaction = client
                                .transaction_list
                                .get(&tx)
                                .expect("transaction id doesn't exist");
                            let amount = target_transaction.amount.unwrap();
                            client.held -= amount;
                            client.disputed = false;
                            client.locked = true;
                            client
                                .transaction_list
                                .insert(transaction.transaction_id, transaction);
                        }
                    });
            }
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
struct ClientId(u16);

#[derive(Debug, PartialEq)]
struct Client {
    client_id: ClientId,
    available: Decimal,
    held: Decimal,
    locked: bool,
    transaction_list: HashMap<TransactionId, Transaction>,
    disputed: bool,
}

impl Client {
    fn new_with_deposit(deposit: Transaction) -> Client {
        let mut transaction_list = HashMap::new();
        transaction_list.insert(deposit.transaction_id, deposit);
        let amount = deposit.amount.unwrap();
        Client {
            client_id: deposit.client_id,
            available: amount,
            held: Decimal::ZERO,
            locked: false,
            transaction_list,
            disputed: false,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
struct Transaction {
    transaction_id: TransactionId,
    client_id: ClientId,
    transaction_type: TransactionType,
    amount: Option<Decimal>,
}

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
struct TransactionId(u32);

#[derive(Debug, PartialEq, Copy, Clone)]
enum TransactionType {
    Deposit,
    Withdraw,
    Dispute(TransactionId),
    Resolve(TransactionId),
    Chargeback(TransactionId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deposit_to_non_existing_client_id() {
        let mut payments_engine = PaymentsEngine {
            client_list: HashMap::new(),
        };

        let deposit = Transaction {
            transaction_id: TransactionId(1),
            client_id: ClientId(1),
            transaction_type: TransactionType::Deposit,
            amount: Some(Decimal::ONE_HUNDRED),
        };

        payments_engine.recv_tx(deposit);

        let client = payments_engine
            .client_list
            .get(&ClientId(1))
            .expect("client id doesn't exist...");

        let mut client_mock = Client {
            client_id: ClientId(1),
            available: Decimal::ONE_HUNDRED,
            held: Decimal::ZERO,
            locked: false,
            transaction_list: HashMap::new(),
            disputed: false,
        };

        client_mock
            .transaction_list
            .insert(deposit.transaction_id, deposit);

        assert_eq!(client, &client_mock);
    }

    #[test]
    fn deposit_to_existing_client_id() {
        let mut payments_engine = PaymentsEngine {
            client_list: HashMap::new(),
        };

        let first_deposit = Transaction {
            transaction_id: TransactionId(1),
            client_id: ClientId(1),
            transaction_type: TransactionType::Deposit,
            amount: Some(Decimal::ONE),
        };

        payments_engine.recv_tx(first_deposit);

        let second_deposit = Transaction {
            transaction_id: TransactionId(2),
            client_id: ClientId(1),
            transaction_type: TransactionType::Deposit,
            amount: Some(Decimal::ONE),
        };

        payments_engine.recv_tx(second_deposit);

        let client_after_second_deposit = payments_engine
            .client_list
            .get(&ClientId(1))
            .expect("client id doesn't exist");

        let mut transaction_list_mock = HashMap::new();
        transaction_list_mock.insert(first_deposit.transaction_id, first_deposit);
        transaction_list_mock.insert(second_deposit.transaction_id, second_deposit);

        let client_after_second_deposit_mock = Client {
            client_id: ClientId(1),
            available: Decimal::TWO,
            held: Decimal::ZERO,
            locked: false,
            transaction_list: transaction_list_mock,
            disputed: false,
        };

        assert_eq!(
            client_after_second_deposit,
            &client_after_second_deposit_mock
        );
    }

    #[test]
    fn withdraw_from_client_id() {
        let mut payments_engine = PaymentsEngine {
            client_list: HashMap::new(),
        };

        let deposit = Transaction {
            transaction_id: TransactionId(1),
            client_id: ClientId(1),
            transaction_type: TransactionType::Deposit,
            amount: Some(Decimal::ONE_HUNDRED),
        };

        payments_engine.recv_tx(deposit);

        let withdraw = Transaction {
            transaction_id: TransactionId(2),
            client_id: ClientId(1),
            transaction_type: TransactionType::Withdraw,
            amount: Some(Decimal::ONE_HUNDRED),
        };

        payments_engine.recv_tx(withdraw);

        let client_after_withdraw = payments_engine
            .client_list
            .get(&ClientId(1))
            .expect("client id doesn't exist...");

        let mut transaction_list_mock = HashMap::new();
        transaction_list_mock.insert(deposit.transaction_id, deposit);
        transaction_list_mock.insert(withdraw.transaction_id, withdraw);

        let client_after_withdraw_mock = Client {
            client_id: ClientId(1),
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            locked: false,
            transaction_list: transaction_list_mock,
            disputed: false,
        };

        assert_eq!(client_after_withdraw, &client_after_withdraw_mock);
    }

    #[test]
    fn withdraw_insufficient_amount_from_client_id() {
        let mut payments_engine = PaymentsEngine {
            client_list: HashMap::new(),
        };

        let deposit = Transaction {
            transaction_id: TransactionId(1),
            client_id: ClientId(1),
            transaction_type: TransactionType::Deposit,
            amount: Some(Decimal::ONE),
        };

        payments_engine.recv_tx(deposit);

        let withdraw = Transaction {
            transaction_id: TransactionId(2),
            client_id: ClientId(1),
            transaction_type: TransactionType::Withdraw,
            amount: Some(Decimal::ONE_HUNDRED),
        };

        payments_engine.recv_tx(withdraw);

        let client_after_failed_withdraw = payments_engine
            .client_list
            .get(&ClientId(1))
            .expect("client id doesn't exist...");

        let mut transaction_list_mock = HashMap::new();
        transaction_list_mock.insert(deposit.transaction_id, deposit);
        transaction_list_mock.insert(withdraw.transaction_id, withdraw);

        let client_after_failed_withdraw_mock = Client {
            client_id: ClientId(1),
            available: Decimal::ONE,
            held: Decimal::ZERO,
            locked: false,
            transaction_list: transaction_list_mock,
            disputed: false,
        };

        assert_eq!(
            client_after_failed_withdraw,
            &client_after_failed_withdraw_mock
        );
    }

    #[test]
    fn dispute_a_deposit() {
        let mut payments_engine = PaymentsEngine {
            client_list: HashMap::new(),
        };

        let deposit = Transaction {
            transaction_id: TransactionId(1),
            client_id: ClientId(1),
            transaction_type: TransactionType::Deposit,
            amount: Some(Decimal::ONE_HUNDRED),
        };

        payments_engine.recv_tx(deposit);

        let dispute = Transaction {
            transaction_id: TransactionId(2),
            client_id: ClientId(1),
            transaction_type: TransactionType::Dispute(deposit.transaction_id),
            amount: None,
        };

        payments_engine.recv_tx(dispute);

        let client = payments_engine
            .client_list
            .get(&ClientId(1))
            .expect("client id doesn't exist...");

        let mut client_mock = Client {
            client_id: ClientId(1),
            available: Decimal::ZERO,
            held: Decimal::ONE_HUNDRED,
            locked: false,
            transaction_list: HashMap::new(),
            disputed: true,
        };

        client_mock
            .transaction_list
            .insert(deposit.transaction_id, deposit);

        client_mock
            .transaction_list
            .insert(dispute.transaction_id, dispute);

        assert_eq!(client, &client_mock);
    }

    #[test]
    fn resolve_a_dispute() {
        let mut payments_engine = PaymentsEngine {
            client_list: HashMap::new(),
        };

        let deposit = Transaction {
            transaction_id: TransactionId(1),
            client_id: ClientId(1),
            transaction_type: TransactionType::Deposit,
            amount: Some(Decimal::ONE_HUNDRED),
        };

        payments_engine.recv_tx(deposit);

        let dispute = Transaction {
            transaction_id: TransactionId(2),
            client_id: ClientId(1),
            transaction_type: TransactionType::Dispute(TransactionId(1)),
            amount: None,
        };

        payments_engine.recv_tx(dispute);

        let resolve = Transaction {
            transaction_id: TransactionId(3),
            client_id: ClientId(1),
            transaction_type: TransactionType::Resolve(TransactionId(1)),
            amount: None,
        };

        payments_engine.recv_tx(resolve);

        let client = payments_engine
            .client_list
            .get(&ClientId(1))
            .expect("client id doesn't exist...");

        let mut client_mock = Client {
            client_id: ClientId(1),
            available: Decimal::ONE_HUNDRED,
            held: Decimal::ZERO,
            locked: false,
            transaction_list: HashMap::new(),
            disputed: false,
        };

        client_mock
            .transaction_list
            .insert(deposit.transaction_id, deposit);

        client_mock
            .transaction_list
            .insert(dispute.transaction_id, dispute);

        client_mock
            .transaction_list
            .insert(resolve.transaction_id, resolve);

        assert_eq!(client, &client_mock);
    }

    #[test]
    fn chargeback_a_dispute() {
        let mut payments_engine = PaymentsEngine {
            client_list: HashMap::new(),
        };

        let deposit = Transaction {
            transaction_id: TransactionId(1),
            client_id: ClientId(1),
            transaction_type: TransactionType::Deposit,
            amount: Some(Decimal::ONE_HUNDRED),
        };

        payments_engine.recv_tx(deposit);

        let dispute = Transaction {
            transaction_id: TransactionId(2),
            client_id: ClientId(1),
            transaction_type: TransactionType::Dispute(TransactionId(1)),
            amount: None,
        };

        payments_engine.recv_tx(dispute);

        let chargeback = Transaction {
            transaction_id: TransactionId(3),
            client_id: ClientId(1),
            transaction_type: TransactionType::Chargeback(TransactionId(1)),
            amount: None,
        };

        payments_engine.recv_tx(chargeback);

        let client = payments_engine
            .client_list
            .get(&ClientId(1))
            .expect("client id doesn't exist...");

        let mut client_mock = Client {
            client_id: ClientId(1),
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            locked: true,
            transaction_list: HashMap::new(),
            disputed: false,
        };

        client_mock
            .transaction_list
            .insert(deposit.transaction_id, deposit);

        client_mock
            .transaction_list
            .insert(dispute.transaction_id, dispute);

        client_mock
            .transaction_list
            .insert(chargeback.transaction_id, chargeback);

        assert_eq!(client, &client_mock);
    }
}
