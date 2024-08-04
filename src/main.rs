use std::{
    fs::{self, File, FileTimes},
    io::Write,
    path::{self, PathBuf},
    time::SystemTime,
};

use anyhow::Context;
use clap::Parser;

enum Word {
    Access,
    Atime,
    Use,
    Modify,
    Mtime,
}

impl From<String> for Word {
    fn from(word: String) -> Self {
        match word.to_lowercase().as_str() {
            "access" => Word::Access,
            "atime" => Word::Atime,
            "use" => Word::Use,
            "modify" => Word::Modify,
            "mtime" => Word::Mtime,
            _ => Word::Use,
        }
    }
}

#[derive(Parser, Debug)]
struct Args {
    /// A  FILE argument that does not exist is created empty, unless -c or -h is supplied.
    files: Option<Vec<path::PathBuf>>,

    /// do not create any files
    #[clap(short = 'c', long)]
    no_create: bool,

    /// change only the access time
    #[clap(short = 'a')]
    access: bool,

    /// parse STRING and use it instead of current time
    #[clap(short = 'd', long)]
    date: Option<String>,

    /// affect each symbolic link instead of any referenced file (useful only
    /// on systems that can change the timestamps of a symlink)
    #[clap(short = 'n', long)]
    no_dereference: bool,

    ///change only the modification time
    #[clap(short = 'm')]
    modify_time_only: bool,

    ///use this file's times instead of current time
    #[clap(short = 'r', long)]
    reference: Option<path::PathBuf>,

    /// [[CC]YY]MMDDhhmm[.ss]
    /// use specified time instead of current time, with a date-time format
    /// that differs from -d's
    #[clap(short = 't')]
    time: Option<String>,

    /// specify which time to change:
    ///   access time (-a): 'access', 'atime', 'use';
    ///   modification time (-m): 'modify', 'mtime'
    #[clap(long = "time")]
    word: Option<String>,
}

const NAME: &str = "tick";

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if let Some(files) = &args.files {
        for file in files {
            let metadata = fs::metadata(&file);
            match metadata {
                Ok(_) => {
                    tick(&args, &file)?;
                }
                Err(_) => {
                    if !args.no_create {
                        std::fs::File::create(&file)?;
                    }
                }
            }
        }
    } else {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        handle.write(
            format!(
                "{}: missing operand\nTry '{} --help' for more information\n",
                NAME, NAME
            )
            .as_bytes(),
        )?;
    }
    Ok(())
}

fn set_access(file_obj: &File, time: &Source) {
    let source_time: SystemTime = match time {
        Source::Single(t) => t.clone(),
        Source::Multi(t, _) => t.clone(),
    };
    let filetimes = FileTimes::new().set_accessed(source_time);
    let _ = file_obj.set_times(filetimes);
}

fn set_modified(file_obj: &File, time: &Source) {
    let source_time: SystemTime = match time {
        Source::Single(t) => t.clone(),
        Source::Multi(_, t) => t.clone(),
    };
    let filetimes = FileTimes::new().set_modified(source_time);
    let _ = file_obj.set_times(filetimes);
}

enum Source {
    Single(SystemTime),
    Multi(SystemTime, SystemTime),
}

fn tick(args: &Args, file_path: &PathBuf) -> anyhow::Result<()> {
    let file_obj =
        fs::File::open(file_path).with_context(|| format!("opening file {:?}", file_path))?;

    let src: Source = match (&args.date, &args.time, &args.reference) {
        (Some(date), None, None) => Source::Single(
            dateparser::parse(&date)
                .with_context(|| format!("parsing date string {:?}", &date))?
                .into(),
        ),

        (None, Some(time), None) => Source::Single(
            dateparser::parse(&time)
                .with_context(|| format!("parsing time string {:?}", &time))?
                .into(),
        ),
        (None, None, Some(reference)) => {
            let ref_meta = fs::metadata(reference)?;
            let atime = ref_meta
                .accessed()
                .with_context(|| format!("getting accessed time {:?}", reference))?;
            let mtime = ref_meta
                .modified()
                .with_context(|| format!("getting modified time {:?}", reference))?;
            Source::Multi(atime, mtime)
        }
        (None, None, None) => Source::Single(SystemTime::now()),
        _ => anyhow::bail!("Cannot use -t, -d, or -r at the same time"),
    };

    let filetimes = FileTimes::new();
    match (&args.access, &args.modify_time_only, &args.word) {
        (true, true, _) => {
            set_access(&file_obj, &src);
            set_modified(&file_obj, &src);
        }
        (true, false, None) => set_access(&file_obj, &src),
        (_, false, Some(w)) => {
            on_time(w, src, &file_obj);
        }
        (false, true, None) => set_modified(&file_obj, &src),
        (false, true, Some(w)) => {
            set_modified(&file_obj, &src);
            on_time(w, src, &file_obj);
        }
        (false, false, None) => {
            set_modified(&file_obj, &src);
            set_access(&file_obj, &src);
        }
    }

    let _ = file_obj.set_times(filetimes);

    Ok(())
}

fn on_time(word: &str, src: Source, file_obj: &File) {
    let word_kind = Word::from(word.to_string());
    match word_kind {
        Word::Use => set_access(&file_obj, &src),
        Word::Access => set_access(&file_obj, &src),
        Word::Atime => set_access(&file_obj, &src),
        Word::Modify => set_modified(&file_obj, &src),
        Word::Mtime => set_modified(&file_obj, &src),
    }
}
