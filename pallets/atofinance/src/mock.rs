use crate as pallet_atofinance;
use crate::traits::*;
use crate::types::{EnumPuzzleStatus, PuzzleSubjectHash};
use crate::*;
use frame_support::sp_runtime::app_crypto::sp_core::sr25519::Signature;
use frame_support::sp_runtime::traits::{IdentifyAccount, Verify};
use frame_support::{
	assert_noop, assert_ok, ord_parameter_types, parameter_types,
	traits::{Contains, GenesisBuild, OnInitialize, SortedMembers},
	weights::Weight,
	PalletId,
};
use frame_system as system;
use lazy_static::lazy_static;
use pallet_balances;
use pallet_balances::{BalanceLock, Error as BalancesError};
use sp_core::H256;
use sp_runtime::RuntimeDebug;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Permill,
};
use std::collections::HashMap;
use std::sync::Mutex;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
// pub(crate) type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub(crate) type AccountId = u64;
/// Balance of an account.
pub type Balance = u128;
pub type BlockNumber = u64;
pub const DOLLARS: Balance = 1_000_000_000_000;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		AtochaPot: pallet_atofinance::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const AresFinancePalletId: PalletId = PalletId(*b"ocw/fund");
	pub const BasicDollars: Balance = DOLLARS;
	pub const TicketFee: Balance = 5 * DOLLARS;
	pub const DepositFee: Balance = 100 * DOLLARS;
	pub const DayBlockCount: u32 = 14400;
	pub const StakingPeriod: u32 = 10;
	pub const PerEraOfBlockNumber: BlockNumber = 5;
	pub TargetIssuanceRate: Permill = Permill::from_float(0.1);
	pub ChallengeThreshold: Perbill = Perbill::from_float(0.6);
	pub RaisingPeriodLength: BlockNumber = 5;
}

impl crate::imps::challenge_manager::Config for Test {
	type ChallengeThreshold = ChallengeThreshold;
	type RaisingPeriodLength = RaisingPeriodLength;
}

impl crate::Config for Test {
	type Event = Event;
	type PalletId = AresFinancePalletId;
	type Currency = pallet_balances::Pallet<Self>;
	type SlashHandler = ();
	type RewardHandler = ();
	type BasicDollars = BasicDollars;
	type TicketFee = TicketFee;
	type DepositFee = DepositFee;
	type DayBlockCount = DayBlockCount;
	type StakingPeriod = StakingPeriod;
	type TargetIssuanceRate = TargetIssuanceRate;
	type PerEraOfBlockNumber = PerEraOfBlockNumber;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxLocks: u32 = 10;
}
impl pallet_balances::Config for Test {
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type MaxLocks = MaxLocks;
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(1, 100000000000000),
			(2, 200000000000000),
			(3, 300000000000000),
			(4, 400000000000000),
			(5, 500000000000000),
			(6, 600000000000000),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	crate::GenesisConfig::<Test> { _pt: Default::default() }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn toVec(to_str: &str) -> Vec<u8> {
	to_str.as_bytes().to_vec()
}

pub(crate) fn init_puzzle_ledger(puzzle_hash: Vec<u8>) {
	const ACCOUNT_ID_1: u64 = 1;
	const ACCOUNT_ID_2: u64 = 2;
	// Dispatch a signed extrinsic.
	assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 100_000_000_000_000);
	assert_eq!(Balances::free_balance(ACCOUNT_ID_2), 200_000_000_000_000);

	// puzzle must exists.
	assert_ok!(AtochaPot::do_bonus(
		puzzle_hash.clone(),
		ACCOUNT_ID_1,
		10_000_000_000_000,
		5u32.into()
	));
	assert_ok!(AtochaPot::do_sponsorship(
		puzzle_hash.clone(),
		ACCOUNT_ID_1,
		20_000_000_000_000,
		15u32.into(), // block number
		"Some-Things-1".as_bytes().to_vec()
	));
	assert_ok!(AtochaPot::do_sponsorship(
		puzzle_hash.clone(),
		ACCOUNT_ID_2,
		30_000_000_000_000,
		30u32.into(), // block number
		"Some-Things-2".as_bytes().to_vec()
	));
	let pot_ledger = AtoFinanceLedger::<Test>::get(&puzzle_hash);
	assert_eq!(pot_ledger.funds, 10_000_000_000_000);
	assert_eq!(pot_ledger.total, 60_000_000_000_000);
	assert_eq!(pot_ledger.sponsor_list.len(), 3);
	assert_eq!(<TokenReward<Test>>::get_total_bonus(&puzzle_hash, 0), Some(60_000_000_000_000));
}