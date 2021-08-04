/*
编译标签：编译的时候使用 std 或者 no_std 两种形式，如果不是 std 的编译标签的话就一定是 no_std
 */
#![cfg_attr(not(feature = "std"), no_std)]

/// A module for proof of existence

/// 暴露组件，从而让 runtime 引用
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	
	/// 可调用函数的返回结果：DispatchResultWithPostInfo
	/// runtime 开发所需要的宏：pallet_prelude::*
	/// frame_system::pallet_prelude::* ：系统以来的信息

	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*
	};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;


	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn proofs)]
	pub type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		(T::AccountId, T::BlockNumber),
	>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated(T::AccountId, Vec<u8>),
		ClaimRevoked(T::AccountId, Vec<u8>),
		TransferClaim(T::AccountId, T::AccountId, Vec<u8>),
	}

	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyExist,
		ClaimNotExist,
		NotClaimOwner,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub(super) fn create_claim(
			origin: OriginFor<T>,
			claim: Vec<u8>,
		) -> DispatchResultWithPostInfo {

			let sender = ensure_signed(origin)?;

			ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExist);

			let current_block = <frame_system::Module<T>>::block_number();

			Proofs::<T>::insert(&claim, (&sender, current_block));

			Self::deposit_event(Event::ClaimCreated(sender, claim));

			Ok(().into())
		}

		#[pallet::weight(0)]
		fn revoke_claim(
			origin: OriginFor<T>,
			claim: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

			ensure!(sender == owner, Error::<T>::NotClaimOwner);

			Proofs::<T>::remove(&claim);

			Self::deposit_event(Event::ClaimRevoked(sender, claim));

			Ok(().into())
		}

		#[pallet::weight(0)]
		fn transfer_claim(
			origin: OriginFor<T>,
			receiver: T::AccountId,
			claim: Vec<u8>,
		) -> DispatchResultWithPostInfo {

			let sender = ensure_signed(origin)?;
			let (owner, block_num) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

			ensure!(sender == owner, Error::<T>::NotClaimOwner);

			Proofs::<T>::insert(&claim, (receiver.clone(), block_num));

			Self::deposit_event(Event::TransferClaim(sender, receiver, claim));

			Ok(().into())
		}
	}
}
