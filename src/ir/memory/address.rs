use num_traits::Num;
use num_traits::identities::{Zero, One};

use std::cmp::Ordering;
use std::fmt::{self, Display};
use std::ops::{Add, Div, Mul, Rem, Sub};
use std::str::FromStr;

use thiserror::Error;

use crate::ir::value::bv::{self, BitVec};

#[derive(Debug, Clone, Hash)]
#[repr(transparent)]
pub struct Address(BitVec);

impl Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[derive(Debug, Error)]
pub enum AddressParseError {
    #[error(transparent)]
    Parse(#[from] bv::error::ParseError),
    #[error("cannot parse address from string with radix {0}")]
    Radix(u32),
    #[error("addresses cannot be negative")]
    Sign,
    #[error("addresses cannot be zero-sized")]
    ZeroSize,
}

impl FromStr for Address {
    type Err = AddressParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bv = s.parse::<BitVec>()?;
        if bv.is_signed() {
            Err(AddressParseError::Sign)
        } else if bv.bits() == 0 {
            Err(AddressParseError::ZeroSize)
        } else {
            Ok(bv.into())
        }
    }
}

impl From<Address> for BitVec {
    fn from(addr: Address) -> Self {
        addr.0
    }
}

impl From<BitVec> for Address {
    fn from(bv: BitVec) -> Self {
        if bv.bits() == 0 {
            panic!("addresses cannot be zero-sized")
        }

        Self(bv.unsigned())
    }
}

impl From<u8> for Address {
    fn from(value: u8) -> Self {
        Self(BitVec::from(value))
    }
}

impl From<u16> for Address {
    fn from(value: u16) -> Self {
        Self(BitVec::from(value))
    }
}

impl From<u32> for Address {
    fn from(value: u32) -> Self {
        Self(BitVec::from(value))
    }
}

impl From<u64> for Address {
    fn from(value: u64) -> Self {
        Self(BitVec::from(value))
    }
}

impl From<u128> for Address {
    fn from(value: u128) -> Self {
        Self(BitVec::from(value))
    }
}

impl Zero for Address {
    fn zero() -> Address {
        Self::from(BitVec::zero(1))
    }
    
    fn set_zero(&mut self) {
        *self = Self::from(BitVec::one(self.bits() as usize));
    }
    
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl One for Address {
    fn one() -> Address {
        Self::from(BitVec::one(1))
    }
    
    fn set_one(&mut self) {
        *self = Self::from(BitVec::one(self.bits() as usize));
    }
    
    fn is_one(&self) -> bool {
        self.0.is_one()
    }
}

impl Add<Address> for Address {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Self::from(match lbits.cmp(&rbits) {
            Ordering::Equal => self.0.add(rhs.0),
            Ordering::Less => self.0.cast(rbits as usize).add(rhs.0),
            Ordering::Greater => self.0.add(rhs.0.cast(lbits as usize))
        })
    }
}

impl Add<&Address> for Address {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Self::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 + &rhs.0,
            Ordering::Less => &self.0.cast(rbits as usize) + &rhs.0,
            Ordering::Greater => &self.0 + &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Add<usize> for Address {
    type Output = Self;

    fn add(self, rhs: usize) -> Self {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        self.0.add(rhs_bv).into()
    }
}

impl Add<Address> for &Address {
    type Output = Address;

    fn add(self, rhs: Address) -> Address {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Address::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 + &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) + &rhs.0,
            Ordering::Greater => &self.0 + &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Add<&Address> for &Address {
    type Output = Address;

    fn add(self, rhs: &Address) -> Address {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Address::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 + &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) + &rhs.0,
            Ordering::Greater => &self.0 + &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Add<usize> for &Address {
    type Output = Address;

    fn add(self, rhs: usize) -> Address {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        (&self.0 + &rhs_bv).into()
    }
}

impl Div<Address> for Address {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Self::from(match lbits.cmp(&rbits) {
            Ordering::Equal => self.0.div(rhs.0),
            Ordering::Less => self.0.cast(rbits as usize).div(rhs.0),
            Ordering::Greater => self.0.div(rhs.0.cast(lbits as usize))
        })
    }
}

impl Div<&Address> for Address {
    type Output = Self;

