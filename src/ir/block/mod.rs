use crate::ir::{Def, Jmp, Phi};
use crate::prelude::{Identifiable, Entity};

use std::mem::take;

#[derive(Clone)]
pub struct Blk {
    phis: Vec<Entity<Phi>>,
    defs: Vec<Entity<Def>>,
    jmps: Vec<Entity<Jmp>>,
}

impl Blk {
    pub fn new() -> Entity<Blk> {
        Self::new_with(
            Default::default(),
            Default::default(),
            Default::default(),
        )
    }

    pub fn new_with(phis: Vec<Entity<Phi>>, defs: Vec<Entity<Def>>, jmps: Vec<Entity<Jmp>>) -> Entity<Blk> {
        Entity::new("blk", Self {
            phis,
            defs,
            jmps,
        })
    }
    
    pub fn defs(&self) -> &[Entity<Def>] {
        &self.defs
    }
    
    pub fn phis(&self) -> &[Entity<Phi>] {
        &self.phis
    }

    pub fn jmps(&self) -> &[Entity<Jmp>] {
        &self.jmps
    }
    
    pub fn add_def(&mut self, def: Entity<Def>) {
        self.defs.push(def);
    } 

    pub fn add_phi(&mut self, phi: Entity<Phi>) {
        self.phis.push(phi);
    } 

    pub fn add_jmp(&mut self, jmp: Entity<Jmp>) {
        self.jmps.push(jmp);
    } 
    
    fn split_off(&mut self, pos: Option<usize>) -> Entity<Self> {
        let ndefs = if let Some(pos) = pos {
            self.defs.split_off(pos)
        } else {
            Default::default()
        };

        let nblk = Self::new_with(
            Default::default(),
            ndefs,
            take(&mut self.jmps),
        );
        
        self.add_jmp(Jmp::branch(nblk.id()));
        
        nblk
    }
    
    pub fn split_top(&mut self) -> Entity<Blk> {
        self.split_off(Some(0))
    }
    
    pub fn split_bottom(&mut self) -> Entity<Blk> {
        self.split_off(Some(self.defs.len()))
    }
    
    pub fn split_before(&mut self, def: impl Identifiable<Def>) -> Entity<Self> {
        let id = def.id();
        let pos = self.defs.iter().position(|def| def.id() == id);
        self.split_off(pos)
    }

    pub fn split_after(&mut self, def: impl Identifiable<Def>) -> Entity<Self> {
        let id = def.id();
        let pos = self.defs.iter().position(|def| def.id() == id).map(|pos| pos + 1);
        self.split_off(pos)
    }
}