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
	use scale_info::TypeInfo;
	use sp_std::str;
	use sp_std::vec::Vec;
	// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	pub type HappeningIndex = u128;
	pub type EventPrice = u32;
	pub type MaxSellPrice = u32;

	type HappeningInfoOf = HappeningInfo<HappeningIndex, EventPrice>;
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Default)]
	#[scale_info(skip_type_params(T))]
	pub struct HappeningInfo<HappeningIndex, EventPrice> {
		id: HappeningIndex,
		price: EventPrice,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Default)]
	#[scale_info(skip_type_params(T))]
	pub struct TicketInfo {
		happeningid: HappeningIndex,
		price: EventPrice,
		// sellable: bool,
		// max_sell_price: MaxSellPrice,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Default)]
	#[scale_info(skip_type_params(T))]
	pub struct AccountInfo {
		backend_userid: Vec<u8>,
		organisation: Vec<u8>,
	}


	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn happenings)]
	pub(super) type Happenings<T: Config> =
		StorageMap<_, Blake2_128Concat, HappeningIndex, HappeningInfoOf, ValueQuery>;
	#[pallet::storage]
	#[pallet::getter(fn tickets)]
	pub(super) type Tickets<T: Config> =
		StorageMap<_, Blake2_128Concat, (HappeningIndex, T::AccountId, Vec<u128>), TicketInfo, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn accounts)]
	pub(super) type Accounts<T: Config> =
		StorageMap<_, Blake2_128Concat, (T::AccountId, Vec<u8>), AccountInfo, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn fund_count)]
	/// The total number of events that have been created
	pub(super) type HappeningCount<T: Config> = StorageValue<_, HappeningIndex, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when a proof has been claimed. [who, claim]
		ClaimCreated(T::AccountId, Vec<u8>),
		EventCreated(HappeningInfo<HappeningIndex, EventPrice>),
		EventResult(HappeningInfo<HappeningIndex, EventPrice>),
		TicketBought(TicketInfo),
		TicketResult(TicketInfo),
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
			price: u32,
			event_index: Vec<u8>,
		) -> DispatchResult {
			ensure_signed(origin)?;
			let index = <HappeningCount<T>>::get();
			<HappeningCount<T>>::put(index + 1);
			<Happenings<T>>::insert(index, HappeningInfo { id: index, price });
			log::info!("{:?}", price);

			Self::deposit_event(Event::EventCreated(HappeningInfo { id: index, price }));
			Ok(())
		}
		#[pallet::weight(0_000)]
		pub fn get_event(origin: OriginFor<T>, index: u128) -> DispatchResult {
			let hap = <Happenings<T>>::get(index);
			Self::deposit_event(Event::EventResult(hap));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn buy_ticket(origin: OriginFor<T>, event_index: u128, user_id: Vec<u128> ) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let hap = <Happenings<T>>::get(event_index);
			let tick = TicketInfo { happeningid: hap.id, price: hap.price };
			<Tickets<T>>::insert((&hap.id, &sender, user_id ), &tick);
			Self::deposit_event(Event::TicketBought(tick));
			Ok(())
		}

		#[pallet::weight(0_000)]
		pub fn get_ticket(origin: OriginFor<T>, index: u128, user_id: Vec<u128>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let tick = <Tickets<T>>::get((index, &sender, user_id));
			Self::deposit_event(Event::TicketResult(tick));
			Ok(())
		}
	}
}
