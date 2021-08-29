#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_io::hashing::blake2_128;

use frame_support::traits::{BalanceStatus::Reserved, Currency, Randomness, ReservableCurrency};

use sp_runtime::{
	traits::{
		AtLeast32BitUnsigned, Bounded, CheckedAdd, CheckedSub, Saturating, StaticLookup, Zero,
	},
};

pub use pallet::*;

//  #[cfg(test)]
//  mod mock;

//  #[cfg(test)]
//  mod tests;

type BalanceOf<T> =
	<<T as Config<>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[derive(Encode, Decode)]
	pub struct Kitty(pub [u8; 16]);

	type KittyIndex = u32;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The units in which we record balances.
		// type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
		type Currency: ReservableCurrency<Self::AccountId>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		#[pallet::constant]
		type CreateKittyReserve: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreate(T::AccountId, KittyIndex),
		KittyTransfer(T::AccountId, KittyIndex, T::AccountId),
		KittyBuy(T::AccountId, KittyIndex, T::AccountId),
		KittySetPrice(T::AccountId, KittyIndex, BalanceOf<T>),
	}

	#[pallet::storage]
	#[pallet::getter(fn kitties_count)]
	pub type KittiesCount<T: Config> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, KittyIndex, Option<Kitty>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type Owner<T: Config> =
		StorageMap<_, Blake2_128Concat, KittyIndex, Option<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn price_of)]
	pub type PriceOf<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		KittyIndex,
		Option<BalanceOf<T>>,
		ValueQuery
	>;

	#[pallet::error]
	pub enum Error<T> {
		KittiesCountOverflow,
		NotKittyOwner,
		SameParentIndex,
		InvalidKittyIndex,
		ReserveFailed,
		InvalidKittyPrice,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			T::Currency::reserve(
				&who,
				T::CreateKittyReserve::get()
			).map_err(|_| {
				Error::<T>::ReserveFailed
			})?;

			let kitty_id = match Self::kitties_count() {
				Some(id) => {
					ensure!(id != KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					id
				}
				None => 0,
			};

			let dna = Self::random_value(&who);

			Kitties::<T>::insert(kitty_id, Some(Kitty(dna)));

			Owner::<T>::insert(kitty_id, Some(who.clone()));

			KittiesCount::<T>::put(kitty_id + 1);

			Self::deposit_event(Event::KittyCreate(who, kitty_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			kitty_id: KittyIndex,
			new_owner_id: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotKittyOwner);

			Owner::<T>::insert(kitty_id, Some(new_owner_id.clone()));

			Self::deposit_event(Event::KittyTransfer(who, kitty_id, new_owner_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: KittyIndex,
			kitty_id_2: KittyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);
			let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
			let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;

			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id_1), Error::<T>::NotKittyOwner);
			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id_2), Error::<T>::NotKittyOwner);

			let child_id = match Self::kitties_count() {
				Some(id) => {
					ensure!(id != KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					id
				}
				None => 1,
			};

			let dna_1 = kitty1.0;
			let dna_2 = kitty2.0;

			let selector = Self::random_value(&who);
			let mut child_dna = [0u8; 16];

			for i in 0..dna_1.len() {
				child_dna[i] = (selector[i] & dna_1[i]) | (!selector[i] & dna_2[i]);
			}

			Kitties::<T>::insert(child_id, Some(Kitty(child_dna)));

			Owner::<T>::insert(child_id, Some(who.clone()));

			KittiesCount::<T>::put(child_id + 1);

			Self::deposit_event(Event::KittyCreate(who, child_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn buy(origin: OriginFor<T>, owner: T::AccountId, kitty_id: KittyIndex) -> DispatchResult {

			Ok(())
		}


		#[pallet::weight(0)]
		pub fn set_kitty_price(origin: OriginFor<T>, kitty_id: KittyIndex, kitty_price: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(
				Some(who.clone()) == Owner::<T>::get(kitty_id),
				Error::<T>::NotKittyOwner
			);
			ensure!(!kitty_price.is_zero(), Error::<T>::InvalidKittyPrice);
			PriceOf::<T>::insert(kitty_id, Some(kitty_price));

			Self::deposit_event(Event::KittySetPrice(who, kitty_id, kitty_price));

			Ok(())
		}

	}


	impl<T: Config> Pallet<T> {
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
