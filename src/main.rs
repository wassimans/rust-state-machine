mod balances;
mod proof_of_existence;
mod support;
mod system;

use crate::support::Dispatch;

mod types {

	pub type AccountID = String;
	pub type Balance = u128;
	pub type BlockNumber = u32;
	pub type Nonce = u32;
	pub type Extrinsic = crate::support::Extrinsic<AccountID, crate::RuntimeCall>;
	pub type Header = crate::support::Header<BlockNumber>;
	pub type Block = crate::support::Block<Header, Extrinsic>;
	pub type Content = &'static str;
}

pub enum RuntimeCall {
	Balances(balances::Call<Runtime>),
	ProofOfExistence(proof_of_existence::Call<Runtime>),
}

// This is our main Runtime.
// It accumulates all of the different pallets we want to use.
#[derive(Debug)]
pub struct Runtime {
	system: system::Pallet<Self>,
	balances: balances::Pallet<Self>,
	proof_of_existence: proof_of_existence::Pallet<Self>,
}

impl system::Config for Runtime {
	type BlockNumber = types::BlockNumber;

	type AccountID = types::AccountID;

	type Nonce = types::Nonce;
}

impl balances::Config for Runtime {
	type Balance = types::Balance;
}

impl proof_of_existence::Config for Runtime {
	type Content = types::Content;
}

impl Runtime {
	// Create a new instance of the main Runtime, by creating a new instance of each pallet.
	fn new() -> Self {
		Self {
			system: system::Pallet::new(),
			balances: balances::Pallet::new(),
			proof_of_existence: proof_of_existence::Pallet::new(),
		}
	}

	// Execute a block of extrinsics. Increments the block number.
	fn execute_block(&mut self, block: types::Block) -> support::DispatchResult {
		self.system.inc_block_number();
		if self.system.block_number() != block.header.block_number {
			return Err("The imported block number doesn't match the current's");
		}

		for (i, support::Extrinsic { caller, call }) in block.extrinsics.into_iter().enumerate() {
			self.system.inc_nonce(&caller);
			let _res = self.dispatch(caller, call).map_err(|e| {
				eprintln!(
					"Extrinsic Error\n\tBlock Number: {}\n\tExtrinsic Number: {}\n\tError: {}",
					block.header.block_number, i, e
				)
			});
		}

		Ok(())
	}
}

impl crate::support::Dispatch for Runtime {
	type Caller = <Runtime as system::Config>::AccountID;
	type Call = RuntimeCall;
	// Dispatch a call on behalf of a caller. Increments the caller's nonce.
	//
	// Dispatch allows us to identify which underlying module call we want to execute.
	// Note that we extract the `caller` from the extrinsic, and use that information
	// to determine who we are executing the call on behalf of.
	fn dispatch(
		&mut self,
		caller: Self::Caller,
		runtime_call: Self::Call,
	) -> support::DispatchResult {
		match runtime_call {
			RuntimeCall::Balances(balances::Call::Transfer { to, amount }) => {
				self.balances.transfer(caller, to, amount)?;
			},
			RuntimeCall::ProofOfExistence(proof_of_existence::Call::CreateClaim {
				caller,
				claim,
			}) => {
				self.proof_of_existence.create_claim(caller, claim)?;
			},
			RuntimeCall::ProofOfExistence(proof_of_existence::Call::RevokeClaim {
				caller,
				claim,
			}) => {
				self.proof_of_existence.revoke_claim(caller, claim)?;
			},
		}
		Ok(())
	}
}

fn main() {
	let mut runtime = Runtime::new();
	let alice = "alice".to_string();
	let bob = "bob".to_string();
	let charlie = "charlie".to_string();
	runtime.balances.set_balance(&alice, 100);
	let alice_content = "The Book of Alice";
	let bob_content = "The Book of Bob";

	let block_genesis = types::Block {
		header: support::Header { block_number: 1 },
		extrinsics: vec![
			support::Extrinsic {
				caller: alice.clone(),
				call: RuntimeCall::Balances(balances::Call::Transfer {
					to: bob.clone(),
					amount: 30,
				}),
			},
			support::Extrinsic {
				caller: alice.clone(),
				call: RuntimeCall::Balances(balances::Call::Transfer {
					to: charlie.clone(),
					amount: 20,
				}),
			},
		],
	};

	let block_2 = types::Block {
		header: support::Header { block_number: 2 },
		extrinsics: vec![
			support::Extrinsic {
				caller: alice.clone(),
				call: RuntimeCall::ProofOfExistence(proof_of_existence::Call::CreateClaim {
					caller: alice.clone(),
					claim: alice_content,
				}),
			},
			support::Extrinsic {
				caller: bob.clone(),
				call: RuntimeCall::ProofOfExistence(proof_of_existence::Call::CreateClaim {
					caller: bob.clone(),
					claim: bob_content,
				}),
			},
		],
	};

	let block_3 = types::Block {
		header: support::Header { block_number: 3 },
		extrinsics: vec![support::Extrinsic {
			caller: alice.clone(),
			call: RuntimeCall::ProofOfExistence(proof_of_existence::Call::RevokeClaim {
				caller: alice.clone(),
				claim: alice_content,
			}),
		}],
	};

	let block_4 = types::Block {
		header: support::Header { block_number: 4 },
		extrinsics: vec![support::Extrinsic {
			caller: bob.clone(),
			call: RuntimeCall::ProofOfExistence(proof_of_existence::Call::CreateClaim {
				caller: bob.clone(),
				claim: bob_content,
			}),
		}],
	};

	runtime.execute_block(block_genesis).expect("Invalid block");
	runtime.execute_block(block_2).expect("Invalid block");
	runtime.execute_block(block_3).expect("Invalid block");
	runtime.execute_block(block_4).expect("Invalid block");

	println!("{:#?}", runtime);
}
