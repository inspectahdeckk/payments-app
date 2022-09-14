#![allow(dead_code)]

use std::collections::HashMap;
use rust_decimal::Decimal;


#[derive(Debug)]
struct PaymentsEngine {
    client_list: HashMap<ClientId, Client>,
}

impl PaymentsEngine {

    fn recv_tx(&mut self, transaction: Transaction) {
        
		match transaction.type_of_transaction {
            TransactionType::Deposit=> {
				self.client_list
					.entry(transaction.client_id)
					.and_modify(|client| {
						client.available += transaction.amount;
						client.transaction_list.insert(transaction.transaction_id, transaction);
					}) 
					.or_insert(Client::new_with_deposit(transaction));
            },
			
			TransactionType::Withdraw=> {
				self.client_list
					.entry(transaction.client_id)
					.and_modify(|client| {
						if transaction.amount > client.available {
							client.transaction_list.insert(transaction.transaction_id, transaction);
							eprintln!("Withdraw amount is bigger than available amount...");
						} else {
							client.available -= transaction.amount;
							client.transaction_list.insert(transaction.transaction_id, transaction);
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
}

impl Client {
	fn new_with_deposit(deposit: Transaction) -> Client {
		let mut transaction_list = HashMap::new();
		transaction_list.insert(deposit.transaction_id, deposit);
		Client {
			client_id: deposit.client_id,
			available: deposit.amount,
			held: Decimal::ZERO,
			locked: false,
			transaction_list, 
		}
	}
}

#[derive(Debug, PartialEq, Copy, Clone)]
struct Transaction {
	transaction_id: TransactionId,
	client_id: ClientId,
	type_of_transaction: TransactionType, 	
	amount: Decimal,
}

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
struct TransactionId(u32);

#[derive(Debug, PartialEq, Copy, Clone)]
enum TransactionType {
	Deposit,
	Withdraw,
//	Dispute,
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
			type_of_transaction: TransactionType::Deposit,
			amount: Decimal::ONE_HUNDRED,
		};
        
		payments_engine.recv_tx(deposit);
		
		let client = payments_engine.client_list.get(&ClientId(1)).expect("client id doesn't exist...");  
		
		let mut client_mock = Client {
			client_id: ClientId(1),
  			available: Decimal::ONE_HUNDRED,
  		    held: Decimal::ZERO,
  		    locked: false,
			transaction_list: HashMap::new(),		
		};
		
		client_mock.transaction_list.insert(deposit.transaction_id, deposit);
		
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
			type_of_transaction: TransactionType::Deposit,
			amount: Decimal::ONE,
		};

		payments_engine.recv_tx(first_deposit);
		
		let second_deposit = Transaction {
			transaction_id: TransactionId(2),
			client_id: ClientId(1),
			type_of_transaction: TransactionType::Deposit,
			amount: Decimal::ONE,
		};
			
		payments_engine.recv_tx(second_deposit);
		
		let client_after_second_deposit = payments_engine.client_list.get(&ClientId(1)).expect("client id doesn't exist");
		
		let mut transaction_list_mock = HashMap::new();
		transaction_list_mock.insert(first_deposit.transaction_id, first_deposit);
		transaction_list_mock.insert(second_deposit.transaction_id, second_deposit);
		
		let client_after_second_deposit_mock = Client {
			client_id: ClientId(1),
			available: Decimal::TWO,
			held: Decimal::ZERO,
			locked: false,
			transaction_list: transaction_list_mock,
		};

		assert_eq!(client_after_second_deposit, &client_after_second_deposit_mock);
	}
	
	#[test]
	fn withdraw_from_client_id() {
		
		let mut payments_engine = PaymentsEngine {
			client_list: HashMap::new(),
		};
		
		let deposit = Transaction {
			transaction_id: TransactionId(1),
			client_id: ClientId(1),
			type_of_transaction: TransactionType::Deposit,
			amount: Decimal::ONE_HUNDRED,
		};
		
		payments_engine.recv_tx(deposit);
		
		let withdraw = Transaction {
			transaction_id: TransactionId(2),
			client_id: ClientId(1),
			type_of_transaction: TransactionType::Withdraw,
			amount: Decimal::ONE_HUNDRED,
		};
		
		payments_engine.recv_tx(withdraw);
		
		let client_after_withdraw = payments_engine.client_list.get(&ClientId(1)).expect("client id doesn't exist...");
		
		let mut transaction_list_mock = HashMap::new();
		transaction_list_mock.insert(deposit.transaction_id, deposit);
		transaction_list_mock.insert(withdraw.transaction_id, withdraw);
		
		let client_after_withdraw_mock = Client {
			client_id: ClientId(1),
			available: Decimal::ZERO,
			held: Decimal::ZERO,
			locked: false,
			transaction_list: transaction_list_mock,
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
			type_of_transaction: TransactionType::Deposit,
			amount: Decimal::ONE,
		};
		
		payments_engine.recv_tx(deposit);
		
		let withdraw = Transaction {
			transaction_id: TransactionId(2),
			client_id: ClientId(1),
			type_of_transaction: TransactionType::Withdraw,
			amount: Decimal::ONE_HUNDRED,
		};
		
		payments_engine.recv_tx(withdraw);
		
		let client_after_failed_withdraw = payments_engine.client_list.get(&ClientId(1)).expect("client id doesn't exist...");
		
		let mut transaction_list_mock = HashMap::new();
		transaction_list_mock.insert(deposit.transaction_id, deposit);
		transaction_list_mock.insert(withdraw.transaction_id, withdraw);
		
		let client_after_failed_withdraw_mock = Client {
			client_id: ClientId(1),
			available: Decimal::ONE,
			held: Decimal::ZERO,
			locked: false,
			transaction_list: transaction_list_mock,
		};
		
		assert_eq!(client_after_failed_withdraw, &client_after_failed_withdraw_mock);
	}

}















