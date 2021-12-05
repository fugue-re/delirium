use crate::ir::{Addr, Blk, Sub};
use crate::ir::memory::{Mem, Region};
use crate::lift::{Lifter, LifterBuilder, LifterBuilderError, LifterError};
use crate::prelude::{Endian, Entity, EntityRef, Id, Identifiable};
use crate::oracles::{BlkOracle, SubOracle};

use fugue::ir::disassembly::ContextDatabase;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use thiserror::Error;

pub struct ProjectBuilder {
    lifter_builder: LifterBuilder,
}

#[derive(Debug, Error)]
pub enum ProjectBuilderError {
    #[error(transparent)]
    LifterBuilder(#[from] LifterBuilderError),
}

impl ProjectBuilder {
    pub fn new_with(
        path: impl AsRef<Path>,
        ignore_errors: bool,
    ) -> Result<Self, ProjectBuilderError> {
        Ok(Self {
            lifter_builder: LifterBuilder::new_with(path, ignore_errors)?,
        })
    }

    pub fn new(path: impl AsRef<Path>) -> Result<Self, ProjectBuilderError> {
        Ok(Self {
            lifter_builder: LifterBuilder::new(path)?,
        })
    }

    pub fn project<'r>(
        &self,
        name: impl Into<Cow<'static, str>>,
        arch: impl Into<Cow<'static, str>>,
        convention: impl AsRef<str>,
    ) -> Result<Entity<Project<'r>>, ProjectBuilderError> {
        Ok(Project::new(
            name,
            self.lifter_builder.build(arch, convention)?,
        ))
    }

    pub fn project_with<'r>(
        &self,
        name: impl Into<Cow<'static, str>>,
        processor: impl AsRef<str>,
        endian: Endian,
        bits: u32,
        variant: impl AsRef<str>,
        convention: impl AsRef<str>,
    ) -> Result<Entity<Project<'r>>, ProjectBuilderError> {
        Ok(Project::new(
            name,
            self.lifter_builder.build_with(processor, endian, bits, variant, convention)?,
        ))
    }
}

#[derive(Clone)]
pub struct Project<'r> {
    name: Cow<'static, str>,

    lifter: Lifter,
    disassembly_context: ContextDatabase,

    memory: Mem<'r>,

    blk_oracle: Option<Arc<dyn BlkOracle>>,
    sub_oracle: Option<Arc<dyn SubOracle>>,
    
    blks: BTreeMap<Id<Blk>, Entity<Blk>>,
    blks_to_addr: BTreeMap<Id<Blk>, Addr>,
    addr_to_blks: BTreeMap<Addr, Id<Blk>>,
    
    subs: BTreeMap<Id<Sub>, Entity<Sub>>,
    subs_to_addr: BTreeMap<Id<Sub>, Addr>,
    addr_to_subs: BTreeMap<Addr, Id<Sub>>,
    syms_to_subs: BTreeMap<Cow<'static, str>, Id<Sub>>,
}

impl<'r> Project<'r> {
    pub fn new(name: impl Into<Cow<'static, str>>, lifter: Lifter) -> Entity<Self> {
        Entity::new("project", Self {
            name: name.into(),

            disassembly_context: lifter.context(),
            lifter,

            memory: Mem::new("M"),

            blk_oracle: None,
            sub_oracle: None,
            
            blks: Default::default(),
            blks_to_addr: Default::default(),
            addr_to_blks: Default::default(),

            subs: Default::default(),
            subs_to_addr: Default::default(),
            addr_to_subs: Default::default(),
            syms_to_subs: Default::default(),
        })
    }
    
    pub fn add_region_mapping(&mut self, region: Entity<Region<'r>>) {
        self.memory.add_region(region);
    }

    pub fn add_region_mapping_with(
        &mut self,
        name: impl Into<Arc<str>>,
        addr: impl Into<Addr>,
        endian: Endian,
        bytes: impl Into<Cow<'r, [u8]>>,
    ) {
        self.memory.add_region(Region::new(name, addr, endian, bytes));
    }
    
    pub fn add_blk(&mut self, addr: impl Into<Addr>) -> Result<Vec<Id<Blk>>, LifterError> {
        let addr = addr.into();
        if let Some(region) = self.memory.find_region(&addr) {
            // unwrap is safe here: we know that addr is in region
            let bytes = region.view_bytes_from(&addr).unwrap();
            // see if we have some a priori knowledge about the block's bounds
            let size_hint = self.blk_oracle
                .as_ref()
                .and_then(|o| o.blk_size(&addr));
            let blks = self.lifter.lift_blk_with(
                &mut self.disassembly_context,
                &addr,
                bytes,
                size_hint,
            )?;
            // if blks is empty, then disassembly likely failed
            if blks.is_empty () {
                // error?
                Ok(Vec::default())
            } else {
                // otherwise, we index the blocks into the current project
                // we take the identity of the first block to represent the
                // group of blocks formed, which would represent a single
                // basic block in IDA's block model.
                let blk_id = blks[0].id();
                self.blks_to_addr.insert(blk_id, addr.clone());
                self.addr_to_blks.insert(addr, blk_id);
                
                let mut blk_ids = Vec::with_capacity(blks.len());
                for blk in blks.into_iter() {
                    let blk_id = blk.id();
                    blk_ids.push(blk_id);
                    self.blks.insert(blk_id, blk);
                }
                Ok(blk_ids)
            }
        // this is likely an errors: there is no mapped region corresponding to
        // the address we want to build the block from.
        } else {
            // error?
            Ok(Vec::default())
        }
    }
    
    pub fn memory(&self) -> &Mem<'r> {
        &self.memory
    }
    
    pub fn lifter(&self) -> &Lifter {
        &self.lifter
    }
}