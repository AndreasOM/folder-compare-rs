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
pub struct Compare {
    checksum_file_old: String,
    checksum_file_new: String,
    changed_file: Option< String >,
    added_file: Option< String >,
    removed_file: Option< String >,
}

impl Compare {
    pub fn new( checksum_file_old: &str, checksum_file_new: &str ) -> Self {
        Self {
            checksum_file_old: checksum_file_old.to_string(),
            checksum_file_new: checksum_file_new.to_string(),
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
impl CommandAsync for Compare {
    async fn run( &mut self ) -> anyhow::Result<()> {
        let old_checksums = Checksums::load( &self.checksum_file_old )?;
        let new_checksums = Checksums::load( &self.checksum_file_new )?;

//        dbg!(&old_checksums, &new_checksums);

        let bar = ProgressBar::new( 1_000_000u64 );
        let spinner_style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{prefix:.bold.dim} {spinner} {wide_msg} {pos} files found.");

        bar.set_style(spinner_style.clone());

        // :TODO: there is high potential for doing this smarter, but for now we just do the brute force, straight forward things
        let mut added = Vec::new();
        let mut changed = Vec::new();
        let mut removed = Vec::new();
        let mut unchanged = Vec::new();

        if old_checksums.algorithm() != new_checksums.algorithm() {
            println!("ERROR: Different algorithms for checksums");
            return Ok(());
        };
        for o in old_checksums.entries() {
            bar.inc( 1 );
            match new_checksums.find( &o.path() ) {
                None => {
                    removed.push( o.path().to_owned() );
                },
                Some( n ) => {
                    if o.size() != n.size() {
                        changed.push( o.path().to_owned() );
                    } else {
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
            bar.inc( 1 );
            match old_checksums.find( &n.path() ) {
                Some( _ ) => {},
                None => added.push( n.path().to_owned() ),
            }
        }
/*
        dbg!(&unchanged);
        dbg!(&changed);
        dbg!(&removed);
        dbg!(&added);
*/
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
