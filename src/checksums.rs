use std::path::{Path,PathBuf};
use std::error::Error;
use serde::{Deserialize, Serialize};
use rayon::prelude::*;
use std::io::Read;
use sha1::Sha1;

use std::io::BufReader;

#[derive(Debug,Deserialize,Serialize)]
pub struct ChecksumsEntry {
    path: PathBuf,
    size: u64,
    hash: String,
}

impl ChecksumsEntry {
    pub fn new(
        path: &Path,
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

    pub fn calculate_hash(&mut self, base_dir: &Path, algorithm: &str ) -> anyhow::Result< () > {
        let mut fullpath = PathBuf::new();
        fullpath.push( &base_dir );
        fullpath.push( &self.path() );
        let mut f = match std::fs::File::open(&fullpath) {
            Err( e ) => { dbg!( &fullpath ); panic!("") },   // :TODO: handle
            Ok( f ) => f,
        };

        let mut sha1 = Sha1::new();
        let mut data = Vec::<u8>::new();
        // :TODO: make configurable
        const BLOCKSIZE: usize = 128*1024;

        if BLOCKSIZE == 0 {
            f.read_to_end(&mut data);
            sha1.update(&mut data);
        } else {
            let mut r = BufReader::with_capacity( BLOCKSIZE, f );
            let mut buffer = [0; BLOCKSIZE];
            loop {
                let n = r.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                sha1.update(&buffer[..n]);
            }
        }
        let hash = sha1.digest();
        self.set_hash( &hash.to_string().to_uppercase() );
        Ok(())
    }

    pub fn path( &self ) -> &PathBuf {
        &self.path
    }

    pub fn size( &self ) -> u64 {
        self.size
    }

    pub fn hash( &self ) -> &str {
        &self.hash
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

    pub fn load( filename: &str ) -> anyhow::Result< Checksums > {
        let json = std::fs::read_to_string( &filename )?;
        let s = serde_json::from_str( &json )?;
        Ok(s)
    }

    pub fn add( &mut self, entry: ChecksumsEntry ) {
        self.total_size += entry.size;
        self.entries.push( entry );
    }

    pub fn find_mut( &mut self, filename: &PathBuf ) -> Option < &mut ChecksumsEntry > {
        self.entries.iter_mut().find({ |e|
            e.path == *filename
        })
    }

    pub fn find( &self, filename: &PathBuf ) -> Option < &ChecksumsEntry > {
        self.entries.iter().find({ |e|
            e.path == *filename
        })
    }

    pub fn algorithm( &self ) -> &str {
        &self.algorithm
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
    pub fn entries( &self ) -> &Vec<ChecksumsEntry> {
        &self.entries
    }
}
