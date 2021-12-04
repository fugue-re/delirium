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
pub struct Addr(BitVec);

impl Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[derive(Debug, Error)]
pub enum AddrParseError {
    #[error(transparent)]
    Parse(#[from] bv::error::ParseError),
    #[error("cannot parse address from string with radix {0}")]
    Radix(u32),
    #[error("addresses cannot be negative")]
    Sign,
    #[error("addresses cannot be zero-sized")]
    ZeroSize,
}

impl FromStr for Addr {
    type Err = AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bv = s.parse::<BitVec>()?;
        if bv.is_signed() {
            Err(AddrParseError::Sign)
        } else if bv.bits() == 0 {
            Err(AddrParseError::ZeroSize)
        } else {
            Ok(bv.into())
        }
    }
}

impl From<Addr> for BitVec {
    fn from(addr: Addr) -> Self {
        addr.0
    }
}

impl From<BitVec> for Addr {
    fn from(bv: BitVec) -> Self {
        if bv.bits() == 0 {
            panic!("addresses cannot be zero-sized")
        }

        Self(bv.unsigned())
    }
}

impl From<u8> for Addr {
    fn from(value: u8) -> Self {
        Self(BitVec::from(value))
    }
}

impl From<u16> for Addr {
    fn from(value: u16) -> Self {
        Self(BitVec::from(value))
    }
}

impl From<u32> for Addr {
    fn from(value: u32) -> Self {
        Self(BitVec::from(value))
    }
}

impl From<u64> for Addr {
    fn from(value: u64) -> Self {
        Self(BitVec::from(value))
    }
}

impl From<u128> for Addr {
    fn from(value: u128) -> Self {
        Self(BitVec::from(value))
    }
}

impl Zero for Addr {
    fn zero() -> Addr {
        Self::from(BitVec::zero(1))
    }
    
    fn set_zero(&mut self) {
        *self = Self::from(BitVec::one(self.bits() as usize));
    }
    
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl One for Addr {
    fn one() -> Addr {
        Self::from(BitVec::one(1))
    }
    
    fn set_one(&mut self) {
        *self = Self::from(BitVec::one(self.bits() as usize));
    }
    
    fn is_one(&self) -> bool {
        self.0.is_one()
    }
}

impl Add<Addr> for Addr {
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

impl Add<&Addr> for Addr {
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

impl Add<usize> for Addr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        self.0.add(rhs_bv).into()
    }
}

impl Add<Addr> for &Addr {
    type Output = Addr;

    fn add(self, rhs: Addr) -> Addr {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Addr::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 + &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) + &rhs.0,
            Ordering::Greater => &self.0 + &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Add<&Addr> for &Addr {
    type Output = Addr;

    fn add(self, rhs: &Addr) -> Addr {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Addr::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 + &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) + &rhs.0,
            Ordering::Greater => &self.0 + &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Add<usize> for &Addr {
    type Output = Addr;

    fn add(self, rhs: usize) -> Addr {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        (&self.0 + &rhs_bv).into()
    }
}

impl Div<Addr> for Addr {
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

impl Div<&Addr> for Addr {
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

impl Div<usize> for Addr {
    type Output = Self;

    fn div(self, rhs: usize) -> Self {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        self.0.div(rhs_bv).into()
    }
}

impl Div<Addr> for &Addr {
    type Output = Addr;

    fn div(self, rhs: Addr) -> Addr {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Addr::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 / &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) / &rhs.0,
            Ordering::Greater => &self.0 / &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Div<&Addr> for &Addr {
    type Output = Addr;

    fn div(self, rhs: &Addr) -> Addr {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Addr::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 / &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) / &rhs.0,
            Ordering::Greater => &self.0 / &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Div<usize> for &Addr {
    type Output = Addr;

    fn div(self, rhs: usize) -> Addr {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        (&self.0 / &rhs_bv).into()
    }
}

impl Mul<Addr> for Addr {
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

impl Mul<&Addr> for Addr {
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

impl Mul<usize> for Addr {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        self.0.rem(rhs_bv).into()
    }
}

impl Mul<Addr> for &Addr {
    type Output = Addr;

    fn mul(self, rhs: Addr) -> Addr {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Addr::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 * &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) * &rhs.0,
            Ordering::Greater => &self.0 * &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Mul<&Addr> for &Addr {
    type Output = Addr;

    fn mul(self, rhs: &Addr) -> Addr {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Addr::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 * &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) * &rhs.0,
            Ordering::Greater => &self.0 * &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Mul<usize> for &Addr {
    type Output = Addr;

    fn mul(self, rhs: usize) -> Addr {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        (&self.0 * &rhs_bv).into()
    }
}

impl Rem<Addr> for Addr {
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

impl Rem<&Addr> for Addr {
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

impl Rem<usize> for Addr {
    type Output = Self;

    fn rem(self, rhs: usize) -> Self {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        self.0.rem(rhs_bv).into()
    }
}

impl Rem<Addr> for &Addr {
    type Output = Addr;

    fn rem(self, rhs: Addr) -> Addr {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Addr::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 % &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) % &rhs.0,
            Ordering::Greater => &self.0 % &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Rem<&Addr> for &Addr {
    type Output = Addr;

    fn rem(self, rhs: &Addr) -> Addr {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Addr::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 % &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) % &rhs.0,
            Ordering::Greater => &self.0 % &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Rem<usize> for &Addr {
    type Output = Addr;

    fn rem(self, rhs: usize) -> Addr {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        (&self.0 % &rhs_bv).into()
    }
}

impl Sub<Addr> for Addr {
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

impl Sub<&Addr> for Addr {
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

impl Sub<usize> for Addr {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        self.0.sub(rhs_bv).into()
    }
}

impl Sub<Addr> for &Addr {
    type Output = Addr;

    fn sub(self, rhs: Addr) -> Addr {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Addr::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 - &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) - &rhs.0,
            Ordering::Greater => &self.0 - &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Sub<&Addr> for &Addr {
    type Output = Addr;

    fn sub(self, rhs: &Addr) -> Addr {
        let lbits = self.bits();
        let rbits = rhs.bits();
        
        Addr::from(match lbits.cmp(&rbits) {
            Ordering::Equal => &self.0 - &rhs.0,
            Ordering::Less => &self.0.unsigned_cast(rbits as usize) - &rhs.0,
            Ordering::Greater => &self.0 - &rhs.0.unsigned_cast(lbits as usize),
        })
    }
}

impl Sub<usize> for &Addr {
    type Output = Addr;

    fn sub(self, rhs: usize) -> Addr {
        let lbits = self.bits();
        let rhs_bv = BitVec::from_usize(rhs, lbits as usize);
        
        (&self.0 - &rhs_bv).into()
    }
}

impl PartialEq<Addr> for Addr {
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
impl Eq for Addr { }

impl PartialOrd<Addr> for Addr {
    fn partial_cmp(&self, other: &Addr) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Addr {
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

impl Num for Addr {
    type FromStrRadixErr = AddrParseError;
    
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        let bv = BitVec::from_str_radix(s, radix)?;
        if bv.is_signed() {
            Err(AddrParseError::Sign)
        } else if bv.bits() == 0 {
            Err(AddrParseError::ZeroSize)
        } else {
            Ok(bv.into())
        }
    }
}

impl Addr {
    pub fn as_bits(&self, bits: u32) -> Self {
        self.0.unsigned_cast(bits as usize).into()
    }
    
    pub fn into_bits(self, bits: u32) -> Self {
        self.0.cast(bits as usize).into()
    }
    
    pub fn absolute_difference(&self, other: &Addr) -> Option<usize> {
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