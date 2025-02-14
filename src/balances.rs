use num::traits::{CheckedAdd, CheckedSub, Zero};
use std::collections::BTreeMap;

use crate::{
	support::{self, DispatchResult},
	system,
};

pub trait Config: system::Config {
	type Balance: CheckedAdd + CheckedSub + Zero + Copy;
}

#[derive(Debug)]
pub struct Pallet<T: Config> {
	balances: BTreeMap<T::AccountID, T::Balance>,
}

impl<T: Config> Pallet<T> {
	/// Create a new instance of the balances module.
	pub fn new() -> Self {
		Self { balances: BTreeMap::new() }
	}

	/// Set the balance of an account `who` to some `amount`.
	pub fn set_balance(&mut self, who: &T::AccountID, amount: T::Balance) {
		self.balances.insert(who.clone(), amount);
	}

	/// Get the balance of an account `who`.
	/// If the account has no stored balance, we return zero.
	pub fn balance(&self, who: &T::AccountID) -> T::Balance {
		*self.balances.get(who).unwrap_or(&T::Balance::zero())
	}

	/// Transfer `amount` from one account to another.
	/// This function verifies that `from` has at least `amount` balance to transfer,
	/// and that no mathematical overflows occur.
	pub fn transfer(
		&mut self,
		caller: T::AccountID,
		to: T::AccountID,
		amount: T::Balance,
	) -> DispatchResult {
		let caller_balance = self.balance(&caller);
		let to_balance = self.balance(&to);

		let new_caller_balance = caller_balance.checked_sub(&amount).ok_or("Not enough funds")?;
		let new_to_balance = to_balance.checked_add(&amount).ok_or("Amount is too large")?;

		self.set_balance(&caller, new_caller_balance);
		self.set_balance(&to, new_to_balance);

		Ok(())
	}
}

pub enum Call<T: Config> {
	Transfer { to: T::AccountID, amount: T::Balance },
}

impl<T: Config> crate::support::Dispatch for Pallet<T> {
	type Caller = T::AccountID;
	type Call = Call<T>;
	// Dispatch a call on behalf of a caller. Increments the caller's nonce.
	//
	// Dispatch allows us to identify which underlying module call we want to execute.
	// Note that we extract the `caller` from the extrinsic, and use that information
	// to determine who we are executing the call on behalf of.
	fn dispatch(&mut self, caller: Self::Caller, call: Self::Call) -> support::DispatchResult {
		match call {
			Call::Transfer { to, amount } => {
				self.transfer(caller, to, amount)?;
			},
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use num::Zero;

	use crate::system;

	use super::Pallet;
	struct TestConfig;
	impl system::Config for TestConfig {
		type BlockNumber = u32;

		type AccountID = String;

		type Nonce = u32;
	}
	impl super::Config for TestConfig {
		type Balance = u32;
	}

	#[test]
	fn init_balances() {
		let mut balances = Pallet::<TestConfig>::new();

		assert_eq!(balances.balance(&"alice".to_string()), Zero::zero());
		balances.set_balance(&"alice".to_string(), 100);
		assert_eq!(balances.balance(&"alice".to_string()), 100);
		assert_eq!(balances.balance(&"bob".to_string()), Zero::zero())
	}

	#[test]
	fn transfer_balance() {
		let mut balances = Pallet::<TestConfig>::new();
		balances.set_balance(&"alice".to_string(), 50);
		balances.set_balance(&"bob".to_string(), 100);

		let result = balances.transfer("alice".to_string(), "bob".to_string(), 100);
		assert_eq!(result, Err("Not enough funds"));

		let result = balances.transfer("alice".to_string(), "bob".to_string(), 20);
		assert_eq!(result, Ok(()));

		assert_eq!(balances.balance(&"alice".to_string()), 30);
		assert_eq!(balances.balance(&"bob".to_string()), 120);
	}
}
