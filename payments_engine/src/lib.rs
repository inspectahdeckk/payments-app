#![allow(dead_code)]

use std::collections::HashMap;
use rust_decimal::Decimal;

#[derive(Debug)]
struct PaymentsEngine {
    account_list: HashMap<AccountId, Account>
}

impl PaymentsEngine {

    fn send_tx(&mut self, id: AccountId, tx: Transaction) {
        match tx {
            Transaction::Deposit(amount) => {
				self.account_list
					.entry(id)
					.and_modify(|account| account.available += amount) 
					.or_insert(Account::new_with_deposit(id, amount));
            },
			Transaction::Withdraw(amount) => {
				self.account_list
					.entry(id)
					.and_modify(|account| {
						if amount > account.available {
							eprintln!("Withdraw amount is bigger than available amount...");
						} else {
							account.available -= amount
						}
					}); 
			}
        }
    }

}

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
struct AccountId(u16);


#[derive(Debug, PartialEq)]
struct Account {
    id: AccountId,
    available: Decimal,
    held: Decimal,
    locked: bool,
}

impl Account {
	fn new_with_deposit(id: AccountId, amount: Decimal) -> Account {
		Account {
			id,
			available: amount,
			held: Decimal::ZERO,
			locked: false,
		}
	}
}

enum Transaction {
    Deposit(Decimal),
	Withdraw(Decimal),
}

/********** TESTS **********/
#[cfg(test)]
mod tests {
    use super::*;
	
    #[test]
    fn deposit_to_non_existing_client_id() {
        let mut payments_engine = PaymentsEngine {
			account_list: HashMap::new(),
		};
        payments_engine.send_tx(AccountId(1), Transaction::Deposit(Decimal::ONE_HUNDRED));
        let account = payments_engine.account_list.get(&AccountId(1)).expect("id doesn't exist...");
        assert_eq!(account.available, Decimal::new(100, 0));
    }
	 
    #[test]
    fn deposit_to_existing_client_id() {
		let mut payments_engine = PaymentsEngine {
			account_list: HashMap::new(),
		};
		let account = Account {
			id: AccountId(1),
			available: Decimal::ZERO,
			held: Decimal::ZERO,
			locked: false,
		};
		payments_engine.account_list.insert(AccountId(1), account);
		payments_engine.send_tx(AccountId(1), Transaction::Deposit(Decimal::ONE_HUNDRED));
		let account = payments_engine.account_list.get(&AccountId(1)).expect("id doesn't exist...");
		let amount = account.available;	
		assert_eq!(amount, Decimal::ONE_HUNDRED);
	}

	#[test]
	fn withdraw_from_client_id() {
		let mut payments_engine = PaymentsEngine {
			account_list: HashMap::new(),
		};
		let account = Account {
			id: AccountId(1),
			available: Decimal::ONE_HUNDRED,
			held: Decimal::ZERO,
			locked: false,
		};
		payments_engine.account_list.insert(AccountId(1), account);
		payments_engine.send_tx(AccountId(1), Transaction::Withdraw(Decimal::ONE_HUNDRED));
		let account = payments_engine.account_list.get(&AccountId(1)).expect("id doesn't exist...");
		let amount = account.available;	
		assert_eq!(amount, Decimal::ZERO);
	}

	#[test]
	fn withdraw_insufficient_amount_from_client_id() {
		let mut payments_engine = PaymentsEngine {
			account_list: HashMap::new(),
		};
		let account = Account {
			id: AccountId(1),
			available: Decimal::ONE,
			held: Decimal::ZERO,
			locked: false,
		};
		payments_engine.account_list.insert(AccountId(1), account);
		payments_engine.send_tx(AccountId(1), Transaction::Withdraw(Decimal::ONE_HUNDRED));
		let account = payments_engine.account_list.get(&AccountId(1)).expect("id doesn't exist...");
		let amount = account.available;	
		assert_eq!(amount, Decimal::ONE);
	}

}















