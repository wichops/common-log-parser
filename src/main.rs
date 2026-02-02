use std::{fs::File};
use std::io::{BufReader};
use std::io::prelude::*;

use clap::Parser;


pub mod parser;

use crate::parser::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    filename: String,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Cli::parse();

    println!("filename: {}", args.filename);
    let file = File::open(args.filename)?;
    let buf = BufReader::new(file);

    for line in buf.lines() {
        let line = line?;
        let log_entry = parse_common_log(&line)?;

        println!("{:?}", log_entry);
    }

    Ok(())
}