    fn div(self, rhs: &Self) -> Self {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Self::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 / &rhs.0,
            Ordering::Less => &self.0.cast(rbits as usize) / &rhs.0,
            Ordering::Greater => &self.0 / &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Div<usize> for Address {
    type Output = Self;

    fn div(self, rhs: usize) -> Self {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        self.0.div(rhs_bv).into()
    }
}

impl Div<Address> for &Address {
    type Output = Address;

    fn div(self, rhs: Address) -> Address {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Address::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 / &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) / &rhs.0,
            Ordering::Greater => &self.0 / &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Div<&Address> for &Address {
    type Output = Address;

    fn div(self, rhs: &Address) -> Address {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Address::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 / &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) / &rhs.0,
            Ordering::Greater => &self.0 / &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Div<usize> for &Address {
    type Output = Address;

    fn div(self, rhs: usize) -> Address {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        (&self.0 / &rhs_bv).into()
    }
}

impl Mul<Address> for Address {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Self::from(match lbits.cmp(&rbits) {
            Ordering::Equal => self.0.mul(rhs.0),
            Ordering::Less => self.0.cast(rbits as usize).mul(rhs.0),
            Ordering::Greater => self.0.mul(rhs.0.cast(lbits as usize))
        })
    }
}

impl Mul<&Address> for Address {
    type Output = Self;

    fn mul(self, rhs: &Self) -> Self {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Self::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 * &rhs.0,
            Ordering::Less => &self.0.cast(rbits as usize) * &rhs.0,
            Ordering::Greater => &self.0 * &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Mul<usize> for Address {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        self.0.rem(rhs_bv).into()
    }
}

impl Mul<Address> for &Address {
    type Output = Address;

    fn mul(self, rhs: Address) -> Address {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Address::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 * &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) * &rhs.0,
            Ordering::Greater => &self.0 * &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Mul<&Address> for &Address {
    type Output = Address;

    fn mul(self, rhs: &Address) -> Address {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Address::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 * &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) * &rhs.0,
            Ordering::Greater => &self.0 * &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Mul<usize> for &Address {
    type Output = Address;

    fn mul(self, rhs: usize) -> Address {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        (&self.0 * &rhs_bv).into()
    }
}

impl Rem<Address> for Address {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Self::from(match lbits.cmp(&rbits) {
            Ordering::Equal => self.0.rem(rhs.0),
            Ordering::Less => self.0.cast(rbits as usize).rem(rhs.0),
            Ordering::Greater => self.0.rem(rhs.0.cast(lbits as usize))
        })
    }
}

impl Rem<&Address> for Address {
    type Output = Self;

    fn rem(self, rhs: &Self) -> Self {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Self::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 % &rhs.0,
            Ordering::Less => &self.0.cast(rbits as usize) % &rhs.0,
            Ordering::Greater => &self.0 % &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Rem<usize> for Address {
    type Output = Self;

    fn rem(self, rhs: usize) -> Self {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        self.0.rem(rhs_bv).into()
    }
}

impl Rem<Address> for &Address {
    type Output = Address;

    fn rem(self, rhs: Address) -> Address {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Address::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 % &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) % &rhs.0,
            Ordering::Greater => &self.0 % &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Rem<&Address> for &Address {
    type Output = Address;

    fn rem(self, rhs: &Address) -> Address {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Address::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 % &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) % &rhs.0,
            Ordering::Greater => &self.0 % &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Rem<usize> for &Address {
    type Output = Address;

    fn rem(self, rhs: usize) -> Address {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        (&self.0 % &rhs_bv).into()
    }
}

impl Sub<Address> for Address {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Self::from(match lbits.cmp(&rbits) {
            Ordering::Equal => self.0.sub(rhs.0),
            Ordering::Less => self.0.cast(rbits as usize).sub(rhs.0),
            Ordering::Greater => self.0.sub(rhs.0.cast(lbits as usize))
        })
    }
}

impl Sub<&Address> for Address {
    type Output = Self;

    fn sub(self, rhs: &Self) -> Self {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Self::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 - &rhs.0,
            Ordering::Less => &self.0.cast(rbits as usize) - &rhs.0,
            Ordering::Greater => &self.0 - &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Sub<usize> for Address {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        self.0.sub(rhs_bv).into()
    }
}

impl Sub<Address> for &Address {
    type Output = Address;

    fn sub(self, rhs: Address) -> Address {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Address::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 - &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) - &rhs.0,
            Ordering::Greater => &self.0 - &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Sub<&Address> for &Address {
    type Output = Address;

    fn sub(self, rhs: &Address) -> Address {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Address::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 - &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) - &rhs.0,
            Ordering::Greater => &self.0 - &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Sub<usize> for &Address {
    type Output = Address;

    fn sub(self, rhs: usize) -> Address {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        (&self.0 - &rhs_bv).into()
    }
}

impl PartialEq<Address> for Address {
    fn eq(&self, rhs: &Self) -> bool {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 == &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) == &rhs.0,
            Ordering::Greater => &self.0 == &rhs.0.unsigned_cast(lbits as usize),
        }
    }
}
impl Eq for Address { }

impl PartialOrd<Address> for Address {
    fn partial_cmp(&self, other: &Address) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Address {
    fn cmp(&self, rhs: &Self) -> Ordering {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        match lbits.cmp(&rbits) {
            Ordering::Equal => self.0.cmp(&rhs.0),
            Ordering::Less => self.0.unsigned_cast(rbits as usize).cmp(&rhs.0),
            Ordering::Greater => self.0.cmp(&rhs.0.unsigned_cast(lbits as usize)),
        }
    }
}

impl Num for Address {
    type FromStrRadixErr = AddressParseError;
    
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        let bv = BitVec::from_str_radix(s, radix)?;
        if bv.is_signed() {
            Err(AddressParseError::Sign)
        } else if bv.bits() == 0 {
            Err(AddressParseError::ZeroSize)
        } else {
            Ok(bv.into())
        }
    }
}

impl Address {
    pub fn as_bits(&self, bits: u32) -> Self {
        self.0.unsigned_cast(bits as usize).into()
    }
    
    pub fn into_bits(self, bits: u32) -> Self {
        self.0.cast(bits as usize).into()
    }
    
    pub fn absolute_difference(&self, other: &Address) -> Option<usize> {
        if self >= other {
            BitVec::from(self - other).to_usize()
        } else {
            BitVec::from(other - self).to_usize()
        }
    }

    pub fn bits(&self) -> u32 {
        self.0.bits() as u32
    }
}