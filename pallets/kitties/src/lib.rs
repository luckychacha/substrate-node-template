#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, Randomness, ReservableCurrency};

use sp_runtime::traits::{
	AtLeast32BitUnsigned, Bounded, Zero,
};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use codec::{Decode, Encode};
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;

	#[derive(Encode, Decode)]
	pub struct Kitty(pub [u8; 16]);

	pub type KittyIndex = u32;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The units in which we record balances.
		// type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		#[pallet::constant]
		type CreateKittyReserve: Get<BalanceOf<Self>>;

		type KittyIndex: From<u32>
			+ Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaxEncodedLen
			+ Bounded;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreate(T::AccountId, T::KittyIndex),
		KittyTransfer(T::AccountId, T::KittyIndex, T::AccountId),
		KittyBuy(T::AccountId, T::KittyIndex, T::AccountId),
		KittySetPrice(T::AccountId, T::KittyIndex, BalanceOf<T>),
	}

	#[pallet::storage]
	#[pallet::getter(fn kitties_count)]
	pub type KittiesCount<T: Config> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<Kitty>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type Owner<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn price_of)]
	pub type PriceOf<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<BalanceOf<T>>, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		KittiesCountOverflow,
		NotKittyOwner,
		SameParentIndex,
		InvalidKittyIndex,
		ReserveFailed,
		InvalidKittyPrice,
		KittyNotForSale,
		BalanceNotEnough,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let dna = Self::random_value(&who);
			let kitty_id = Self::create_kitty(&who, dna)?;
			Self::deposit_event(Event::KittyCreate(who, kitty_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			new_owner_id: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::transfer_kitty(&who, &new_owner_id, kitty_id)?;
			Self::deposit_event(Event::KittyTransfer(who, kitty_id, new_owner_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: T::KittyIndex,
			kitty_id_2: T::KittyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);
			let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
			let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;

			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id_1), Error::<T>::NotKittyOwner);
			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id_2), Error::<T>::NotKittyOwner);

			let dna_1 = kitty1.0;
			let dna_2 = kitty2.0;

			let selector = Self::random_value(&who);
			let mut child_dna = [0u8; 16];

			for i in 0..dna_1.len() {
				child_dna[i] = (selector[i] & dna_1[i]) | (!selector[i] & dna_2[i]);
			}
			let child_id = Self::create_kitty(&who, child_dna)?;

			Self::deposit_event(Event::KittyCreate(who, child_id));

			Ok(())
		}

		/// set a price for a kitty, if the price is greater than 0,it can be bought by other people.
		#[pallet::weight(0)]
		pub fn sell(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			kitty_price: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotKittyOwner);
			ensure!(!kitty_price.is_zero(), Error::<T>::InvalidKittyPrice);
			PriceOf::<T>::insert(kitty_id, Some(kitty_price));

			Self::deposit_event(Event::KittySetPrice(who, kitty_id, kitty_price));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn buy(
			origin: OriginFor<T>,
			owner: T::AccountId,
			kitty_id: T::KittyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Some(owner.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotKittyOwner);
			let _ = Self::buy_kitty(&owner, &who, kitty_id);

			Self::deposit_event(Event::KittyBuy(who, kitty_id, owner));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn create_kitty(
			who: &T::AccountId,
			kitty_dna: [u8; 16],
		) -> Result<T::KittyIndex, DispatchError> {
			let (kitty_id, count) = Self::generate_kitty_id()?;

			T::Currency::reserve(&who, T::CreateKittyReserve::get())
				.map_err(|_| Error::<T>::ReserveFailed)?;

			Kitties::<T>::insert(kitty_id, Some(Kitty(kitty_dna)));

			Owner::<T>::insert(kitty_id, Some(who.clone()));

			KittiesCount::<T>::put(count + 1);

			Ok(kitty_id)
		}

		fn buy_kitty(
			owner_id: &T::AccountId,
			new_owner_id: &T::AccountId,
			kitty_id: T::KittyIndex,
		) -> Result<T::KittyIndex, DispatchError> {
			let price = PriceOf::<T>::get(kitty_id).ok_or(Error::<T>::KittyNotForSale)?;
			ensure!(
				(price + T::CreateKittyReserve::get()) < T::Currency::free_balance(&new_owner_id),
				Error::<T>::BalanceNotEnough
			);

			T::Currency::transfer(
				&new_owner_id,
				&owner_id,
				price,
				frame_support::traits::ExistenceRequirement::KeepAlive,
			)?;
			Self::transfer_kitty(&owner_id, &new_owner_id, kitty_id)?;

			Ok(kitty_id)
		}

		fn transfer_kitty(
			owner_id: &T::AccountId,
			new_owner_id: &T::AccountId,
			kitty_id: T::KittyIndex,
		) -> DispatchResult {
			ensure!(Some(owner_id.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotKittyOwner);
			
			T::Currency::unreserve(&owner_id, T::CreateKittyReserve::get());
			T::Currency::reserve(&new_owner_id, T::CreateKittyReserve::get())
				.map_err(|_| {
					Error::<T>::ReserveFailed
				})?;
			
			
			Owner::<T>::insert(kitty_id, Some(new_owner_id.clone()));

			// if transfer action from sell or has set price before
			// need to remove price of kitty price.
			match PriceOf::<T>::get(kitty_id.clone()) {
				Some(_) => PriceOf::<T>::remove(kitty_id.clone()),
				None => ()
			}

			Ok(())
		}

		fn generate_kitty_id() -> Result<(T::KittyIndex, u32), DispatchError> {
			let id = match Self::kitties_count() {
				Some(count) => {
					ensure!(count != KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					count
				}
				None => 0_u32,
			};

			Ok((T::KittyIndex::from(id), id))
		}

		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payloads = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			payloads.using_encoded(blake2_128)
		}
	}
}
