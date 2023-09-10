#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(feature = "async")]
mod async_lock;
#[cfg(feature = "async")]
mod async_locking;

mod error;
mod lock;
mod locking;
mod types;

pub use self::{error::*, lock::Locket, locking::*, types::*};

#[cfg(feature = "async")]
pub use self::async_lock::*;
#[cfg(feature = "async")]
pub use async_locking::*;

#[cfg(feature = "parking_lot")]
pub use parking_lot;

#[cfg(feature = "spin")]
pub use spin;
