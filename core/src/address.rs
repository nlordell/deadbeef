use hex::FromHexError;
use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
    str::{self, FromStr},
};

use crate::keccak;

/// An Ethereum public address.
#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub struct Address(pub [u8; 20]);

impl Address {
    /// Returns the zero address.
    pub const fn zero() -> Self {
        Self([0; 20])
    }

    /// Returns `Some(self)` if the address is non-zero.
    pub fn non_zero(self) -> Option<NonZeroAddress> {
        if self == Self::zero() {
            None
        } else {
            Some(NonZeroAddress(self))
        }
    }
}

impl Debug for Address {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        struct DisplayAsDebug<T>(T);

        impl<T> Debug for DisplayAsDebug<T>
        where
            T: Display,
        {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        f.debug_tuple("Address")
            .field(&DisplayAsDebug(self))
            .finish()
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut buf = *b"0x0000000000000000000000000000000000000000";
        let addr = &mut buf[2..];
        hex::encode_to_slice(self.0.as_slice(), addr).expect("error decoding hex");

        let digest = keccak::v256(addr);
        for i in 0..addr.len() {
            let byte = digest[i / 2];
            let nibble = 0xf & if i % 2 == 0 { byte >> 4 } else { byte };
            if nibble >= 8 {
                addr[i] = addr[i].to_ascii_uppercase();
            }
        }

        f.write_str(unsafe { str::from_utf8_unchecked(&buf) })
    }
}

impl FromStr for Address {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut address = Self::default();
        let s = s.strip_prefix("0x").unwrap_or(s);
        hex::decode_to_slice(s, &mut address.0)?;
        Ok(address)
    }
}

/// A non-zero Ethereum public address.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct NonZeroAddress(Address);

impl NonZeroAddress {
    /// Creates a new non-zero address from the specified bytes.
    ///
    /// # Panics
    ///
    /// Panics if the address is zero.
    pub const fn from_bytes(bytes: [u8; 20]) -> Self {
        let mut i = 0;
        while i < 20 {
            if bytes[i] != 0 {
                return Self(Address(bytes));
            }
            i += 1;
        }
        panic!("invalid zero address");
    }

    /// Returns the inner address.
    pub fn get(self) -> Address {
        self.0
    }
}

impl Display for NonZeroAddress {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl FromStr for NonZeroAddress {
    type Err = NonZeroAddressParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Address::from_str(s)?
            .non_zero()
            .ok_or(NonZeroAddressParseError::Zero)
    }
}

/// An error reading a non-zero address from a string.
#[derive(Debug)]
pub enum NonZeroAddressParseError {
    /// The address is zero.
    Zero,
    /// The address is invalid.
    Invalid(FromHexError),
}

impl Display for NonZeroAddressParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Zero => f.write_str("zero address"),
            Self::Invalid(err) => Display::fmt(err, f),
        }
    }
}

impl Error for NonZeroAddressParseError {}

impl From<FromHexError> for NonZeroAddressParseError {
    fn from(err: FromHexError) -> Self {
        Self::Invalid(err)
    }
}

#[macro_export]
macro_rules! address {
    ($s:literal) => {
        $crate::Address($crate::hex!($s))
    };
    (nz $s:literal) => {
        $crate::NonZeroAddress::from_bytes($crate::hex!($s))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_address() {
        for s in &[
            "0x90F8bf6A479f320ead074411a4B0e7944Ea8c9C1",
            "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
        ] {
            let address = s.parse::<Address>().unwrap();
            assert_eq!(address.to_string(), *s);
        }
    }
}
