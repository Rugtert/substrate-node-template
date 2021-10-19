#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::str;
	use sp_std::vec::Vec;
	use scale_info::TypeInfo;
	// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}


	pub type HappeningIndex = u128;
	pub type EventPrice = u16;

	pub type Ticket<AccountIdOf, HappeningIndex> = TicketInfo<AccountIdOf, HappeningIndex>;
	
	type HappeningInfoOf = HappeningInfo<HappeningIndex,EventPrice>;


	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Default)]
	#[scale_info(skip_type_params(T))]
	pub struct HappeningInfo<HappeningIndex, EventPrice> {
		id: HappeningIndex,
		price: EventPrice,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Default)]
	#[scale_info(skip_type_params(T))]
	pub struct TicketInfo<AccountIdOf, HappeningIndex> {
		holder: AccountIdOf,
		happeningid: HappeningIndex,
		price: u16,
	}
	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;



	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	#[pallet::getter(fn Happenings)]
	pub(super) type Happenings<T: Config> =
		StorageMap<_, Blake2_128Concat, HappeningIndex, HappeningInfoOf, ValueQuery>;
	
	#[pallet::storage]
	#[pallet::getter(fn fund_count)]
	/// The total number of funds that have so far been allocated.
	pub(super) type HappeningCount<T: Config> = StorageValue<_, HappeningIndex, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when a proof has been claimed. [who, claim]
		ClaimCreated(T::AccountId, Vec<u8>),
		EventCreated(HappeningIndex, EventPrice),
		EventResult(Vec<u128>),
		/// Event emitted when a claim is revoked by the owner. [who, claim]
		ClaimRevoked(T::AccountId, Vec<u8>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The proof has already been claimed.
		ProofAlreadyClaimed,
		/// The proof does not exist, so it cannot be revoked.
		NoSuchProof,
		/// The proof is claimed by another account, so caller can't revoke it.
		NotProofOwner,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
	// Dispatchable functions allows users to interact with the pallet and invoke state changes.

	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_event(
			origin: OriginFor<T>,
			price: u16,
		) -> DispatchResult {
			ensure_signed(origin)?;
			let index = <HappeningCount<T>>::get();
			<HappeningCount<T>>::put(index + 1);
			<Happenings<T>>::insert(index, HappeningInfo {
				id: index,
				price,
			});
			log::info!("{:?}", price);
			// Happening::<T>::insert(&sender,&event_name);
			Self::deposit_event(Event::EventCreated(index, price));
			Ok(())
		}
		#[pallet::weight(0_000)]
		pub fn get_event(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			// let hap = Happening::<T>::get(&sender);
			// Self::deposit_event(Event::EventResult(hap));
			Ok(())
		}
	}
}
