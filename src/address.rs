use hex::FromHexError;
use std::{
    fmt::{self, Debug, Display, Formatter},
    str::{self, FromStr},
};
use tiny_keccak::{Hasher as _, Keccak};

/// An Ethereum public address.
#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub struct Address(pub [u8; 20]);

impl Address {
    /// Returns the zero address.
    pub const fn zero() -> Self {
        Self([0; 20])
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

        let digest = {
            let mut output = [0_u8; 32];
            let mut hasher = Keccak::v256();
            hasher.update(addr);
            hasher.finalize(&mut output);
            output
        };

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

macro_rules! address {
    ($s:literal) => {
        $crate::address::Address(::hex_literal::hex!($s))
    };
}

pub(crate) use address;

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
