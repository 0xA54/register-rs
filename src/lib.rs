#![no_std]
#![feature(generic_const_exprs)]

// use std::convert::Infallible;
use core::convert::Infallible;

pub use register_rs_derive::*;
pub use intbits::*;

pub trait Register<W> {
    /// Register address
    const ADDRESS: W;
    /// Number of bytes to read/ write
    const LENGTH: usize;

    /// Value on reset
    /// 
    /// The fields named as `reserved` must not be over-ridden by the user,
    /// otherwise behavior is not guaranteed. Use `RadioRegister::reset_value()` when
    /// instantiating registers with these fields.
    fn reset_value() -> Self;
}

/// Register is readable
pub trait ReadableRegister<W, T: Register<W> = Self>: Register<W> + Sized {
    /// Convert buffer into type.
    /// Panics if buffer length is incorrect
    fn from_bytes(buffer: &[W; Self::LENGTH]) -> RegisterResult<Self>;
}

/// Register is writeable
// pub trait WriteableRegister<W, T: Register<W> = Self>: Register<W> {
    pub trait WriteableRegister<W, T: Register<W> = Self>: Register<W> + Sized {
    fn into_bytes(&self) -> RegisterResult<[W; Self::LENGTH]>;
}

/// Alias for `Result<T, RegisterError>`
pub type RegisterResult<T> = Result<T, RegisterError>;


/// Possible error types 
#[derive(Debug, PartialEq)]
pub enum RegisterError {
    /// Failed to perform conversion from bits into an enum variant
    ConversionError,
    /// The provided configuration is invalid. Did you try setting an invalid state?
    InvalidConfiguration,
}

impl From<Infallible> for RegisterError {
    fn from(value: Infallible) -> Self {
        RegisterError::ConversionError
        // unreachable!() // To risk it or not to risk it
    }
}