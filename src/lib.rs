#![no_std]
#![feature(generic_const_exprs)]
#[allow(incomplete_features)]

pub trait Register<WORD> {
    const ADDRESS: WORD;
    const LENGTH: usize;
    fn reset_value() -> Self;
}

/// Register is readable
pub trait ReadableRegister<WORD, T: Register<WORD> = Self>: Register<WORD> + Sized {
    /// Convert buffer into type.
    /// Panics if buffer length is incorrect
    fn from_bytes(buffer: &[WORD; Self::LENGTH]) -> RegisterResult<Self>;
}

/// Register is writeable
// pub trait WriteableRegister<W, T: Register<W> = Self>: Register<W> {
pub trait WriteableRegister<WORD, T: Register<WORD> = Self>: Register<WORD> + Sized {
    fn into_bytes(&self) -> RegisterResult<[WORD; Self::LENGTH]>;
}

pub enum RegisterError {}

pub type RegisterResult<T> = Result<T, RegisterError>;
