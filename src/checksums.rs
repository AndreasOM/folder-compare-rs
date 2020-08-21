use std::path::PathBuf;
use std::error::Error;
use serde::{Deserialize, Serialize};
use rayon::prelude::*;

#[derive(Debug,Deserialize,Serialize)]
pub struct ChecksumsEntry {
    path: PathBuf,
    size: u64,
    hash: String,
}

impl ChecksumsEntry {
    pub fn new(
        path: &PathBuf,
        size: u64,
        hash: &str,
    ) -> Self {
        Self {
            path: path.to_owned(),
            size: size,
            hash: hash.to_string(),
        }
    }

    pub fn set_hash(&mut self, hash: &str) {
        self.hash = hash.to_string();
    }

    pub fn path( &self ) -> &PathBuf {
        &self.path
    }
}

#[derive(Debug,Deserialize,Serialize)]
pub struct Checksums {
    algorithm: String,
    entries: Vec<ChecksumsEntry>,
    total_size: u64,
}

impl Checksums {
    pub fn new( algorithm: &str ) -> Self {
        Self {
            algorithm: algorithm.to_string(),
            entries: Vec::new(),
            total_size: 0,
        }
    }

    pub fn save( &self, filename: &str )-> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string( &self )?;
        std::fs::write( filename, json )?;
        Ok(())
    }

    pub fn add( &mut self, entry: ChecksumsEntry ) {
        self.total_size += entry.size;
        self.entries.push( entry );
    }

    pub fn len( &self ) -> usize {
        self.entries.len()
    }

    pub fn total_size( &self ) -> u64 {
        self.total_size
    }

    pub fn iter_mut( &mut self ) -> std::slice::IterMut::< ChecksumsEntry > {
        self.entries.iter_mut()
    }

    pub fn par_iter_mut( &mut self ) -> rayon::slice::IterMut::< ChecksumsEntry > {
        self.entries.par_iter_mut()
    }

    pub fn entries_mut( &mut self ) -> &mut Vec<ChecksumsEntry> {
        &mut self.entries
    }
}
