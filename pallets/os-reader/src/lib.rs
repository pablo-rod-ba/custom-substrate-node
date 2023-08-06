#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
use sp_os_reader::{InherentError, InherentType, INHERENT_IDENTIFIER};
pub use sp_std::result;
use sp_std::{ops::Deref, vec::Vec};
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		log::info,
		pallet_prelude::{ValueQuery, *},
	};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type MaxOsValueLenght: Get<u32>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type OsValue<T: Config> = StorageValue<_, BoundedVec<u8, T::MaxOsValueLenght>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CurrentOsValue { os_value: Vec<u8> },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight((
			T::WeightInfo::set_os_value(),
			DispatchClass::Mandatory
		))]
		pub fn set_os_value(origin: OriginFor<T>, os_value: Vec<u8>) -> DispatchResult {
			ensure_none(origin)?;

			let bounded_os_value: BoundedVec<_, T::MaxOsValueLenght> =
				BoundedVec::try_from(os_value.clone()).unwrap();

			// Update storage.
			<OsValue<T>>::put(bounded_os_value);

			// Emit an event.
			Self::deposit_event(Event::CurrentOsValue { os_value });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}

	#[pallet::inherent]
	impl<T: Config> ProvideInherent for Pallet<T> {
		type Call = Call<T>;
		type Error = InherentError;
		const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

		fn create_inherent(data: &InherentData) -> Option<Self::Call> {
			info!("pallet ProvideInherent create_inherent called");
			let inherent_data = data
				.get_data::<InherentType>(&INHERENT_IDENTIFIER)
				.expect("Os Reader inherent data not correctly encoded")
				.expect("Os Reader inherent data must be provided");
			let data = inherent_data.deref();
			info!("pallet ProvideInherent create_inherent: {:?}",data);
			Some(Call::set_os_value { os_value: data.to_vec() })
		}

		fn check_inherent(
			_call: &Self::Call,
			data: &InherentData,
		) -> result::Result<(), Self::Error> {
			let _inherent_data = data
				.get_data::<InherentType>(&INHERENT_IDENTIFIER)
				.expect("Os Reader inherent data not correctly encoded")
				.expect("Os Reader inherent data must be provided");
			Ok(())
		}

		fn is_inherent(call: &Self::Call) -> bool {
			matches!(call, Call::set_os_value { .. })
		}
	}
}
