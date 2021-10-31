#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use sp_std::vec::Vec;

	pub type EvenementId = u128;
	pub type Prijs = u32;
	pub type MaxPrijs = u32;
	pub type Naam = Vec<u8>;
	pub type KlantId = u128;
	pub type AantalTickets = u128;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Default)]
	#[scale_info(skip_type_params(T))]
	pub struct Evenement<EvenementId, Naam, Prijs, MaxPrijs, AantalTickets> {
		id: EvenementId,
		naam: Naam,
		prijs: Prijs,
		max_prijs: MaxPrijs,
		aantal_tickets: AantalTickets,
	}

	#[pallet::storage]
	#[pallet::getter(fn evenementen)]
	pub(super) type Evenementen<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		EvenementId,
		Evenement<EvenementId, Naam, Prijs, MaxPrijs, AantalTickets>,
		ValueQuery,
	>;

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Default)]
	#[scale_info(skip_type_params(T))]
	pub struct Ticket<Bool> {
		is_gescand: Bool,
	}

	#[pallet::storage]
	#[pallet::getter(fn tickets)]
	pub(super) type Tickets<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		(EvenementId, KlantId),
		Ticket<bool>,
		ValueQuery,
	>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		EvenementAangemaakt(Evenement<EvenementId, Naam, Prijs, MaxPrijs, AantalTickets>),
		TicketAangemaakt(Ticket<bool>),
		Evenement(Evenement<EvenementId, Naam, Prijs, MaxPrijs, AantalTickets>),
		Ticket(Ticket<bool>),
		Beschikbaarheid(u128),
	}

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
		TicketNotFound,
		EvenementNotFound,
		MaxPriceExceeded,
		NoTicketsAvailable,
		TicketAlreadyScanned,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_evenement(
			origin: OriginFor<T>,
			prijs: Prijs,
			max_prijs: MaxPrijs,
			naam: Naam,
			id: EvenementId,
			aantal_tickets: AantalTickets,
		) -> DispatchResult {
			ensure_signed(origin)?;

			let evenement = Evenement { id, prijs, max_prijs, naam, aantal_tickets };

			<Evenementen<T>>::insert(id, evenement.clone());

			Self::deposit_event(Event::EvenementAangemaakt(evenement));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn get_evenement(origin: OriginFor<T>, id: EvenementId) -> DispatchResult {
			ensure_signed(origin)?;

			let evenement = <Evenementen<T>>::get(id);

			if evenement.id == 0 {
				Err(Error::<T>::EvenementNotFound)?
			}

			Self::deposit_event(Event::Evenement(evenement));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn get_evenement_beschikbaarheid(origin: OriginFor<T>, id: EvenementId) -> DispatchResult {
			ensure_signed(origin)?;

			let evenement = <Evenementen<T>>::get(id);
			if evenement.id == 0 {
				Err(Error::<T>::EvenementNotFound)?
			}

			let beschikbaarheid = Self::get_beschikbaarheid(id);

			Self::deposit_event(Event::Beschikbaarheid(beschikbaarheid));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn buy_ticket(
			origin: OriginFor<T>,
			evenement_id: EvenementId,
			klant_id: KlantId,
			is_gescand: bool,
		) -> DispatchResult {
			ensure_signed(origin)?;

			let evenement = <Evenementen<T>>::get(evenement_id);

			if evenement.id == 0 {
				Err(Error::<T>::EvenementNotFound)?
			}

			let beschikbaarheid = Self::get_beschikbaarheid(evenement.id);

			if beschikbaarheid < 1 {
				Err(Error::<T>::NoTicketsAvailable)?
			}

			let ticket = Ticket { is_gescand };

			<Tickets<T>>::insert((evenement_id, klant_id), ticket.clone());

			Self::deposit_event(Event::TicketAangemaakt(ticket));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn scan_ticket(
			origin: OriginFor<T>,
			evenement_id: EvenementId,
			klant_id: KlantId,
		) -> DispatchResult {
			ensure_signed(origin)?;

			if <Tickets<T>>::contains_key((evenement_id, klant_id)) == false {
				Err(Error::<T>::TicketNotFound)?
			}

			let mut ticket = <Tickets<T>>::get((evenement_id, klant_id));

			if ticket.is_gescand == true {
				Err(Error::<T>::TicketAlreadyScanned)?
			}

			ticket.is_gescand = true;

			<Tickets<T>>::insert((evenement_id, klant_id), &ticket);

			Self::deposit_event(Event::Ticket(ticket));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn sell_ticket(
			origin: OriginFor<T>,
			evenement_id: EvenementId,
			klant_oud: KlantId,
			klant_nieuw: KlantId,
			prijs: Prijs,
		) -> DispatchResult {
			ensure_signed(origin)?;

			if <Evenementen<T>>::contains_key(evenement_id) == false {
				Err(Error::<T>::EvenementNotFound)?
			}

			if <Tickets<T>>::contains_key((evenement_id, klant_oud)) == false {
				Err(Error::<T>::TicketNotFound)?
			}

			let ticket = <Tickets<T>>::get((evenement_id, klant_oud));

			if ticket.is_gescand == true {
				Err(Error::<T>::TicketAlreadyScanned)?
			}

			let evenement = <Evenementen<T>>::get(evenement_id);

			if evenement.max_prijs < prijs {
				Err(Error::<T>::MaxPriceExceeded)?
			}

			<Tickets<T>>::swap(
				(evenement_id, klant_oud), 
				(evenement_id, klant_nieuw)
			);

			let ticket = <Tickets<T>>::get((evenement_id, klant_nieuw));
			Self::deposit_event(Event::Ticket(ticket));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_beschikbaarheid(id: EvenementId) -> u128 {
			let evenement = <Evenementen<T>>::get(id);

			let mut count = 0;
			for ticket in <Tickets<T>>::iter_keys() {
				if ticket.0 == evenement.id {
					count += 1;
				}
			}

			let result = evenement.aantal_tickets - count;
			return result;
		}
	}
}
