/// A region represents a mapping of some program segment. Regions
/// are indexed by addresses of a fixed size and are associated with
/// a particular endianness.
use std::borrow::{Borrow, Cow};
use std::sync::Arc;

use thiserror::Error;

use crate::ir::memory::Addr;
use crate::ir::value::bv::BitVec;

use crate::prelude::bytes::{ByteCast, Endian, BE, LE};
use crate::prelude::intervals::Interval;
use crate::prelude::{Entity, Id};

#[derive(Clone)]
pub struct Region<'r> {
    name: Arc<str>,
    range: Interval<Addr>,
    endian: Endian,
    bytes: Cow<'r, [u8]>,
}

#[derive(Debug, Error)]
pub enum RegionIOError {
    #[error("read/write byte range is unrepresentable for region `{0}`")]
    Range(Arc<str>),
    #[error("out-of-bounds read from region `{0}`")]
    OOBRead(Arc<str>),
    #[error("out-of-bounds write into region `{0}`")]
    OOBWrite(Arc<str>),
}

impl<'r> Region<'r> {
    pub fn new_with(
        id: Id<Self>,
        name: impl Into<Arc<str>>,
        addr: impl Into<Addr>,
        endian: Endian,
        bytes: impl Into<Cow<'r, [u8]>>,
    ) -> Entity<Self> {
        let address = addr.into();
        let bytes = bytes.into();
        if bytes.len() == 0 {
            // check for zero
            panic!("region size cannot be zero");
        }
        let last_address = &address + bytes.len();
        if last_address <= address {
            // check for potential overflow
            panic!(
                "address range not representable by {} bit addresses starting at {}",
                address.bits(),
                address
            );
        }

        Entity::from_parts(
            id,
            Self {
                name: name.into(),
                range: Interval::from(address..last_address),
                endian,
                bytes: bytes.into(),
            },
        )
    }

    pub fn new(
        name: impl Into<Arc<str>>,
        addr: impl Into<Addr>,
        endian: Endian,
        bytes: impl Into<Cow<'r, [u8]>>,
    ) -> Entity<Self> {
        Self::new_with(Id::new("region"), name, addr, endian, bytes)
    }
    pub fn interval(&self) -> &Interval<Addr> {
        &self.range
    }
    pub fn name(&self) -> &Arc<str> {
        &self.name
    }
    pub fn address(&self) -> &Addr {
        self.range.start()
    }
    pub fn address_size(&self) -> u32 {
        self.address().bits()
    }
    pub fn endian(&self) -> Endian {
        self.endian
    }

    pub fn bytes(&self) -> &[u8] {
        &*self.bytes
    }

    pub fn bytes_mut(&mut self) -> &mut [u8] {
        self.bytes.to_mut()
    }
    pub fn contains_range(&self, address: impl Borrow<Addr>, count: usize) -> bool {
        let address = address.borrow();
        count > 0
            && self.interval().contains_point(address)
            && self.interval().contains_point(&(address + (count - 1)))
    }

    pub fn read_bits(
        &self,
        address: impl Borrow<Addr>,
        bits: u32,
    ) -> Result<BitVec, RegionIOError> {
        let aligned = bits % 8 == 0;
        let count = bits / 8 + if aligned { 0 } else { 1 };
        let range = self.view_bytes(address, count as usize)?;
        let bv = if self.endian().is_little() {
            BitVec::from_le_bytes(range)
        } else {
            BitVec::from_be_bytes(range)
        };
        if aligned {
            Ok(bv)
        } else if self.endian().is_little() {
            // truncate msb bits
            Ok(bv.cast(bits as usize))
        } else {
            // shift out lsb bits and truncate
            Ok((bv >> (8 - (bits % 8))).cast(bits as usize))
        }
    }

