use sp_runtime::traits::Block as _;

mod pallet_old {
	use frame_support::{
		decl_storage, decl_error, decl_event, decl_module, weights::Weight, traits::Get, Parameter
	};
	use frame_system::ensure_root;

	pub trait Trait: frame_system::Trait {
		type SomeConst: Get<Self::Balance>;
		type Balance: Parameter + codec::HasCompact + From<u32> + Into<Weight> + Default;
		type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	}

	decl_storage! {
		trait Store for Module<T: Trait> as Example {
			/// Some documentation
			Dummy get(fn dummy) config(): Option<T::Balance>;
			Bar get(fn bar) config(): map hasher(blake2_128_concat) T::AccountId => T::Balance;
			Foo get(fn foo) config(): T::Balance = 3.into();
			Double get(fn double): double_map hasher(blake2_128_concat) u32, hasher(twox_64_concat) u64 => u16;
		}
	}

	decl_event!(
		pub enum Event<T> where Balance = <T as Trait>::Balance {
			/// Dummy event, just here so there's a generic type that's used.
			Dummy(Balance),
		}
	);

	decl_module! {
		pub struct Module<T: Trait> for enum Call where origin: T::Origin {
			type Error = Error<T>;
			fn deposit_event() = default;
			const SomeConst: T::Balance = T::SomeConst::get();

			#[weight = <T::Balance as Into<Weight>>::into(new_value.clone())]
			fn set_dummy(origin, #[compact] new_value: T::Balance) {
				ensure_root(origin)?;

				<Dummy<T>>::put(&new_value);
				Self::deposit_event(RawEvent::Dummy(new_value));
			}

			fn on_initialize(_n: T::BlockNumber) -> Weight {
				<Dummy<T>>::put(T::Balance::from(10));
				10
			}

			fn on_finalize(_n: T::BlockNumber) {
				<Dummy<T>>::put(T::Balance::from(11));
			}
		}
	}

	decl_error! {
		pub enum Error for Module<T: Trait> {
			/// Some wrong behavior
			Wrong,
		}
	}
}

