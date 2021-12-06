use fugue::ir::convention::Convention;
use fugue::ir::{LanguageDB, Translator};
use fugue::ir::disassembly::ContextDatabase;

use std::borrow::{Borrow, Cow};
use std::path::Path;

use thiserror::Error;

use crate::ir::{Addr, Blk};
use crate::prelude::{Endian, Entity};

mod ecode;
use ecode::passes::ECodeVarIndex;
use ecode::utils::ECodeExt;

#[derive(Clone)]
pub struct LifterBuilder {
    language_db: LanguageDB,
}

#[derive(Debug, Error)]
pub enum LifterBuilderError {
    #[error(transparent)]
    ArchDef(#[from] fugue::arch::ArchDefParseError),
    #[error(transparent)]
    Backend(#[from] fugue::ir::error::Error),
    #[error("unsupported architecture")]
    UnsupportedArch,
    #[error("unsupported architecture calling convention")]
    UnsupportedConv,
}

impl LifterBuilder {
    pub fn new_with(
        path: impl AsRef<Path>,
        ignore_errors: bool,
    ) -> Result<Self, LifterBuilderError> {
        let language_db = LanguageDB::from_directory_with(path, ignore_errors)?;
        Ok(Self { language_db })
    }
    pub fn new(path: impl AsRef<Path>) -> Result<Self, LifterBuilderError> {
        Self::new_with(path, true)
    }

    pub fn build(
        &self,
        tag: impl Into<Cow<'static, str>>,
        convention: impl AsRef<str>,
    ) -> Result<Lifter, LifterBuilderError> {
        let tag = tag.into();
        let convention = convention.as_ref();

        let builder = self
            .language_db
            .lookup_str(&*tag)?
            .ok_or_else(|| LifterBuilderError::UnsupportedArch)?;
        let translator = builder.build()?;

        if let Some(convention) = translator.compiler_conventions().get(&*convention).cloned() {
            Ok(Lifter::new(translator, convention))
        } else {
            Err(LifterBuilderError::UnsupportedConv)
        }
    }

    pub fn build_with(
        &self,
        processor: impl AsRef<str>,
        endian: Endian,
        bits: u32,
        variant: impl AsRef<str>,
        convention: impl AsRef<str>,
    ) -> Result<Lifter, LifterBuilderError> {
        let convention = convention.as_ref();

        let processor = processor.as_ref();
        let variant = variant.as_ref();

        let builder = self
            .language_db
            .lookup(processor, endian, bits as usize, variant)
            .ok_or_else(|| LifterBuilderError::UnsupportedArch)?;
        let translator = builder.build()?;

        if let Some(convention) = translator.compiler_conventions().get(&*convention).cloned() {
            Ok(Lifter::new(translator, convention))
        } else {
            Err(LifterBuilderError::UnsupportedConv)
        }
    }
}

#[derive(Clone)]
pub struct Lifter {
    translator: Translator,
    convention: Convention,
    register_ecode_index: ECodeVarIndex,
}

#[derive(Debug, Error)]
pub enum LifterError {
    #[error(transparent)]
    AddrSize(#[from] crate::ir::memory::address::AddrConvertError),
    #[error(transparent)]
    Disassembly(#[from] fugue::ir::error::Error),
}

impl Lifter {
    fn new(translator: Translator, convention: Convention) -> Self {
        Self {
            register_ecode_index: ECodeVarIndex::registers(&translator),
            translator,
            convention,
        }
    }
    
    pub fn context(&self) -> ContextDatabase {
        self.translator.context_database()
    }

    pub fn lift_blk(&self, ctxt: &mut ContextDatabase, addr: impl Borrow<Addr>, bytes: &[u8]) -> Result<Vec<Entity<Blk>>, LifterError> {
        self.lift_blk_with(ctxt, addr, bytes, None)
    }
    
    // We lift blocks based on IDA's model of basic blocks (i.e., only
    // terminate a block on local control-flow/a return).
    //
    // We note that each lifted instruction may have internal control-flow
    // and so for a given block under IDA's model, we may produce many strict
    // basic blocks (i.e., Blks). For each instruction lifted, we determine
    // the kind of control-flow that leaves the chunk of ECode statements. We
    // classify each flow as one of six kinds:
    //
    //  1. Unresolved (call, return, branch, cbranch with computed target)
    //  2. IntraIns   (cbranch, branch with inter-chunk flow)
    //  3. IntraBlk   (fall-through)
    //  4. InterBlk   (cbranch, branch with non-inter-chunk flow)
    //  5. InterSub   (call, return)
    //  6. Intrinsic  (intrinsic in statement position)
    //  
    // Each architectural instruction initially becomes one or more blocks; we
    // can later apply a merge strategy to clean blocks up if needed. However,
    // this representation enables us to avoid splitting blocks at a later
    // stage and allows us to build a mapping between each instruction and its 
    // blocks.
    pub fn lift_blk_with(&self, ctxt: &mut ContextDatabase, addr: impl Borrow<Addr>, bytes: &[u8], size_hint: Option<usize>) -> Result<Vec<Entity<Blk>>, LifterError> {
        let addr = addr.borrow();
        let actual_size = bytes.len();
        let attempt_size = size_hint
            .map(|hint| actual_size.min(hint))
            .unwrap_or(actual_size);
        
        let bytes = &bytes[..attempt_size];

        log::debug!("lifting block at {} with size boundary of {}", addr, attempt_size);

        let mut blks = Vec::new();
        let mut stmts = Vec::new();
        let mut offset = 0;

        while offset < attempt_size {
            let taddr = self.translator.address(u64::try_from(addr + offset)?);
            let view = &bytes[offset..];

            log::trace!("lifting instruction at {}", taddr);
            
            if let Ok(ecode) = self.translator.lift_ecode(ctxt, taddr, view) {
                log::trace!(
                    "lifted instruction sequence consists of {} operations over {} bytes",
                    ecode.operations().len(),
                    ecode.length()
                );
                
                let targets = ecode.branch_targets();
                let length = ecode.length();

                log::trace!(
                    "lifted instruction sequence consists of {} branch targets",
                    targets.len(),
                );
                
                let mut should_stop = false;
                for (i, tgt) in targets.iter() {
                    log::trace!("- from {}.{}: {}", addr + offset, i, tgt);
                    should_stop |= tgt.ends_block();
                }
                
                log::trace!(
                    "lifted instruction should terminate block: {}",
                    should_stop,
                );
                
                if should_stop {
                    break
                }
                
                stmts.push((ecode, targets));
                
                offset += length;
            } else {
                log::trace!("instruction could not be lifted");
                break;
            }
        }
        Ok(blks)
    }
}

#[cfg(test)]
mod test {
    use std::env;
    use std::path::PathBuf;
    use super::*;
    
    #[test]
    fn test_blk_disasm() -> Result<(), Box<dyn std::error::Error>> {
        env_logger::init();

        let root = env::var("DELIRIUM_TEST_ENV_ROOT")?;
        let path = PathBuf::from_iter([&root, "processors"]);

        let builder = LifterBuilder::new(&path)?;
        let lifter = builder.build("x86:LE:32:default", "gcc")?;
        
        let mut ctxt = lifter.context();

        let mut lift = |addr: u32, bytes: &[u8]| lifter.lift_blk(&mut ctxt, Addr::from(addr), bytes);
        
        let _blk1 = lift(0x1000, &[0x90u8])?;
        let _blk2 = lift(0x1001, &[0xf3u8, 0xaau8, 0x90u8])?;
        
        Ok(())
    }
}