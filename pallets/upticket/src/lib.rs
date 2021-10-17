#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	use sp_std::str;
	use super::*;
	// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	pub type HappeningIndex = u32;
	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	type HappeningInfoOf<T> =
	HappeningInfo<AccountIdOf<T>>;

	pub struct HappeningInfo<AccountId> {
		host: AccountId,
		price: u8,
		eventname: Vec<u8>,
	}

	#[derive(Encode, Decode, Default, PartialEq, Eq)]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct Ticket<AccountId> {
		holder: AccountId,
		happeningid: u32,
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when a proof has been claimed. [who, claim]
		ClaimCreated(T::AccountId, Vec<u8>),
		EventCreated(T::AccountId, Vec<u8>),
		EventResult(Vec<u8>),
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

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub(super) type Happening<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<u8>, ValueQuery>;
	
	#[pallet::storage]
	pub(super) type TicketStore<T: Config> = StorageDoubleMap<_, Blake2_128Concat, T::AccountId,Blake2_128Concat, Vec<u8>,Vec<u8>,ValueQuery>;
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
	// Dispatchable functions allows users to interact with the pallet and invoke state changes.

	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_event(origin: OriginFor<T>, event_name: Vec<u8>, price: u8) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let current_events = Happening::<T>::get(&sender);
			log::info!("{:?}", price);
			Happening::<T>::insert(&sender,&event_name);
			Self::deposit_event(Event::EventCreated(sender, event_name));
			Ok(())
		}
		
		#[pallet::weight(0_000)]
		pub fn get_event(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let hap = Happening::<T>::get(&sender);
			Self::deposit_event(Event::EventResult(hap));
			Ok(())		
		}
	}
}
