//! Low-level ARM Neon assembly optimizations
//!
//! This module contains SIMD-optimized operations for critical performance paths.

pub mod attacks_neon;
pub mod neon_ops;

pub use self::prelude::*;

pub mod prelude {
    pub use super::attacks_neon::*;
    pub use super::neon_ops::*;
}

pub mod lib {
    //! Re-exports for this module
    pub use super::*;
}
