use std::str::FromStr;
use clap::{Arg, App, ArgMatches};
use humanize_rs::bytes::Bytes;

use args::{Args, EXCLUDES, DedupAction};

fn is_numeric(s : String) -> Result<(), String>
{
    match Bytes::<u64>::from_str(&s) {
        Ok(_)  => { Ok(()) },
        Err(e) => { Err(format!("Failed to parse number {}: {}", s, e)) },
    }
}

macro_rules! construct_parser {
    () => {
        App::new("File System Deduplicator")
            .version(clap::crate_version!())
            .about(clap::crate_description!())
            .set_term_width(80)
            .arg(Arg::with_name("paths")
                .help("Root path(s) to deduplicate files from")
                .required(true)
                .multiple(true)
                .value_name("PATHS")
            )
            .arg(Arg::with_name("action")
                .short("a")
                .long("--action")
                .possible_values(&["hardlink", "symlink", "print"])
                .help("Sets a custom config file")
                .takes_value(true)
                .default_value("print")
                .value_name("ACTION")
            )
            .arg(Arg::with_name("output")
                .short("o")
                .long("--output")
                .help("File to print found duplicates to")
                .takes_value(true)
                .value_name("OUTPUT")
            )
            .arg(Arg::with_name("include")
                .short("i")
                .long("--include")
                .help("Include patterns")
                .takes_value(true)
                .multiple(true)
                .value_name("INCLUDE")
            )
            .arg(Arg::with_name("exclude")
                .short("e")
                .long("--exclude")
                .help("Exclude patterns")
                .takes_value(true)
                .multiple(true)
                .value_name("EXCLUDE")
            )
            .arg(Arg::with_name("ignore_default_excludes")
                .long("--ignore-default-excludes")
                .help(
                    &format!(
                        "Ignore default exclude patterns: {:?}", EXCLUDES
                    )
                )
            )
            .arg(Arg::with_name("sloppy")
                .long("--sloppy")
                .help("Ignore most I/O errors and skip corresponding files")
            )
            .arg(Arg::with_name("dry_run")
                .long("--dry-run")
                .help(
                    "Do not alter filesystem. \
                    Only log actions that fdedup is going to take"
                )
            )
            .arg(Arg::with_name("no_progress")
                .long("--no-progress")
                .help("Do not show command progress")
            )
            .arg(Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Increment verbosity level")
            )
            .arg(Arg::with_name("q")
                .short("q")
                .multiple(true)
                .help("Decrement verbosity level")
            )
            .arg(Arg::with_name("one_file_system")
                .short("x")
                .long("--one-file-system")
                .help("Do not cross filesystem boundaries")
            )
            .arg(Arg::with_name("hash")
                .long("--hash")
                .possible_values(&["md5", "sha1", "sha256", "sha512" ])
                .help("File hashing function")
                .takes_value(true)
                .default_value("sha512")
                .value_name("HASH")
            )
            .arg(Arg::with_name("n_read")
                .long("--nread")
                .help(
                     "Number of bytes to read from each file for byte \
                      heuristics"
                )
                .takes_value(true)
                .default_value("128")
                .value_name("NREAD")
                .validator(is_numeric)
            )
            .arg(Arg::with_name("min_file_size")
                .long("--min-size")
                .help("Minimum file size to consider")
                .default_value("128")
                .takes_value(true)
                .value_name("MIN_FILE_SIZE")
                .validator(is_numeric)
            )
            .arg(Arg::with_name("max_file_size")
                .long("--max-size")
                .help("Maximum file size to consider")
                .takes_value(true)
                .value_name("MAX_FILE_SIZE")
                .validator(is_numeric)
            )
    };
}

impl Args {

    fn parse_excludes(matches : &ArgMatches) -> Vec<String> {
        let mut result : Vec<String> = Vec::new();

        if ! matches.is_present("ignore_default_excludes") {
            for exclude in EXCLUDES.iter() {
                result.push(exclude.to_string());
            }
        }

        if let Some(excludes) = matches.values_of("exclude") {
            let mut v = excludes.map(|x| { x.to_string() }).collect();
            result.append(&mut v);
        }

        result
    }

    fn parse_includes(matches : &ArgMatches) -> Vec<String> {
        let mut result : Vec<String> = Vec::new();

        if let Some(includes) = matches.values_of("include") {
            result = includes.map(|x| { x.to_string() }).collect();
        }

        result
    }

    #[allow(unused_parens)]
    fn parse_verbosity(matches : &ArgMatches) -> &'static str {
        let n : i64 = (
              (matches.occurrences_of("v") as i64)
            - (matches.occurrences_of("q") as i64)
        );

        if n >= 2 {
            return "trace";
        }
        if n <= -2 {
            return "error";
        }

        match n {
            1  => "debug",
            -1 => "warn",
            _  => "info",
        }
    }

    pub fn parse() -> Self {
        let matches = construct_parser!().get_matches();

        let paths : Vec<String> = matches.values_of("paths").unwrap()
            .map(|x| { x.to_string() }).collect();

        let action : DedupAction = DedupAction::from_str(
            matches.value_of("action").unwrap()
        ).unwrap();

        let result_path : Option<String>
            = matches.value_of("output").map(|x| { x.to_string() });

        let excludes = Self::parse_excludes(&matches);
        let includes = Self::parse_includes(&matches);

        let abort_on_error  : bool = ! matches.is_present("sloppy");
        let show_progress   : bool = ! matches.is_present("no_progress");
        let one_file_system : bool = matches.is_present("one_file_system");
        let dry_run         : bool = matches.is_present("dry_run");

        let verbosity = Args::parse_verbosity(&matches).to_string();
        let hash      = matches.value_of("hash").unwrap().to_string();

        let n_read : usize = usize::from_str(
            matches.value_of("n_read").unwrap()
        ).unwrap();

        let min_file_size : Option<u64> = matches.value_of("min_file_size")
            .map( |x| Bytes::from_str(x).unwrap().size() );

        let max_file_size : Option<u64> = matches.value_of("max_file_size")
            .map( |x| Bytes::from_str(x).unwrap().size() );

        Args {
            paths, action, result_path, includes, excludes, abort_on_error,
            show_progress, verbosity, one_file_system, hash, n_read,
            min_file_size, max_file_size, dry_run
        }
    }

}

