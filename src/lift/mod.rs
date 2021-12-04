use fugue::ir::convention::Convention;
use fugue::ir::{LanguageDB, Translator};
use fugue::ir::disassembly::ContextDatabase;

use std::borrow::Cow;
use std::path::Path;

use thiserror::Error;

use crate::ir::{Addr, Blk};
use crate::prelude::{Endian, Entity};

mod ecode;
use ecode::passes::ECodeVarIndex;

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

    pub fn lift_blk(&self, ctxt: &mut ContextDatabase, addr: &Addr, bytes: &[u8]) -> Vec<Entity<Blk>> {
        self.lift_blk_with(ctxt, addr, bytes, None)
    }
    
    pub fn lift_blk_with(&self, ctxt: &mut ContextDatabase, addr: &Addr, bytes: &[u8], size_hint: Option<usize>) -> Vec<Entity<Blk>> {
        let actual_size = bytes.len();
        let attempt_size = size_hint
            .map(|hint| actual_size.min(hint))
            .unwrap_or(actual_size);
        
        let bytes = &bytes[..attempt_size];
        let mut blks = Vec::new();
        
        blks
    }
}