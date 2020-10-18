extern crate clap;
extern crate crypto_hash;
extern crate env_logger;
extern crate globset;
extern crate humanize_rs;
extern crate indicatif;
extern crate fastrand;
#[macro_use] extern crate log;

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};

use crypto_hash::Algorithm;
use env_logger::Env;
use indicatif::HumanBytes;
use log::{info, Level};

#[macro_use] mod err_macro;
pub mod args;
pub mod fs_entry;
pub mod dups;
pub mod utils;

use args::{Args, DedupAction};
use fs_entry::{FSEntry, Dev, INode, Priority, collect_files};
use dups::search::{
    remove_unique_entries_by_heuristic_fn, remove_unique_entries_by_heuristic,
    group_by_heuristic
};
use dups::heuristics::{
    HeuristicFn, fn_first_bytes, fn_last_bytes, fn_file_hash
};
use dups::dedup::deduplicate;

fn collect_all_files(args : &Args) -> io::Result<Vec<FSEntry>> {
    let mut files_map : HashMap<(Dev, INode), FSEntry> = HashMap::new();

    for (idx,path) in args.paths.iter().enumerate() {
        info!("Scanning '{}' for entries...", path);
        collect_files(
            &path, &mut files_map, args.abort_on_error, args.show_progress,
            args.one_file_system, &args.includes, &args.excludes,
            args.min_file_size, args.max_file_size, idx as Priority
        )?;
    }

    Ok(files_map.into_iter().map( | (_, v) | { v } ).collect())
}

fn log_possible_duplicates(entries : &[FSEntry], name : &str) {
    if log_enabled!(Level::Info) {
        let n_entries = entries.len();
        let n_files : usize = entries.iter().map( |x| x.paths.len() ).sum();

        info!(
            "Possibly identical files after grouping by {}: {} (inodes: {}).",
            name, n_files, n_entries
        );
    }

    trace!("Entries: {:#?}", entries);
}

fn remove_unique_by_fn_wrap(
    entries : Vec<FSEntry>, args : &Args, name : &str, func : Box<HeuristicFn>
) -> io::Result<Vec<FSEntry>>
{
    info!("Grouping entries by {}", name);

    let result = remove_unique_entries_by_heuristic_fn(
        entries,
        args.action == DedupAction::Hardlink,
        format!("Grouping by {}", name).as_str(),
        args.show_progress,
        args.abort_on_error,
        func
    );

    match &result {
        Ok(r)  => { log_possible_duplicates(r, name); },
        Err(e) => { warn!("Grouping by {} failed: {}", name, e); },
    }

    result
}

fn remove_unique_files_by_size(entries : Vec<FSEntry>, args : &Args)
    -> Vec<FSEntry>
{
    let result = remove_unique_entries_by_heuristic(
        entries, args.action == DedupAction::Hardlink
    );

    log_possible_duplicates(&result, "file size");

    result
}

fn remove_unique_files_by_bytes(entries : Vec<FSEntry>, args : &Args)
    -> io::Result<Vec<FSEntry>>
{
    let n_read = args.n_read;

    let result = remove_unique_by_fn_wrap(
        entries, args, "first bytes",
        Box::new(move | entries | { fn_first_bytes(entries, n_read) })
    )?;

    remove_unique_by_fn_wrap(
        result, args, "last bytes",
        Box::new(move | entries | { fn_last_bytes(entries, n_read) })
    )
}

pub fn remove_unique_files(entries : Vec<FSEntry>, args : &Args)
    -> io::Result<Vec<FSEntry>>
{
    info!("Grouping entries by size");
    let mut result = remove_unique_files_by_size(entries, args);

    if args.n_read > 0 {
        result = remove_unique_files_by_bytes(result, args)?;
    }

    let algo = algo_from_str(&args.hash)?;
    remove_unique_by_fn_wrap(
        result, args, &format!("hash ({})", str::to_uppercase(&args.hash)),
        Box::new(move | entries | { fn_file_hash(entries, algo) })
    )
}

