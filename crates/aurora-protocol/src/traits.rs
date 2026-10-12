pub use aurora_derive::{FromBytes, ToBytes, bytes};

pub trait ToBytes {
    const SIZE: usize;

    type Output: AsRef<[u8]>;

    fn to_bytes(&self) -> Self::Output;
}

pub trait FromBytes: Sized {
    const SIZE: usize;

    fn from_bytes(raw: &[u8]) -> Option<Self>;
}
