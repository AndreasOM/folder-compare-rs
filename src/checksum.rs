use std::error::Error;
use walkdir::WalkDir;
use std::{fmt,fs};
use std::path::{Path,PathBuf};
use std::io::Read;
use crate::checksums::*;
use crate::command_async::CommandAsync;
use indicatif::{ProgressBar,ProgressStyle};

use rayon::prelude::*;

use async_trait::async_trait;

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

#[derive(Debug)]
pub struct Checksum {
    checksum_file: String,
    base_dir: PathBuf,
}

impl Checksum {
    pub fn new( checksum_file: &str, base_dir: &PathBuf ) -> Self {
        Self {
            checksum_file: checksum_file.to_string(),
            base_dir: base_dir.to_owned(),
        }
    }
}

#[async_trait]
impl CommandAsync for Checksum {
    async fn run( &mut self ) -> anyhow::Result<()> {
        let mut checksums = Checksums::new( "sha1" );

        
        let bar = ProgressBar::new( 1_000_000u64 );
        let spinner_style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{prefix:.bold.dim} {spinner} {wide_msg} {pos} files found.");

        bar.set_style(spinner_style.clone());

        for e in WalkDir::new( &self.base_dir ) {
            match e {
                Ok( e ) => {
                    match e.metadata() {
                        Ok( m ) => if m.is_file() {
                            bar.inc( 1 );
                            let rp = e.path().strip_prefix( &self.base_dir )?;
                            let ce = ChecksumsEntry::new( rp, m.len(), "" );
                            checksums.add( ce );
                        },
                        Err( e ) => {
//                            return Err( Box::new( ChecksumError::Generic( String::from( "Missing metadata" ) ) ) );
                        },        
                    };
                },
                Err( e ) => {
//                    return Err( Box::new( ChecksumError::Generic( String::from( "WalkDir error" ) ) ) );
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
        let algorithm = checksums.algorithm().to_string();
//        for e in checksums.entries_mut().par_iter_mut() {
        checksums.entries_mut().par_iter_mut()
            .for_each(|e| {
                bar.inc( 1 );
                e.calculate_hash( &self.base_dir, &algorithm );
            });
//        }
//        dbg!( &checksums );
        checksums.save( &self.checksum_file );
        Ok(())
    }
}

