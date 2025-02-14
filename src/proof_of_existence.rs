use core::fmt::Debug;
use std::collections::BTreeMap;

use crate::support::DispatchResult;

pub trait Config: crate::system::Config {
	/// The type which represents the content that can be claimed using this pallet.
	/// Could be the content directly as bytes, or better yet the hash of that content.
	/// We leave that decision to the runtime developer.
	type Content: Debug + Ord;
}

/// This is the Proof of Existence Module.
/// It is a simple module that allows accounts to claim existence of some data.
#[derive(Debug)]
pub struct Pallet<T: Config> {
	/// A simple storage map from content to the owner of that content.
	/// Accounts can make multiple different claims, but each claim can only have one owner.
	claims: BTreeMap<T::Content, T::AccountID>,
}

impl<T: Config> Pallet<T> {
	/// Create a new instance of the Proof of Existence Module.
	pub fn new() -> Self {
		Self { claims: BTreeMap::new() }
	}

	/// Get the owner (if any) of a claim.
	pub fn get_claim(&self, claim: &T::Content) -> Option<&T::AccountID> {
		match self.claims.contains_key(claim) {
			true => self.claims.get(claim),
			false => None,
		}
	}

	/// Create a new claim on behalf of the `caller`.
	/// This function will return an error if someone already has claimed that content.
	pub fn create_claim(&mut self, caller: T::AccountID, claim: T::Content) -> DispatchResult {
		match self.claims.contains_key(&claim) {
			true => Err("Claim already exists"),
			false => {
				let _res = self.claims.insert(claim, caller);
				Ok(())
			},
		}
	}

	/// Revoke an existing claim on some content.
	/// This function should only succeed if the caller is the owner of an existing claim.
	/// It will return an error if the claim does not exist, or if the caller is not the owner.
	pub fn revoke_claim(&mut self, caller: T::AccountID, claim: T::Content) -> DispatchResult {
		if let Some(owner) = self.get_claim(&claim) {
			if *owner == caller {
				let _res = self.claims.remove(&claim);
				Ok(())
			} else {
				Err("Not the owner")
			}
		} else {
			Err("Claim does not exist")
		}
	}
}

pub enum Call<T: Config> {
	CreateClaim { caller: T::AccountID, claim: T::Content },
	RevokeClaim { caller: T::AccountID, claim: T::Content },
}

#[cfg(test)]
mod test {
	use super::Pallet;

	struct TestConfig;

	impl super::Config for TestConfig {
		type Content = &'static str;
	}

	impl crate::system::Config for TestConfig {
		type AccountID = &'static str;
		type BlockNumber = u32;
		type Nonce = u32;
	}

	#[test]
	fn basic_proof_of_existence() {
		let mut pallet = Pallet::<TestConfig>::new();

		assert!(pallet.claims.is_empty());
		let _ = pallet.create_claim("Alice", "The Book of Alice");
		assert!(!pallet.claims.is_empty());

		let result = pallet.create_claim("Alice", "The Book of Alice");
		assert_eq!(result, Err("Claim already exists"));
		let bob_claim = pallet.get_claim(&"The Book of Bob");
		assert_eq!(bob_claim, None);
		let result = pallet.revoke_claim("Bob", "The Book of Bob");
		assert_eq!(result, Err("Claim does not exist"));
		let result = pallet.revoke_claim("Bob", "The Book of Alice");
		assert_eq!(result, Err("Not the owner"));

		let result = pallet.revoke_claim("Alice", "The Book of Alice");
		assert_eq!(result, Ok(()));

		assert!(pallet.claims.is_empty());
	}
}
