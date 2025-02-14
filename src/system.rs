use num::{
	traits::{CheckedAdd, Zero},
	One,
};
use std::{collections::BTreeMap, ops::AddAssign};

pub trait Config {
	type BlockNumber: CheckedAdd + Zero + One + Copy + AddAssign;
	type AccountID: Ord + Clone;
	type Nonce: CheckedAdd + Zero + One + Copy + AddAssign;
}

/// This is the System Pallet.
/// It handles low level state needed for your blockchain.
#[derive(Debug)]
pub struct Pallet<T: Config> {
	/// The current block number.
	block_number: T::BlockNumber,
	/// A map from an account to their nonce.
	nonce: BTreeMap<T::AccountID, T::Nonce>,
}

impl<T: Config> Pallet<T> {
	/// Create a new instance of the System Pallet.
	pub fn new() -> Self {
		Self { block_number: T::BlockNumber::zero(), nonce: BTreeMap::new() }
	}

	/// Get the current block number.
	pub fn block_number(&self) -> T::BlockNumber {
		self.block_number
	}

	/// Increases the block number by one.
	pub fn inc_block_number(&mut self) {
		self.block_number += T::BlockNumber::one();
	}

	/// Increment the nonce of an account. This helps us keep track of how many transactions each
	/// account has made.
	pub fn inc_nonce(&mut self, who: &T::AccountID) {
		let nonce = *self.nonce.get(who).unwrap_or(&T::Nonce::zero());
		let new_nonce = nonce + T::Nonce::one();

		self.nonce.insert(who.clone(), new_nonce);
	}
}

#[cfg(test)]
mod tests {
	use super::Pallet;
	struct TestConfig;
	impl super::Config for TestConfig {
		type BlockNumber = u32;
		type AccountID = String;
		type Nonce = u32;
	}

	#[test]
	fn init_system() {
		let mut system_pallet = Pallet::<TestConfig>::new();
		system_pallet.inc_block_number();
		assert_eq!(system_pallet.block_number, 1);
		system_pallet.inc_nonce(&"alice".to_string());
		assert_eq!(system_pallet.nonce.get("alice"), Some(&1));
		assert_eq!(system_pallet.nonce.get("bob"), None);
	}
}
