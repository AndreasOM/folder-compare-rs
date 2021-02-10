use std::error::Error;
use walkdir::WalkDir;
use std::{fmt,fs};
use std::path::{Path,PathBuf};
use std::io::Read;
use crate::checksums::*;
use crate::command_async::CommandAsync;
use crate::message::Message;

use indicatif::{MultiProgress,ProgressBar,ProgressStyle};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

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

        // :TODO: make configurable
        rayon::ThreadPoolBuilder::new().num_threads(8).build_global().unwrap();

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

        // handle progress update in thread, so we can let rayon do the work management
        let (tx,rx) = channel();
        let total_size = checksums.total_size();
        let total_files = checksums.len() as u64;
        let watcher = tokio::spawn(async move {
            let mut keep_running = true;
            let mut delay = 20;
            let mut multi_bar = MultiProgress::new();
            let bar_size = ProgressBar::new( total_size );
//            let bar_size = multi_bar.add( bar_size );
            bar_size.set_style(
                ProgressStyle::default_bar()
                .template( "{spinner:.green} Calculating checksums [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} at {bytes_per_sec} bytes/s {percent}% ETA: ~{eta_precise}" )
            );
            let bar_files = ProgressBar::new( total_files );
//            let bar_files = multi_bar.add( bar_files );
            bar_files.set_style(
                ProgressStyle::default_bar()
                .template( "{spinner:.green} Calculating checksums [{wide_bar:.cyan/blue}] {pos}/{len} {percent}% ETA: ~{eta}" )
            );

            while keep_running {
                tokio::time::delay_for( std::time::Duration::from_millis(delay) ).await;
                match rx.try_recv() {
                    Ok( msg ) => {
                        match msg {
                            Message::Started( total_size, total_files ) => {
                                bar_size.set_length( total_size as u64 );
                                bar_files.set_length( total_files as u64 );
                            },
                            Message::Progress( size ) => {
                                bar_size.inc( size as u64 );
                            },
                            Message::FileDone => {
//                                bar_files.inc( 1 );   // :TODO: MultiProgress currently seems broken :(
                            },
                            Message::Done => {
                                keep_running = false;
                            },
                            m => {
                                dbg!(&m);
                            },
                        }
                        delay = 1;      // if we got a mesage we try again fast
                    },
                    Err( _e ) => {
                        delay = 2000;   // if we didn't get a message we can sleep for a bit
                    },
                }
//                dbg!(&delay);
            }; // while keep_running
            multi_bar.join();
        });

        tx.send( Message::Started( checksums.total_size(), checksums.len() as u64 ) )?;

        let algorithm = checksums.algorithm().to_string();
        let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads( 16 )
                    .build()
                    .unwrap();
        let entries = checksums.entries_mut();
        let base_dir = self.base_dir.clone();
        let ctx = tx.clone();
        pool.scope(move |s| {
            for e in entries.into_iter() {
                let tx = ctx.clone();
                let base_dir = base_dir.clone();
                let algorithm = algorithm.clone();
                s.spawn(move |_| {
                    e.calculate_hash( &base_dir, &algorithm, Some( tx ) );
                });
            }
        });
        /*
        checksums.entries_mut().par_iter_mut()
            .for_each(|e| {
                bar.inc( 1 );
                e.calculate_hash( &self.base_dir, &algorithm, Some( tx ) );
            });
            */
        tx.send( Message::Done );
//        dbg!( &checksums );
        checksums.save( &self.checksum_file );
        tokio::join!( watcher );
        Ok(())
    }
}

