use clap::{Arg,App,SubCommand};
use std::error::Error;
use checksum::Checksum;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
        println!("Hello, world!");

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
                        .get_matches();

        if let ( "checksum", Some( sub_matches ) ) = matches.subcommand() {
            let checksum_file = sub_matches.value_of( "checksum-file" ).unwrap_or("checksum.json").to_string();
            let base_dir = sub_matches.value_of( "base-dir" ).unwrap_or(".").to_string();
            let mut checksum = Checksum::new( &checksum_file, &base_dir );
            checksum.run().await;
        } else {
            std::process::exit( -1 );
        }

        Ok(())
}


mod checksum;
mod checksums;