#[frame_support::pallet(Example)]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_system::ensure_root;

	#[pallet::trait_]
	pub trait Trait: frame_system::Trait {
		type Balance: Parameter + codec::HasCompact + From<u32> + Into<Weight> + Default
			+ MaybeSerializeDeserialize;
		#[pallet::const_]
		type SomeConst: Get<Self::Balance>;
		type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	}

	#[pallet::module]
	pub struct Module<T>(PhantomData<T>);

	#[pallet::module_interface]
	impl<T: Trait> ModuleInterface<T::BlockNumber> for Module<T> {
		fn on_initialize(_n: T::BlockNumber) -> Weight {
			<Dummy<T>>::put(T::Balance::from(10));
			10
		}

		fn on_finalize(_n: T::BlockNumber) {
			<Dummy<T>>::put(T::Balance::from(11));
		}
	}

	#[pallet::call]
	impl<T: Trait> Call for Module<T> {
		#[pallet::weight(<T::Balance as Into<Weight>>::into(new_value.clone()))]
		fn set_dummy(origin: OriginFor<T>, #[pallet::compact] new_value: T::Balance) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			<Dummy<T>>::put(&new_value);
			frame_system::Module::<T>::deposit_event(<<T as Trait>::Event as From<_>>::from(Event::<T>::Dummy(new_value))); // TODO TODO: better way to deposit event 
			// frame_system::Module::<T>::deposit_event(<T as Trait>::Event::from(Event::<T>::Dummy(new_value))); // TODO TODO: span for this error ?

			Ok(().into())
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Some wrong behavior
		Wrong,
	}

	#[pallet::event]
	pub enum Event<T: Trait> {
		/// Dummy event, just here so there's a generic type that's used.
		Dummy(T::Balance),
	}

	#[pallet::storage] #[allow(type_alias_bounds)]
	/// Some documentation
	type Dummy<T: Trait> = StorageValueType<DummyP, T::Balance, OptionQuery>;

	#[pallet::storage] #[allow(type_alias_bounds)]
	type Bar<T: Trait> = StorageMapType<BarP, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;

	pub struct OnFooEmpty<T: Trait>(PhantomData<T>); // TODO TODO: maybe allow faster declaration with parameter_types
	impl<T: Trait> Get<T::Balance> for OnFooEmpty<T> {
		fn get() -> T::Balance {
			3.into()
		}
	}
	#[pallet::storage] #[allow(type_alias_bounds)]
	type Foo<T: Trait> = StorageValueType<FooP, T::Balance, ValueQuery, OnFooEmpty<T>>;

	#[pallet::storage] #[allow(type_alias_bounds)]
	type Double = StorageDoubleMapType<
		DoubleP, Blake2_128Concat, u32, Twox64Concat, u64, u16, ValueQuery
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Trait> {
		dummy: Option<T::Balance>,
		bar: Vec<(T::AccountId, T::Balance)>,
		foo: T::Balance,
	}

	impl<T: Trait> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				dummy: Default::default(),
				bar: Default::default(),
				foo: OnFooEmpty::<T>::get(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Trait> GenesisBuilder<T> for GenesisConfig<T> {
		fn build(&self) {
			if let Some(dummy) = self.dummy.as_ref() {
				<Dummy<T>>::put(dummy);
			}
			for (k, v) in &self.bar {
				<Bar<T>>::insert(k, v);
			}
			<Foo<T>>::put(&self.foo);
		}
	}
}

frame_support::parameter_types!(
	pub const SomeConst: u64 = 10;
	pub const BlockHashCount: u32 = 250;
	pub const MaximumBlockWeight: frame_support::weights::Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: sp_runtime::Perbill = sp_runtime::Perbill::one();
);

impl frame_system::Trait for Runtime {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u32;
	type Call = Call;
	type Hash = sp_runtime::testing::H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type AccountId = u64;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = frame_support::weights::constants::RocksDbWeight;
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type AvailableBlockRatio = AvailableBlockRatio;
	type MaximumBlockLength = MaximumBlockLength;
	type Version = ();
	type ModuleToIndex = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}
impl pallet::Trait for Runtime {
	type Event = Event;
	type SomeConst = SomeConst;
	type Balance = u64;
}
impl pallet_old::Trait for Runtime {
	type Event = Event;
	type SomeConst = SomeConst;
	type Balance = u64;
}

pub type Header = sp_runtime::generic::Header<u32, sp_runtime::traits::BlakeTwo256>;
pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, Call, (), ()>;

frame_support::construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Module, Call, Event<T>},
		Pallet: pallet::{Module, Call, Event<T>, Config<T>, Storage},
		PalletOld: pallet_old::{Module, Call, Event<T>, Config<T>, Storage},
	}
);

#[cfg(test)]
mod test {
	use super::Runtime;
	use super::pallet;
	use super::pallet_old;
	use codec::{Decode, Encode};

	#[test]
	fn metadata() {
		let metadata = Runtime::metadata();
		let modules = match metadata.1 {
			frame_metadata::RuntimeMetadata::V11(frame_metadata::RuntimeMetadataV11 {
				modules: frame_metadata::DecodeDifferent::Encode(m),
				..
			}) => m,
			_ => unreachable!(),
		};
		pretty_assertions::assert_eq!(modules[1].storage, modules[2].storage);
		pretty_assertions::assert_eq!(modules[1].calls, modules[2].calls);
		pretty_assertions::assert_eq!(modules[1].event, modules[2].event);
		pretty_assertions::assert_eq!(modules[1].constants, modules[2].constants);
		pretty_assertions::assert_eq!(modules[1].errors, modules[2].errors);
	}

	#[test]
	fn types() {
		assert_eq!(
			pallet_old::Event::<Runtime>::decode(&mut &pallet::Event::<Runtime>::Dummy(10).encode()[..]).unwrap(),
			pallet_old::Event::<Runtime>::Dummy(10),
		);

		assert_eq!(
			pallet_old::Call::<Runtime>::decode(&mut &pallet::Call::<Runtime>::set_dummy(10).encode()[..]).unwrap(),
			pallet_old::Call::<Runtime>::set_dummy(10),
		);
	}

	// TODO TODO: add some test for execution
}