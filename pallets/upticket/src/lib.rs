#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use sp_std::str;
	use sp_std::vec::Vec;
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	pub type HappeningIndex = u128;
	pub type EventPrice = u32;
	
	pub type BackendUserId = Vec<u128>;

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
		is_paid: bool,
		max_resell_price: EventPrice,
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
	pub(super) type Tickets<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		(HappeningIndex, T::AccountId, BackendUserId),
		TicketInfo,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn accounts)]
	pub(super) type Accounts<T: Config> =
		StorageMap<_, Blake2_128Concat, (T::AccountId, Vec<u8>), AccountInfo, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		EventCreated(HappeningInfo<HappeningIndex, EventPrice>),
		EventResult(HappeningInfo<HappeningIndex, EventPrice>),
		TicketBought(TicketInfo),
		TicketResult(TicketInfo),
		TicketValid(TicketInfo),
	}

	#[pallet::error]
	pub enum Error<T> {
		TicketNotPaid,
		ResellPriceTooHigh,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_event(
			origin: OriginFor<T>,
			price: EventPrice,
			event_index: HappeningIndex,
		) -> DispatchResult {
			ensure_signed(origin)?;

			<Happenings<T>>::insert(event_index, HappeningInfo { id: event_index, price });
			log::info!("{:?}", price);

			Self::deposit_event(Event::EventCreated(HappeningInfo { id: event_index, price }));
			Ok(())
		}

		#[pallet::weight(0_000)]
		pub fn get_event(_origin: OriginFor<T>, event_index: HappeningIndex) -> DispatchResult {
			let hap = <Happenings<T>>::get(event_index);
			Self::deposit_event(Event::EventResult(hap));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn buy_ticket(
			origin: OriginFor<T>,
			event_index: HappeningIndex,
			user_id: BackendUserId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let hap = <Happenings<T>>::get(event_index);
			let tick = TicketInfo { happeningid: hap.id, price: hap.price, is_paid: false, max_resell_price: calc_max_price(hap.price) };
			<Tickets<T>>::insert((&hap.id, &sender, user_id), &tick);
			Self::deposit_event(Event::TicketBought(tick));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn set_ticket_paid_status(
			origin: OriginFor<T>,
			event_index: HappeningIndex,
			user_id: BackendUserId,
			is_paid: bool,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let hap = <Happenings<T>>::get(event_index);
			let tick = TicketInfo { happeningid: hap.id, price: hap.price, is_paid, max_resell_price: calc_max_price(hap.price) };

			<Tickets<T>>::insert((&hap.id, &sender, user_id), &tick);
			Self::deposit_event(Event::TicketBought(tick));
			Ok(())
		}

		#[pallet::weight(0_000)]
		pub fn get_ticket(
			origin: OriginFor<T>,
			event_index: HappeningIndex,
			user_id: BackendUserId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let tick = <Tickets<T>>::get((event_index, &sender, user_id));
			Self::deposit_event(Event::TicketResult(tick));
			Ok(())
		}

		#[pallet::weight(0_000)]
		pub fn check_ticket(
			origin: OriginFor<T>,
			event_index: HappeningIndex,
			user_id: BackendUserId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let tick = <Tickets<T>>::get((event_index, &sender, user_id));

			if tick.is_paid == false {
				Err(Error::<T>::TicketNotPaid)?
			}
			Self::deposit_event(Event::TicketValid(tick));
			Ok(())
		}

		#[pallet::weight(0_000)]
		pub fn user_sell_ticket(
			origin: OriginFor<T>,
			event_index: HappeningIndex,
			seller_user_id: BackendUserId,
			receiver_user_id: BackendUserId,
			selling_price: u32
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let tick = <Tickets<T>>::get((event_index, &sender, &seller_user_id));

			if tick.is_paid == false {
				Err(Error::<T>::TicketNotPaid)?
			}

			if selling_price > tick.max_resell_price {
				Err(Error::<T>::ResellPriceTooHigh)?
			}
			
			<Tickets<T>>::swap((event_index, &sender, &seller_user_id),(event_index, &sender, &receiver_user_id));

			Ok(())
		}
	}

	fn calc_max_price(price: EventPrice) -> u32 {
		return price + ((((price * 100)/ 100) * 20) / 100);
	}
}
