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
}

#[cfg(test)]
mod tests {
    use super::*;
	
	#[test]
	fn account_constructor() {
		let id = AccountId(1);
		let amount = Decimal::ONE_HUNDRED; 
		assert_eq!(
			Account::new_with_deposit(id, amount),
			Account {
				id: AccountId(1),
				available: Decimal::ONE_HUNDRED,
				held: Decimal::ZERO,
				locked: false,	
			}
		);
	}
    #[test]
    fn deposit_to_non_existant_client_id() {
        let mut payments_engine = PaymentsEngine {
			account_list: HashMap::new(),
		};
        payments_engine.send_tx(AccountId(1), Transaction::Deposit(Decimal::ONE_HUNDRED));
        let account = payments_engine.account_list.get(&AccountId(1)).expect("id doesn't exist...");
        assert_eq!(account.available, Decimal::new(100, 0));
    }
	 

    /*
    #[test]
    fn deposit_to_client_id() {
        let mut clients_map = HashMap::new();
        clients_map.insert(1, Client { id: 1, available: Decimal::ZERO, held: Decimal::ZERO, locked: false });
        clients_map.send_tx();
        let client = clients_map.map.get(&id).expect("error");
*/
}
