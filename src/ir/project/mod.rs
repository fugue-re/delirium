use crate::ir::memory::Memory;

use crate::lift::{Lifter, LifterBuilder, LifterBuilderError};

use crate::prelude::{Endian, Entity};

use std::borrow::Cow;
use std::path::Path;

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

    pub fn empty_project<'r>(
        &self,
        name: impl Into<Cow<'static, str>>,
        arch: impl Into<Cow<'static, str>>,
        convention: impl AsRef<str>,
    ) -> Result<Entity<Project<'r>>, ProjectBuilderError> {
        Ok(Project::empty(
            name,
            self.lifter_builder.build(arch, convention)?,
        ))
    }

    pub fn empty_project_with<'r>(
        &self,
        name: impl Into<Cow<'static, str>>,
        processor: impl AsRef<str>,
        endian: Endian,
        bits: u32,
        variant: impl AsRef<str>,
        convention: impl AsRef<str>,
    ) -> Result<Entity<Project<'r>>, ProjectBuilderError> {
        Ok(Project::empty(
            name,
            self.lifter_builder.build_with(processor, endian, bits, variant, convention)?,
        ))
    }
}

#[derive(Clone)]
pub struct Project<'r> {
    name: Cow<'static, str>,
    memory: Memory<'r>,
    lifter: Lifter,
}

impl<'r> Project<'r> {
    pub fn empty(name: impl Into<Cow<'static, str>>, lifter: Lifter) -> Entity<Self> {
        Entity::new("project", Self {
            name: name.into(),
            memory: Memory::new("M"),
            lifter,
        })
    }
}