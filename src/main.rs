use std::{
    fs::{self, FileTimes},
    path,
};

use anyhow::Context;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// A  FILE argument that does not exist is created empty, unless -c or -h is supplied.
    file: Option<path::PathBuf>,

    /// Do not create any files
    #[clap(short = 'c', long)]
    no_create: bool,

    /// change only the access time
    #[clap(short = 'a')]
    access: bool,

    /// parse STRING and use it instead of current time
    #[clap(short = 'd', long)]
    date: Option<String>,

    // affect  each  symbolic link instead of any referenced file (useful only
    // on systems that can change the timestamps of a symlink)
    #[clap(short = 'n', long)]
    no_dereference: bool,

    ///change only the modification time
    #[clap(short = 'm')]
    modify_time_only: bool,

    ///use this file's times instead of current time
    #[clap(short = 'r', long)]
    reference: Option<path::PathBuf>,

    /// [[CC]YY]MMDDhhmm[.ss]
    /// use specified time instead of current time,  with  a  date-time  format
    /// that differs from -d's
    #[clap(short = 't')]
    time: Option<String>,
}

const NAME: &str = "tick";

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if let Some(file) = args.file {
        let metadata = fs::metadata(&file);
        match metadata {
            Ok(_) => {
                let options =
                    fs::File::open(&file).with_context(|| format!("opening file {:?}", &file))?;

                if args.access {
                    let filetimes = FileTimes::new().set_accessed(std::time::SystemTime::now());
                    let _ = options.set_times(filetimes);
                }

                if args.modify_time_only {
                    let filetimes = FileTimes::new().set_modified(std::time::SystemTime::now());
                    let _ = options.set_times(filetimes);
                }
            }
            Err(_) => {
                if !args.no_create {
                    std::fs::File::create(&file)?;
                }
            }
        }
    } else {
        println!("{NAME}: missing file operand");
        println!("Try '{NAME} --help' for more information");
    }
    Ok(())
}
