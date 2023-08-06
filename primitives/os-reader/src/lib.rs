// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Substrate core types and inherents for timestamps.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
#[cfg(feature = "std")]
pub use log;
use sp_inherents::{InherentData, InherentIdentifier, IsFatalError};
use sp_std::vec::Vec;
#[cfg(feature = "std")]
use std::process::Command;

/// The identifier for the `timestamp` inherent.
pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"osreader";

/// The type of the inherent.
pub type InherentType = OsReader;

/// Unit type wrapper that represents a timestamp.
///
/// Such a timestamp is the time since the UNIX_EPOCH in milliseconds at a given point in time.
#[derive(sp_runtime::RuntimeDebug, Encode, Decode, Clone, Default)]
pub struct OsReader(Vec<u8>);

impl OsReader {
	/// Create new `Self`.
	pub const fn new(inner: Vec<u8>) -> Self {
		Self(inner)
	}

	#[cfg(feature = "std")]
	pub fn current_value() -> Vec<u8> {
		let output = Command::new("cat")
			.arg("os_value.txt")
			.output()
			.expect("failed to execute command");

		let output = String::from_utf8_lossy(&output.stdout);
		let output = output.trim();
		log::info!("OsReader current_value output: {:?}", output);
		output.into()
	}
}

impl sp_std::ops::Deref for OsReader {
	type Target = Vec<u8>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[cfg(feature = "std")]
impl std::fmt::Display for OsReader {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.0)
	}
}

impl From<Vec<u8>> for OsReader {
	fn from(osreader: Vec<u8>) -> Self {
		OsReader(osreader)
	}
}

impl From<OsReader> for Vec<u8> {
	fn from(osreader: OsReader) -> Vec<u8> {
		osreader.into()
	}
}

/// Errors that can occur while checking the timestamp inherent.
#[derive(Encode, sp_runtime::RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Decode, thiserror::Error))]
pub enum InherentError {
	/// The time between the blocks is too short.
	#[cfg_attr(
		feature = "std",
		error("The time since the last timestamp is lower than the minimum period.")
	)]
	TooEarly,
	/// The block timestamp is too far in the future
	#[cfg_attr(feature = "std", error("The timestamp of the block is too far in the future."))]
	TooFarInFuture,
}

impl IsFatalError for InherentError {
	fn is_fatal_error(&self) -> bool {
		match self {
			InherentError::TooEarly => true,
			InherentError::TooFarInFuture => true,
		}
	}
}

impl InherentError {
	/// Try to create an instance ouf of the given identifier and data.
	#[cfg(feature = "std")]
	pub fn try_from(id: &InherentIdentifier, mut data: &[u8]) -> Option<Self> {
		if id == &INHERENT_IDENTIFIER {
			<InherentError as codec::Decode>::decode(&mut data).ok()
		} else {
			None
		}
	}
}

/// Auxiliary trait to extract timestamp inherent data.
pub trait OsReaderInherentData {
	/// Get timestamp inherent data.
	fn osreader_inherent_data(&self) -> Result<Option<InherentType>, sp_inherents::Error>;
}

impl OsReaderInherentData for InherentData {
	fn osreader_inherent_data(&self) -> Result<Option<InherentType>, sp_inherents::Error> {
		self.get_data(&INHERENT_IDENTIFIER)
	}
}

#[derive(Debug)]
/// Provide duration since unix epoch in millisecond for timestamp inherent.
#[cfg(feature = "std")]
pub struct InherentDataProvider {
	os_value: InherentType,
}

#[cfg(feature = "std")]
impl InherentDataProvider {
	/// Create `Self` while using the system time to get the timestamp.
	pub fn from_current_os_value() -> Self {
		log::info!("from_current_os_value called");
		let os_value = OsReader::current_value();
		println!("os_value: {:?}", os_value);
		log::info!("os_value: {:?}", os_value);

		Self { os_value: os_value.into() }
	}

	/// Create `Self` using the given `timestamp`.
	pub fn new() -> Self {
		let os_value = OsReader::current_value();
		println!("os_value: {:?}", os_value);
		Self { os_value: os_value.into() }
	}

	/// Returns the timestamp of this inherent data provider.
	pub fn os_value(&self) -> InherentType {
		self.os_value.clone()
	}
}

#[cfg(feature = "std")]
impl sp_std::ops::Deref for InherentDataProvider {
	type Target = InherentType;

	fn deref(&self) -> &Self::Target {
		&self.os_value
	}
}

#[cfg(feature = "std")]
#[async_trait::async_trait]
impl sp_inherents::InherentDataProvider for InherentDataProvider {
	async fn provide_inherent_data(
		&self,
		inherent_data: &mut InherentData,
	) -> Result<(), sp_inherents::Error> {
		inherent_data.put_data(INHERENT_IDENTIFIER, &self.os_value)
	}

	async fn try_handle_error(
		&self,
		identifier: &InherentIdentifier,
		error: &[u8],
	) -> Option<Result<(), sp_inherents::Error>> {
		Some(Err(sp_inherents::Error::Application(Box::from(InherentError::try_from(
			identifier, error,
		)?))))
	}
}
