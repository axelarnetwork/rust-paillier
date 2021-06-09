use crate::BigInt;
use rand::{CryptoRng, RngCore};

pub fn sample(bit_size: usize) -> BigInt {
    if bit_size == 0 {
        return BigInt::zero();
    }
    let bytes = (bit_size - 1) / 8 + 1;
    let mut buf: Vec<u8> = vec![0; bytes];
    rand::thread_rng().fill_bytes(&mut buf);
    BigInt::from(&*buf) >> (bytes * 8 - bit_size)
}

pub fn sample_with_rng(mut rng: impl CryptoRng + RngCore, bit_size: usize) -> BigInt {
    if bit_size == 0 {
        return BigInt::zero();
    }
    let bytes = (bit_size - 1) / 8 + 1;
    let mut buf: Vec<u8> = vec![0; bytes];
    rng.fill_bytes(&mut buf);
    BigInt::from(&*buf) >> (bytes * 8 - bit_size)
}

pub fn sample_below(upper: &BigInt) -> BigInt {
    assert!(*upper > BigInt::zero());

    let bits = upper.bit_length();
    loop {
        let n = sample(bits);
        if n < *upper {
            return n;
        }
    }
}

pub fn sample_range(lower: &BigInt, upper: &BigInt) -> BigInt {
    assert!(upper > lower);
    lower + sample_below(&(upper - lower))
}

pub fn is_even(bigint: &BigInt) -> bool {
    bigint.is_multiple_of(&BigInt::from(2))
}
pub fn set_bit(bigint: &mut BigInt, bit: usize, bit_val: bool) {
    if bit_val {
        bigint.setbit(bit);
    } else {
        bigint.clrbit(bit);
    }
}

pub fn mod_sub(a: &BigInt, b: &BigInt, modulus: &BigInt) -> BigInt {
    // let a_m = a.gmp.mod_floor(&modulus.gmp);
    // let b_m = b.gmp.mod_floor(&modulus.gmp);

    // let sub_op = a_m - b_m + &modulus.gmp;
    // sub_op.mod_floor(&modulus.gmp).wrap()
    (a.modulus(modulus) - b.modulus(modulus)).modulus(modulus)
}

pub fn mod_add(a: &BigInt, b: &BigInt, modulus: &BigInt) -> BigInt {
    (a.modulus(modulus) + b.modulus(modulus)).modulus(modulus)
}

pub fn mod_mul(x: &BigInt, y: &BigInt, n: &BigInt) -> BigInt {
    (x.modulus(n) * y.modulus(n)).modulus(n)
}

pub fn try_from<T>(n: &BigInt) -> T
where
    Option<T>: for<'a> From<&'a BigInt>,
{
    Option::<T>::from(n).unwrap_or_else(|| panic!("conversion from bigint failed"))
}

// impl TryFrom<&BigInt> for u64 {
//     type Error = TryFromBigIntError;

//     fn try_from(value: &BigInt) -> Result<Self, Self::Error> {
//         Option::<u64>::from(&value.gmp).ok_or(TryFromBigIntError { type_name: "u64" })
//     }
// }

// /// Error type returned when conversion from BigInt to primitive integer type (u64, i64, etc) fails
// #[derive(Debug)]
// pub struct TryFromBigIntError {
//     pub(super) type_name: &'static str,
// }

// impl fmt::Display for TryFromBigIntError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "conversion from BigInt to {} overflowed", self.type_name)
//     }
// }

// impl error::Error for TryFromBigIntError {}
