use std::error::Error;
use walkdir::WalkDir;
use std::{fmt,fs};
use std::path::{Path,PathBuf};
use std::io::{Read,Write};
use crate::checksums::*;
use crate::command_async::CommandAsync;
use indicatif::{ProgressBar,ProgressStyle};
use sha1::Sha1;

use rayon::prelude::*;

use async_trait::async_trait;
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct Verifier {
    checksum_file: String,
    base_dir: PathBuf,
    changed_file: Option< String >,
    added_file: Option< String >,
    removed_file: Option< String >,
}

impl Verifier {
    pub fn new( checksum_file: &str, base_dir: &PathBuf ) -> Self {
        Self {
            checksum_file: checksum_file.to_string(),
            base_dir: base_dir.to_owned(),
            changed_file: None,
            added_file: None,
            removed_file: None,
        }
    }

    pub fn set_changed_file( &mut self, changed_file: &str ) {
        self.changed_file = Some( changed_file.to_string() )
    }
    pub fn set_removed_file( &mut self, removed_file: &str ) {
        self.removed_file = Some( removed_file.to_string() )
    }
    pub fn set_added_file( &mut self, added_file: &str ) {
        self.added_file = Some( added_file.to_string() )
    }
}

#[async_trait]
impl CommandAsync for Verifier {
    async fn run( &mut self ) -> anyhow::Result<()> {
        let old_checksums = Checksums::load( &self.checksum_file )?;
        let mut new_checksums = Checksums::new( "sha1" );

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
                            new_checksums.add( ce );
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

//        dbg!( &new_checksums );
//        println!( "Calculating checksums for {} files. {} bytes total.", new_checksums.len(), new_checksums.total_size() );

        // :TODO: there is high potential for doing this smarter, but for now we just do the brute force, straight forward things
        let mut added = Vec::new();
        let mut changed = Vec::new();
        let mut removed = Vec::new();
        let mut unchanged = Vec::new();

        let algorithm = old_checksums.algorithm().to_string();

        for o in old_checksums.entries() {
            match new_checksums.find_mut( &o.path() ) {
                None => {
                    removed.push( o.path().to_owned() );
                },
                Some( n ) => {
                    if o.size() != n.size() {
                        changed.push( o.path().to_owned() );
                    } else {
                        n.calculate_hash( &self.base_dir, &algorithm, None );
                        if o.hash() != n.hash() {
                            changed.push( o.path().to_owned() );
                        } else {
                            unchanged.push( o.path().to_owned() );
                        }
                    }
                }
            }
        }

        for n in new_checksums.entries() {
            match old_checksums.find( &n.path() ) {
                Some( _ ) => {},
                None => added.push( n.path().to_owned() ),
            }
        }

        dbg!(&unchanged);
        dbg!(&changed);
        dbg!(&removed);
        dbg!(&added);

        if let Some( changed_file ) = &self.changed_file {
            let mut f = std::fs::File::create(&changed_file).expect("create file failed for changed file");
            for e in changed {
                f.write_all(format!("{}\n", e.to_string_lossy() ).as_bytes());
            }
        }
        if let Some( removed_file ) = &self.removed_file {
            let mut f = std::fs::File::create(&removed_file).expect("create file failed for removed file");
            for e in removed {
                f.write_all(format!("{}\n", e.to_string_lossy() ).as_bytes());
            }
        }
        if let Some( added_file ) = &self.added_file {
            let mut f = std::fs::File::create(&added_file).expect("create file failed for added file");
            for e in added {
                f.write_all(format!("{}\n", e.to_string_lossy() ).as_bytes());
            }
        }
        Ok(())
    }


}
