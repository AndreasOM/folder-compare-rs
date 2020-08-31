use clap::{Arg,App,SubCommand};
use std::error::Error;
use checksum::Checksum;
use compare::Compare;
use verifier::Verifier;
use crate::command_async::CommandAsync;

#[tokio::main]
//pub async fn main() -> Result<(), Box<dyn Error>> {
pub async fn main() -> anyhow::Result<()> {
        println!("Let's check...");

        let matches = App::new("folder-compare-rs")
                        .version("0.1")
                        .subcommand( SubCommand::with_name("checksum")
                            .arg( Arg::with_name("checksum-file")
                                .long( "checksum-file" )
                                .value_name( "checksum-file" )
                                .takes_value( true )
                            )
                            .arg( Arg::with_name("base-dir")
                                .long( "base-dir" )
                                .value_name( "base-dir" )
                                .takes_value( true )
                            )
                        )
                        .subcommand( SubCommand::with_name("verify")
                            .arg( Arg::with_name("checksum-file")
                                .long( "checksum-file" )
                                .value_name( "checksum-file" )
                                .takes_value( true )
                            )
                            .arg( Arg::with_name("base-dir")
                                .long( "base-dir" )
                                .value_name( "base-dir" )
                                .takes_value( true )
                            )
                            .arg( Arg::with_name("changed-file")
                                .long( "changed-file" )
                                .value_name( "changed-file" )
                                .takes_value( true )
                            )
                            .arg( Arg::with_name("added-file")
                                .long( "added-file" )
                                .value_name( "added-file" )
                                .takes_value( true )
                            )
                            .arg( Arg::with_name("removed-file")
                                .long( "removed-file" )
                                .value_name( "removed-file" )
                                .takes_value( true )
                            )
                        )
                        .subcommand( SubCommand::with_name("compare")
                            .arg( Arg::with_name("checksum-file-old")
                                .long( "checksum-file-old" )
                                .value_name( "checksum-file-old" )
                                .takes_value( true )
                            )
                            .arg( Arg::with_name("checksum-file-new")
                                .long( "checksum-file-new" )
                                .value_name( "checksum-file-new" )
                                .takes_value( true )
                            )
                            .arg( Arg::with_name("changed-file")
                                .long( "changed-file" )
                                .value_name( "changed-file" )
                                .takes_value( true )
                            )
                            .arg( Arg::with_name("added-file")
                                .long( "added-file" )
                                .value_name( "added-file" )
                                .takes_value( true )
                            )
                            .arg( Arg::with_name("removed-file")
                                .long( "removed-file" )
                                .value_name( "removed-file" )
                                .takes_value( true )
                            )
                        )
                        .get_matches();

        let mut command: Box< dyn CommandAsync > = if let ( "checksum", Some( sub_matches ) ) = matches.subcommand() {
            let checksum_file = sub_matches.value_of( "checksum-file" ).unwrap_or("checksum.json").to_string();
            let base_dir = std::fs::canonicalize(sub_matches.value_of( "base-dir" ).unwrap_or(".").to_string()).expect( "base-dir is invalid");
            let mut checksum = Checksum::new( &checksum_file, &base_dir );
            //checksum.run().await;
            Box::new( checksum )
        } else if let ( "verify", Some( sub_matches ) ) = matches.subcommand() {
            let checksum_file = sub_matches.value_of( "checksum-file" ).unwrap_or("checksum.json").to_string();
            let base_dir = std::fs::canonicalize(sub_matches.value_of( "base-dir" ).unwrap_or(".").to_string()).expect( "base-dir is invalid");
            let changed_file = sub_matches.value_of( "changed-file" ).unwrap_or("").to_string();
            let added_file = sub_matches.value_of( "added-file" ).unwrap_or("").to_string();
            let removed_file = sub_matches.value_of( "removed-file" ).unwrap_or("").to_string();
            let mut checksum = Verifier::new( &checksum_file, &base_dir );
            if changed_file != "" {
                checksum.set_changed_file( &changed_file );
            }
            if added_file != "" {
                checksum.set_added_file( &added_file );
            }
            if removed_file != "" {
                checksum.set_removed_file( &removed_file );
            }

            //checksum.run().await;
            Box::new( checksum )
        } else if let ( "compare", Some( sub_matches ) ) = matches.subcommand() {
            let checksum_file_old = sub_matches.value_of( "checksum-file-old" ).unwrap_or("old-checksum.json").to_string();
            let checksum_file_new = sub_matches.value_of( "checksum-file-new" ).unwrap_or("new-checksum.json").to_string();
            let changed_file = sub_matches.value_of( "changed-file" ).unwrap_or("").to_string();
            let added_file = sub_matches.value_of( "added-file" ).unwrap_or("").to_string();
            let removed_file = sub_matches.value_of( "removed-file" ).unwrap_or("").to_string();
            let mut checksum = Compare::new( &checksum_file_old, &checksum_file_new );
            if changed_file != "" {
                checksum.set_changed_file( &changed_file );
            }
            if added_file != "" {
                checksum.set_added_file( &added_file );
            }
            if removed_file != "" {
                checksum.set_removed_file( &removed_file );
            }

            //checksum.run().await;
            Box::new( checksum )
        } else {
            println!("No comand given. Try help!");
            std::process::exit( -1 );
        };

//        dbg!(&command);
        command.run().await?;

        Ok(())
}


mod checksum;
mod checksums;
mod compare;
mod verifier;
mod command_async;