#![allow(dead_code)]

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::ops::Add;
use std::ops::AddAssign;
use thiserror::Error;

const MIN_DEPOSIT: Decimal = dec!(0.0001);
const MAX_DEPOSIT: Decimal = dec!(50000);
const MIN_WITHDRAW: Decimal = dec!(0.0001);
const MAX_WITHDRAW: Decimal = dec!(50000);

#[derive(Debug)]
struct PaymentsEngine {
    client_list: HashMap<ClientId, Client>,
}

impl PaymentsEngine {
    fn recv_tx(&mut self, transaction: Transaction) -> Result<(), DepositError> {
        match transaction {
            Transaction::Deposit(deposit) => {
                let amount = Amount::new_deposit(deposit.amount)?;
                self.client_list
                    .entry(deposit.client_id)
                    .and_modify(|client| {
                        client.available += amount;
                        client
                            .transaction_list
                            .insert(deposit.transaction_id, Transaction::Deposit(deposit));
                    })
                    .or_insert(Client::new_with_deposit(deposit));
                Ok(())
            } /*
                          Transaction::Withdraw(withdraw) => {
                              let amout = Amount::new_withdraw(withdraw.amount)?;
                              self.client_list
                                  .entry(withdraw.client_id)
                                  .and_modify(|client| {
                                      if amount > client.available {
                                          Err(WithdrawError::LessThanMin)
                                      } else {
                                          client.available -= amount;
                                          client
                                              .transaction_list
                                              .insert(withdraw.transaction_id, Transaction::Withdraw(withdraw));
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
              */
        }
    }
}

#[derive(Error, Debug)]
enum DepositError {
    #[error("amount is less than minimum")]
    LessThanMin,
    #[error("amount is more than maximum")]
    MoreThanMax,
}

#[derive(Error, Debug)]
enum WithdrawError {
    #[error("amount is less than minimum")]
    LessThanMin,
    #[error("amount is more than maximum")]
    MoreThanMax,
}

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
struct ClientId(u16);

#[derive(Debug, PartialEq)]
struct Client {
    client_id: ClientId,
    available: Amount,
    held: Amount,
    locked: bool,
    transaction_list: HashMap<TransactionId, Transaction>,
}

impl Client {
    fn new_with_deposit(deposit: Deposit) -> Client {
        let mut transaction_list = HashMap::new();
        transaction_list.insert(deposit.transaction_id, Transaction::Deposit(deposit));
        let amount = Amount::new_deposit(deposit.amount).unwrap();
        Client {
            client_id: deposit.client_id,
            available: amount,
            held: Amount(Decimal::ZERO),
            locked: false,
            transaction_list,
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
struct Amount(Decimal);

impl Add for Amount {
    type Output = Self;

    fn add(self, other: Amount) -> Self::Output {
        self + other
    }
}

impl AddAssign for Amount {
    fn add_assign(&mut self, other: Amount) {
        *self = *self + other
    }
}

impl Amount {
    fn new_deposit(amount: Decimal) -> Result<Amount, DepositError> {
        if amount < MIN_DEPOSIT {
            Err(DepositError::LessThanMin)
        } else if amount > MAX_DEPOSIT {
            Err(DepositError::MoreThanMax)
        } else {
            Ok(Amount(amount))
        }
    }

    fn new_withdraw(amount: Decimal) -> Result<Amount, WithdrawError> {
        if amount < MIN_WITHDRAW {
            Err(WithdrawError::LessThanMin)
        } else if amount > MAX_WITHDRAW {
            Err(WithdrawError::MoreThanMax)
        } else {
            Ok(Amount(amount))
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
struct TransactionId(u32);

#[derive(Debug, PartialEq, Copy, Clone)]
enum Transaction {
    Deposit(Deposit),
    //  Withdraw(Withdraw),
    //Dispute,
    //Resolve,
    //Chargeback,
}

#[derive(Debug, PartialEq, Copy, Clone)]
struct Deposit {
    transaction_id: TransactionId,
    client_id: ClientId,
    amount: Decimal,
    disputed: bool,
}

#[derive(Debug, PartialEq, Copy, Clone)]
struct Withdraw {
    transaction_id: TransactionId,
    client_id: ClientId,
    amount: Decimal,
    disputed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn add_trait_implementation_for_amount() {
        let x = Amount(dec!(1));
        let y = Amount(dec!(2));
        let z = x + y;
        assert_eq!(Amount(dec!(3)), z);
    }

    #[test_case(Decimal::ONE_HUNDRED; "amount is one hundred")]
    #[test_case(MIN_DEPOSIT; "minimum amount")]
    #[test_case(MAX_DEPOSIT; "maximum deposit")]
    #[test_case(Decimal::NEGATIVE_ONE; "amount is less than zero")]
    fn deposit_to_non_existing_client_id(amount: Decimal) {
        let mut payments_engine = PaymentsEngine {
            client_list: HashMap::new(),
        };

        let deposit = Deposit {
            transaction_id: TransactionId(1),
            client_id: ClientId(1),
            amount: amount,
            disputed: false,
        };

        payments_engine.recv_tx(Transaction::Deposit(deposit));

        let client = payments_engine
            .client_list
            .get(&ClientId(1))
            .expect("client id doesn't exist...");

        let mut fake_client = Client {
            client_id: ClientId(1),
            available: Amount(amount),
            held: Amount(Decimal::ZERO),
            locked: false,
            transaction_list: HashMap::new(),
        };

        fake_client
            .transaction_list
            .insert(deposit.transaction_id, Transaction::Deposit(deposit));

        assert_eq!(client, &fake_client);
    }

    #[test_case(Decimal::ONE_HUNDRED, Decimal::ONE_HUNDRED; "both amounts are 100")]
    //#[test_case(MIN_DEPOSIT; "minimum amount")]
    //#[test_case(MAX_DEPOSIT; "maximum deposit")]
    //#[test_case(Decimal::NEGATIVE_ONE; "amount is less than zero")]
    fn deposit_to_existing_client_id(first_amount: Decimal, second_amount: Decimal) {
        let mut payments_engine = PaymentsEngine {
            client_list: HashMap::new(),
        };

        let first_deposit = Deposit {
            transaction_id: TransactionId(1),
            client_id: ClientId(1),
            amount: first_amount,
            disputed: false,
        };

        payments_engine.recv_tx(Transaction::Deposit(first_deposit));

        let second_deposit = Deposit {
            transaction_id: TransactionId(2),
            client_id: ClientId(1),
            amount: second_amount,
            disputed: false,
        };

        payments_engine.recv_tx(Transaction::Deposit(second_deposit));

        let client_after_second_deposit = payments_engine
            .client_list
            .get(&ClientId(1))
            .expect("client id doesn't exist");

        let mut fake_transaction_list = HashMap::new();
        fake_transaction_list.insert(
            first_deposit.transaction_id,
            Transaction::Deposit(first_deposit),
        );
        fake_transaction_list.insert(
            second_deposit.transaction_id,
            Transaction::Deposit(second_deposit),
        );

        let fake_client_after_second_deposit = Client {
            client_id: ClientId(1),
            available: Amount(first_amount + second_amount),
            held: Amount(Decimal::ZERO),
            locked: false,
            transaction_list: fake_transaction_list,
        };

        assert_eq!(
            client_after_second_deposit,
            &fake_client_after_second_deposit
        );
    }
    /*
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
    */
}