fn setup_logging(args : &Args)
{
    let mut builder = env_logger::Builder::from_env(
        Env::default().default_filter_or(&args.verbosity)
    );

    builder.default_format().format_module_path(false);
    builder.init()
}

fn print_initial_stats(entries : &[FSEntry])
{
    if log_enabled!(Level::Info) {
        let n_inodes = entries.len();
        let n_files : usize = entries.iter().map( |x| x.paths.len() ).sum();
        info!("Found {} entries (inodes: {})", n_files, n_inodes);
    }
}

fn algo_from_str(s : &str) -> io::Result<Algorithm>
{
    match s {
        "md5"    => Ok(Algorithm::MD5),
        "sha1"   => Ok(Algorithm::SHA1),
        "sha256" => Ok(Algorithm::SHA256),
        "sha512" => Ok(Algorithm::SHA512),
        _        => Err(io::Error::new(
            io::ErrorKind::Other, format!("Cannot parse alogorithm: {}", s)
        )),
    }
}

fn print_final_stats(duplicate_groups : &[Vec<FSEntry>]) {

    if ! log_enabled!(Level::Info) {
        return;
    }

    let mut n_dupl_inodes : usize = 0;
    let mut n_dupl_files  : usize = 0;
    let mut n_inodes      : usize = 0;
    let mut saved_size    : u64   = 0;

    for group in duplicate_groups.iter() {
        let group_size : usize = group.len();

        n_inodes      += group_size;
        n_dupl_inodes += group_size - 1;
        n_dupl_files  += group.iter().skip(1).map(
            |x| x.paths.len()
        ).sum::<usize>();

        saved_size += ((group_size - 1) as u64) * group[0].size;
    }

    let mult = (n_inodes as f32) / ((n_inodes - n_dupl_inodes) as f32);

    info!(
        "Found {} duplicate files (inodes: {}). Avr. Mult: {:.2}",
        n_dupl_files, n_dupl_inodes, mult
    );
    info!("Deduplication will save {}", HumanBytes(saved_size));
}

fn write_path_to_results_file(file : &mut File, path : &str) -> io::Result<()>
{
    if path.contains('\n') {
        writeln!(
            file, "    \\\"{}\"",
            path.replace("\\", "\\\\").replace("\n", "\\n")

        )?;
    }
    else {
        writeln!(file, "    {}", path)?;
    }

    Ok(())
}

fn print_results_file(
    duplicate_groups : &[Vec<FSEntry>], path : &Option<String>
) -> io::Result<()>
{
    if duplicate_groups.is_empty() || path.is_none() {
        return Ok(());
    }

    let path : String = path.as_ref().unwrap().clone();
    info!("Saving duplicate entries to {}", path);

    let mut file = File::create(&path)?;

    for group in duplicate_groups.iter() {
        let head = &group[0];
        writeln!(file, "Identical Files. Size: {}", head.size)?;

        for entry in group.iter() {
            writeln!(file, "  {} {}", entry.dev, entry.inode)?;

            for path in entry.paths.iter() {
                write_path_to_results_file(&mut file, path)?;
            }
        }
    }

    Ok(())
}

pub fn run() -> io::Result<()>
{
    let args = Args::parse();
    setup_logging(&args);

    if (args.action == DedupAction::Print) && args.result_path.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "No output file specified for action 'print'".to_string()
        ));
    }

    let entries : Vec<FSEntry> = collect_all_files(&args)?;
    print_initial_stats(&entries);

    let entries = remove_unique_files(entries, &args)?;
    let mut duplicate_groups = group_by_heuristic(
        entries, args.action == DedupAction::Hardlink
    );

    duplicate_groups.sort_by( |a, b| a[0].size.cmp(&b[0].size) );

    print_final_stats(&duplicate_groups);
    print_results_file(&duplicate_groups, &args.result_path)?;

    deduplicate(
        &duplicate_groups, args.action, args.abort_on_error,
        args.dry_run, args.show_progress
    )?;

    Ok(())
}

