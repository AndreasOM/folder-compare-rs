use std::error::Error;
use walkdir::WalkDir;
use std::{fmt,fs};
use std::path::{Path,PathBuf};
use std::io::Read;
use crate::checksums::*;
use indicatif::{ProgressBar,ProgressStyle};
use sha1::Sha1;

use rayon::prelude::*;

#[derive(Debug)]
pub enum ChecksumError {
    Generic( String ),
}

impl fmt::Display for ChecksumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChecksumError")
    }
}
impl Error for ChecksumError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // Some(&self.side)
        None
    }
}

pub struct Checksum {
    checksum_file: String,
    base_dir: String,
}

impl Checksum {
    pub fn new( checksum_file: &str, base_dir: &str ) -> Self {
        Self {
            checksum_file: checksum_file.to_string(),
            base_dir: base_dir.to_string(),
        }
    }

    pub async fn run( &mut self ) -> Result<(), Box<dyn Error>> {
        let mut checksums = Checksums::new( "sha1" );

        for e in WalkDir::new( &self.base_dir ) {
            match e {
                Ok( e ) => {
                    match e.metadata() {
                        Ok( m ) => if m.is_file() {
                            let ce = ChecksumsEntry::new( &e.path().to_path_buf(), m.len(), "" );
                            checksums.add( ce );
                        },
                        Err( e ) => {
                            return Err( Box::new( ChecksumError::Generic( String::from( "Missing metadata" ) ) ) );
                        },        
                    };
                },
                Err( e ) => {
                    return Err( Box::new( ChecksumError::Generic( String::from( "WalkDir error" ) ) ) );
                },
            }
        };

//        dbg!( &checksums );
        println!( "Calculating checksums for {} files. {} bytes total.", checksums.len(), checksums.total_size() );

        let bar = ProgressBar::new( checksums.len() as u64 );

        bar.set_style(
            ProgressStyle::default_bar()
            .template( "{spinner:.green} Calculating checksums [{wide_bar:.cyan/blue}] {pos}/{len} {percent}% ETA: ~{eta}" )
        );
//        for e in checksums.entries_mut().par_iter_mut() {
        checksums.entries_mut().par_iter_mut()
            .for_each(|e| {
                bar.inc( 1 );
                let mut fullpath = PathBuf::new();
                fullpath.push( "." );
//                fullpath.push( &self.base_dir );
                fullpath.push( &e.path() );
                let mut f = match std::fs::File::open(&fullpath) {
                    Err( e ) => { dbg!( &fullpath ); panic!("") },   // :TODO: handle
                    Ok( f ) => f,
                };

                let mut sha1 = Sha1::new();
                let mut data = Vec::<u8>::new();
                // :TODO: read blockwise
                f.read_to_end(&mut data);
                sha1.update(&mut data);
                let hash = sha1.digest();
                e.set_hash( &hash.to_string().to_uppercase() );
            });
//        }
//        dbg!( &checksums );
        checksums.save( &self.checksum_file );
        Ok(())
    }
}

