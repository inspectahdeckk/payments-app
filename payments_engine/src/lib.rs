#![allow(dead_code)]

use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;
use rust_decimal_macros::dec;
use std::collections::HashMap;
//use std::ops::Add;
//use std::ops::AddAssign;
use thiserror::Error;

const MIN_DEPOSIT: Decimal = dec!(0.0001);
const MAX_DEPOSIT: Decimal = dec!(50000);
const MIN_WITHDRAW: Decimal = dec!(0.0001);
const MAX_WITHDRAW: Decimal = dec!(50000);
const DECIMAL_POINTS: u32 = 4;
const ROUNDING_STRATEGY: RoundingStrategy = RoundingStrategy::MidpointNearestEven;

#[derive(Debug)]
struct PaymentsEngine {
    client_list: HashMap<ClientId, Client>,
}

impl PaymentsEngine {
    fn recv_tx(&mut self, transaction: Transaction) -> Result<(), Error> {
        match transaction {
            Transaction::Deposit(deposit) => {
                let amount = Amount::check_and_round_deposit(deposit.amount)?;
                self.client_list
                    .entry(deposit.client_id)
                    .and_modify(|client| {
                        client.available = client.available.checked_add(amount);
                        client
                            .transaction_list
                            .insert(deposit.transaction_id, Transaction::Deposit(deposit));
                    })
                    .or_insert(Client::new_with_deposit(deposit));
                Ok(())
            }

            Transaction::Withdraw(withdraw) => {
                let amount = Amount::check_and_round_withdraw(withdraw.amount)?;
                let client = self
                    .client_list
                    .get_mut(&withdraw.client_id)
                    .expect("client id doesn't exist");
                if client.available < amount {
                    return Err(Error::WithdrawMoreThanAvailable);
                }
                client.available = client.available.checked_subtract(amount);
                client
                    .transaction_list
                    .insert(withdraw.transaction_id, Transaction::Withdraw(withdraw));
                Ok(())
            }

            Transaction::Dispute(dispute) => {
                let client = self
                    .client_list
                    .get_mut(&dispute.client_id)
                    .expect("client id doesn't exist");
                let target_transaction = client
                    .transaction_list
                    .get_mut(&dispute.target_transaction_id);
                match target_transaction {
                    Some(Transaction::Deposit(target)) => {
                        if target.dispute_status == DisputeStatus::NotDisputed {
                            let amount = target.amount;
                            client.available = client.available.checked_subtract(amount);
                            client.held = client.held.checked_add(amount);
                            target.dispute_status = DisputeStatus::Disputed;
                            Ok(())
                        } else {
                            Err(Error::DepositTwiceDisputed)
                        }
                    }
                    Some(Transaction::Withdraw(_target)) => Err(Error::WithdrawDispute),
                    Some(Transaction::Dispute(_target)) => Err(Error::WithdrawDispute), // TODO
                    Some(Transaction::Resolve(_target)) => Err(Error::WithdrawDispute), // TODO
                    None => Err(Error::NonExistingTransaction),
                }
            }
	
            Transaction::Resolve(resolve) => {
				let client = self
					.client_list
					.get_mut(&resolve.client_id)
					.expect("client id doesn't exist");
				let target_transaction = client
					.transaction_list
					.get_mut(&resolve.target_transaction_id);
				match target_transaction {
                    Some(Transaction::Deposit(target)) => {
                        if target.dispute_status == DisputeStatus::Disputed {
                            let amount = target.amount;
                            client.available = client.available.checked_add(amount);
                            client.held = client.held.checked_subtract(amount);
                            target.dispute_status = DisputeStatus::Resolved;
                            Ok(())
                        } else {
                            Err(Error::DepositTwiceDisputed)
                        }
                    }
                    Some(Transaction::Withdraw(_target)) => Err(Error::WithdrawDispute),
                    Some(Transaction::Dispute(_target)) => Err(Error::WithdrawDispute), // TODO
                    Some(Transaction::Resolve(_target)) => Err(Error::WithdrawDispute), // TODO
                    None => Err(Error::NonExistingTransaction),
                }
            } /*
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
enum Error {
    #[error("transaction id doesn't exist")]
    NonExistingTransaction,

    #[error("deposit amount is lower than minimum")]
    DepositLessThanMin,

    #[error("deposit amount is bigger than maximum")]
    DepositMoreThanMax,

    #[error("withdraw amount is lower than minimum")]
    WithdrawLessThanMin,

    #[error("withdraw amount is bigger than maximum")]
    WithdrawMoreThanMax,

    #[error("withdraw amount is bigger than available amount")]
    WithdrawMoreThanAvailable,

    #[error("deposit is under dipuste")]
    DepositTwiceDisputed,

    #[error("cannot dispute a withdraw")]
    WithdrawDispute,
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
        let amount = deposit
            .amount
            .0
            .round_dp_with_strategy(DECIMAL_POINTS, ROUNDING_STRATEGY);
        Client {
            client_id: deposit.client_id,
            available: Amount(amount),
            held: Amount(Decimal::ZERO),
            locked: false,
            transaction_list,
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq, PartialOrd, Copy, Clone)]
struct Amount(Decimal);

impl Amount {
    fn checked_add(self, rhs: Amount) -> Amount {
        let checked_add_decimal = self.0.checked_add(rhs.0).expect("overflow");
        Amount(checked_add_decimal)
    }

    fn checked_subtract(self, rhs: Amount) -> Amount {
        let checked_subtract_decimal = self.0.checked_sub(rhs.0).expect("overflow");
        Amount(checked_subtract_decimal)
    }

    fn check_and_round_deposit(amount: Amount) -> Result<Amount, Error> {
        if amount.0 < MIN_DEPOSIT {
            Err(Error::DepositLessThanMin)
        } else if amount.0 > MAX_DEPOSIT {
            Err(Error::DepositMoreThanMax)
        } else {
            Ok(Amount(
                amount
                    .0
                    .round_dp_with_strategy(DECIMAL_POINTS, ROUNDING_STRATEGY),
            ))
        }
    }

    fn check_and_round_withdraw(amount: Amount) -> Result<Amount, Error> {
        if amount.0 < MIN_WITHDRAW {
            Err(Error::WithdrawLessThanMin)
        } else if amount.0 > MAX_WITHDRAW {
            Err(Error::WithdrawMoreThanMax)
        } else {
            Ok(Amount(
                amount
                    .0
                    .round_dp_with_strategy(DECIMAL_POINTS, ROUNDING_STRATEGY),
            ))
        }
    }
}

/* TODO
impl Add for Amount {
    type Output = Amount;

    fn add(self, other: Amount) -> Self::Output {
        self.0 + other.0
    }
}

impl AddAssign for Amount {
    fn add_assign(&mut self, other: Amount) {
        *self.0 = *self.0 + other.0
    }
}
*/

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
struct TransactionId(u32);

#[derive(Debug, PartialEq, Copy, Clone)]
enum Transaction {
    Deposit(Deposit),
    Withdraw(Withdraw),
    Dispute(Dispute),
    Resolve(Resolve),
    //Chargeback,
}

#[derive(Debug, PartialEq, Copy, Clone)]
struct Deposit {
    transaction_id: TransactionId,
    client_id: ClientId,
    amount: Amount,
    dispute_status: DisputeStatus,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum DisputeStatus {
    NotDisputed,
    Disputed,
    Resolved,
    Chargebacked,
}

#[derive(Debug, PartialEq, Copy, Clone)]
struct Withdraw {
    transaction_id: TransactionId,
    client_id: ClientId,
    amount: Amount,
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Dispute {
    client_id: ClientId,
    target_transaction_id: TransactionId,
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Resolve {
    client_id: ClientId,
    target_transaction_id: TransactionId,
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(Amount(Decimal::ONE_HUNDRED); "deposit amount is one hundred")]
    #[test_case(Amount(MIN_DEPOSIT); "minimum deposit amount")]
    #[test_case(Amount(MAX_DEPOSIT); "maximum deposit amount")]
    //#[test_case(Amount(Decimal::NEGATIVE_ONE); "amount is less than deposit minimum")] TODO
    //#[test_case(Amount(Decimal::MAX); "amount is more than deposit maximum")] TODO
    fn deposit_to_non_existing_client_id(amount: Amount) {
        let mut payments_engine = PaymentsEngine {
            client_list: HashMap::new(),
        };

        let deposit = Deposit {
            transaction_id: TransactionId(1),
            client_id: ClientId(1),
            amount: amount,
            dispute_status: DisputeStatus::NotDisputed,
        };

        payments_engine
            .recv_tx(Transaction::Deposit(deposit))
            .expect("deposit amount error");

        let client = payments_engine
            .client_list
            .get(&ClientId(1))
            .expect("client id doesn't exist...");

        let mut fake_client = Client {
            client_id: ClientId(1),
            available: amount,
            held: Amount(Decimal::ZERO),
            locked: false,
            transaction_list: HashMap::new(),
        };

        fake_client
            .transaction_list
            .insert(deposit.transaction_id, Transaction::Deposit(deposit));

        assert_eq!(client, &fake_client);
    }

    // TODO more test cases
    #[test_case(Amount(Decimal::ONE_HUNDRED), Amount(Decimal::ONE_HUNDRED); "both amounts are 100")]
    fn deposit_to_existing_client_id(first_amount: Amount, second_amount: Amount) {
        let mut payments_engine = PaymentsEngine {
            client_list: HashMap::new(),
        };

        let first_deposit = Deposit {
            transaction_id: TransactionId(1),
            client_id: ClientId(1),
            amount: first_amount,
            dispute_status: DisputeStatus::NotDisputed,
        };

        payments_engine
            .recv_tx(Transaction::Deposit(first_deposit))
            .expect("deposit amount error");

        let second_deposit = Deposit {
            transaction_id: TransactionId(2),
            client_id: ClientId(1),
            amount: second_amount,
            dispute_status: DisputeStatus::NotDisputed,
        };

        payments_engine
            .recv_tx(Transaction::Deposit(second_deposit))
            .expect("deposit amount error");

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
            available: first_amount.checked_add(second_amount),
            held: Amount(Decimal::ZERO),
            locked: false,
            transaction_list: fake_transaction_list,
        };

        assert_eq!(
            client_after_second_deposit,
            &fake_client_after_second_deposit
        );
    }
    //TODO more test cases
    #[test_case(Amount(Decimal::ONE_HUNDRED), Amount(Decimal::ONE_HUNDRED); "normal withdraw")]
    fn withdraw_from_client_id(deposit_amount: Amount, withdraw_amount: Amount) {
        let mut payments_engine = PaymentsEngine {
            client_list: HashMap::new(),
        };

        let deposit = Deposit {
            transaction_id: TransactionId(1),
            client_id: ClientId(1),
            amount: deposit_amount,
            dispute_status: DisputeStatus::NotDisputed,
        };

        payments_engine
            .recv_tx(Transaction::Deposit(deposit))
            .expect("deposit amount error");

        let withdraw = Withdraw {
            transaction_id: TransactionId(2),
            client_id: ClientId(1),
            amount: withdraw_amount,
        };

        payments_engine
            .recv_tx(Transaction::Withdraw(withdraw))
            .expect("withdraw amount error");

        let client_after_withdraw = payments_engine
            .client_list
            .get(&ClientId(1))
            .expect("client id doesn't exist...");

        let mut fake_transaction_list = HashMap::new();
        fake_transaction_list.insert(deposit.transaction_id, Transaction::Deposit(deposit));
        fake_transaction_list.insert(withdraw.transaction_id, Transaction::Withdraw(withdraw));

        let fake_client_after_withdraw = Client {
            client_id: ClientId(1),
            available: deposit_amount.checked_subtract(withdraw_amount),
            held: Amount(Decimal::ZERO),
            locked: false,
            transaction_list: fake_transaction_list,
        };

        assert_eq!(client_after_withdraw, &fake_client_after_withdraw);
    }

    #[test]
    #[should_panic]
    fn withdraw_insufficient_amount_from_client_id() {
        let mut payments_engine = PaymentsEngine {
            client_list: HashMap::new(),
        };

        let deposit = Deposit {
            transaction_id: TransactionId(1),
            client_id: ClientId(1),
            amount: Amount(Decimal::ONE),
            dispute_status: DisputeStatus::NotDisputed,
        };

        payments_engine
            .recv_tx(Transaction::Deposit(deposit))
            .expect("deposit amount error");

        let withdraw = Withdraw {
            transaction_id: TransactionId(2),
            client_id: ClientId(1),
            amount: Amount(Decimal::ONE_HUNDRED),
        };

        payments_engine
            .recv_tx(Transaction::Withdraw(withdraw))
            .expect("withdraw amount error");
    }

    #[test]
    fn dispute_a_deposit() {
        let mut payments_engine = PaymentsEngine {
            client_list: HashMap::new(),
        };

        let deposit = Deposit {
            transaction_id: TransactionId(1),
            client_id: ClientId(1),
            amount: Amount(Decimal::ONE_HUNDRED),
            dispute_status: DisputeStatus::NotDisputed,
        };

        let dispute = Dispute {
            client_id: ClientId(1),
            target_transaction_id: TransactionId(1),
        };

        payments_engine
            .recv_tx(Transaction::Deposit(deposit))
            .expect("deposit amount error");
        payments_engine
            .recv_tx(Transaction::Dispute(dispute))
            .expect("dispute error");

        let client = payments_engine
            .client_list
            .get(&ClientId(1))
            .expect("client id doesn't exist...");

        let mut fake_client = Client {
            client_id: ClientId(1),
            available: Amount(Decimal::ZERO),
            held: Amount(Decimal::ONE_HUNDRED),
            transaction_list: HashMap::new(),
            locked: false,
        };

        let fake_deposit = Deposit {
            transaction_id: TransactionId(1),
            client_id: ClientId(1),
            amount: Amount(Decimal::ONE_HUNDRED),
            dispute_status: DisputeStatus::Disputed,
        };

        fake_client.transaction_list.insert(
            fake_deposit.transaction_id,
            Transaction::Deposit(fake_deposit),
        );

        assert_eq!(client, &fake_client);
    }

    #[test]
    #[should_panic]
    fn dispute_a_non_deposit() {
        let mut payments_engine = PaymentsEngine {
            client_list: HashMap::new(),
        };

        let deposit = Deposit {
            transaction_id: TransactionId(1),
            client_id: ClientId(1),
            amount: Amount(Decimal::ONE_HUNDRED),
            dispute_status: DisputeStatus::NotDisputed,
        };

        let withdraw = Withdraw {
            transaction_id: TransactionId(2),
            client_id: ClientId(1),
            amount: Amount(Decimal::ONE_HUNDRED),
        };

        let dispute = Dispute {
            client_id: ClientId(1),
            target_transaction_id: TransactionId(2),
        };

        payments_engine
            .recv_tx(Transaction::Deposit(deposit))
            .expect("deposit amount error");

        payments_engine
            .recv_tx(Transaction::Withdraw(withdraw))
            .expect("withdraw amount error");

        payments_engine
            .recv_tx(Transaction::Dispute(dispute))
            .expect("dispute error");
    }
    
     #[test]
     fn resolve_a_dispute() {
        let mut payments_engine = PaymentsEngine {
            client_list: HashMap::new(),
        };

        let deposit = Deposit {
            transaction_id: TransactionId(1),
            client_id: ClientId(1),
            amount: Amount(Decimal::ONE_HUNDRED),
            dispute_status: DisputeStatus::NotDisputed,
        };

        let dispute = Dispute {
            client_id: ClientId(1),
            target_transaction_id: TransactionId(1),
        };

		let resolve = Resolve {
			client_id: ClientId(1),
			target_transaction_id: TransactionId(1),	
		};

        payments_engine
            .recv_tx(Transaction::Deposit(deposit))
            .expect("deposit amount error");
        
		payments_engine
            .recv_tx(Transaction::Dispute(dispute))
            .expect("dispute error");
		
		payments_engine
            .recv_tx(Transaction::Resolve(resolve))
            .expect("resolve error");

        let client = payments_engine
            .client_list
            .get(&ClientId(1))
            .expect("client id doesn't exist...");

        let mut fake_client = Client {
            client_id: ClientId(1),
            available: Amount(Decimal::ONE_HUNDRED),
            held: Amount(Decimal::ZERO),
            transaction_list: HashMap::new(),
            locked: false,
        };

        let fake_deposit = Deposit {
            transaction_id: TransactionId(1),
            client_id: ClientId(1),
            amount: Amount(Decimal::ONE_HUNDRED),
            dispute_status: DisputeStatus::Resolved,
        };

        fake_client.transaction_list.insert(
            fake_deposit.transaction_id,
            Transaction::Deposit(fake_deposit),
        );

        assert_eq!(client, &fake_client);
    }    
/*
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