    pub fn write_bits(
        &mut self,
        address: impl Borrow<Addr>,
        bv: impl Borrow<BitVec>,
    ) -> Result<(), RegionIOError> {
        let bv = bv.borrow();
        let bits = bv.bits();

        let endian = self.endian();
        let aligned = bits % 8 == 0;
        let count = bits / 8 + if aligned { 0 } else { 1 };
        let range = self.view_bytes_mut(address, count as usize)?;

        if aligned {
            if endian.is_little() {
                bv.to_le_bytes(range)
            } else {
                bv.to_be_bytes(range)
            }
        } else {
            let nbits = count * 8;
            let shift = 8 - (bits as u32 % 8);

            if endian.is_little() {
                let mask = BitVec::max_value_with(nbits, false) >> shift;
                let orig = BitVec::from_le_bytes(range) & !&mask;
                let bv = (bv.unsigned_cast(nbits) & mask) | orig;

                bv.to_le_bytes(range)
            } else {
                let mask = BitVec::max_value_with(nbits, false) << shift;

                let orig = BitVec::from_be_bytes(range) & !&mask;
                let bv = ((bv.unsigned_cast(nbits) << shift) & mask) | orig;

                bv.to_be_bytes(range)
            }
        }
        Ok(())
    }

    pub fn read_value<T: ByteCast>(&self, address: impl Borrow<Addr>) -> Result<T, RegionIOError> {
        let range = self.view_bytes(address, T::SIZEOF)?;
        Ok(if self.endian().is_little() {
            T::from_bytes::<LE>(range)
        } else {
            T::from_bytes::<BE>(range)
        })
    }

    pub fn write_value<T: ByteCast>(
        &mut self,
        address: impl Borrow<Addr>,
        value: impl Borrow<T>,
    ) -> Result<(), RegionIOError> {
        let endian = self.endian();
        let range = self.view_bytes_mut(address, T::SIZEOF)?;
        let value = value.borrow();

        Ok(if endian.is_little() {
            value.into_bytes::<LE>(range)
        } else {
            value.into_bytes::<BE>(range)
        })
    }
    pub fn view_bytes_from(&self, address: impl Borrow<Addr>) -> Result<&[u8], RegionIOError> {
        let address = address.borrow();
        if !self.range.contains_point(address) {
            return Err(RegionIOError::OOBRead(self.name.clone()));
        }

        let offset = address
            .absolute_difference(&self.address())
            .ok_or_else(|| RegionIOError::Range(self.name.clone()))?;

        Ok(&self.bytes[offset..])
    }

    pub fn view_bytes_from_mut(
        &mut self,
        address: impl Borrow<Addr>,
    ) -> Result<&mut [u8], RegionIOError> {
        let address = address.borrow();
        if !self.range.contains_point(address) {
            return Err(RegionIOError::OOBRead(self.name.clone()));
        }

        let offset = address
            .absolute_difference(&self.address())
            .ok_or_else(|| RegionIOError::Range(self.name.clone()))?;

        Ok(&mut self.bytes.to_mut()[offset..])
    }

    pub fn view_bytes(
        &self,
        address: impl Borrow<Addr>,
        count: usize,
    ) -> Result<&[u8], RegionIOError> {
        let address = address.borrow();
        if !self.contains_range(address, count) {
            return Err(RegionIOError::OOBRead(self.name.clone()));
        }

        let offset = address
            .absolute_difference(&self.address())
            .ok_or_else(|| RegionIOError::Range(self.name.clone()))?;

        Ok(&self.bytes()[offset..offset + count])
    }

    pub fn view_bytes_mut(
        &mut self,
        address: impl Borrow<Addr>,
        count: usize,
    ) -> Result<&mut [u8], RegionIOError> {
        let address = address.borrow();
        if !self.contains_range(address, count) {
            return Err(RegionIOError::OOBWrite(self.name.clone()));
        }

        let offset = address
            .absolute_difference(&self.address())
            .ok_or_else(|| RegionIOError::Range(self.name.clone()))?;

        Ok(&mut self.bytes_mut()[offset..offset + count])
    }
    pub fn len(&self) -> usize {
        self.bytes.len()
    }
}
