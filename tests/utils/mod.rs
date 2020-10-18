extern crate fdedup;
extern crate crypto_hash;
extern crate fastrand;
extern crate tempfile;

pub mod fs_skel;

use std::collections::HashMap;
use std::fs::{File, create_dir_all};
use std::io::{self, Write};
use std::iter::repeat_with;
use std::path::Path;

use fdedup::fs_entry::{FSEntry, INode, Dev, collect_files};
use fdedup::dups::heuristics::{
    fn_first_bytes, fn_last_bytes, fn_file_hash
};
use fdedup::dups::search::{
    group_by_heuristic, remove_unique_entries_by_heuristic_fn
};

pub fn create_random_file(path : &Path, size : u64) -> io::Result<()> {
    let mut f = File::create(path)?;

    let rng   = fastrand::Rng::new();
    let bytes : Vec<u8> =
        repeat_with(|| rng.u8(..)).take(size as usize).collect();

    f.write_all(&bytes)?;
    f.sync_all()?;

    Ok(())
}

pub fn collect_all_files(paths : &[&str]) -> io::Result<Vec<FSEntry>>{
    let mut files_map : HashMap<(Dev, INode), FSEntry> = HashMap::new();

    for (idx, path) in paths.iter().enumerate() {
        collect_files(
            path, &mut files_map, true, false, false,
            &Vec::new(), &Vec::new(), None, None, idx as u32
        )?;
    }

    let result : Vec<FSEntry> =
        files_map.into_iter().map( | (_, v) | { v } ).collect();

    Ok(result)
}

#[allow(dead_code)]
pub fn find_duplicates(
    paths : &[&str], cmp_dev : bool, n_read : usize,
    hash : crypto_hash::Algorithm
) -> io::Result<Vec<Vec<FSEntry>>>
{
    let mut result = collect_all_files(paths)?;

    if n_read > 0 {
        result = remove_unique_entries_by_heuristic_fn(
            result, cmp_dev, "first bytes", false, false,
            Box::new(move | entry | { fn_first_bytes(entry, n_read) })
        )?;

        result = remove_unique_entries_by_heuristic_fn(
            result, cmp_dev, "last bytes", false, false,
            Box::new(move | entry | { fn_last_bytes(entry, n_read) })
        )?;
    }

    result = remove_unique_entries_by_heuristic_fn(
        result, cmp_dev, "last bytes", false, false,
        Box::new(move | entry | { fn_file_hash(entry, hash) })
    )?;

    Ok(group_by_heuristic(result, cmp_dev))
}

pub fn create_basic_fs_structure(
    dirs  : &[&str], files : &[(&str, u64)]
) -> io::Result<tempfile::TempDir>
{
    let dir       = tempfile::tempdir()?;
    let root_path = dir.path();

    for dir in dirs.iter() {
        create_dir_all(root_path.join(dir))?;
    }

    for (fname,size) in files.iter() {
        create_random_file(&root_path.join(fname), *size)?;
    }

    Ok(dir)
}

pub fn copy_files(
    tempdir : &tempfile::TempDir,
    files   : &[(&str, u64)],
    copies  : &[&[&str]],
) -> io::Result<()>
{
    let root_path = tempdir.path();

    for ((src,_size),dst_list) in files.iter().zip(copies.iter()) {
        for dst in dst_list.iter() {
            std::fs::copy(root_path.join(src), root_path.join(dst))?;
        }
    }

    Ok(())
}

pub fn hardlink_files(
    tempdir : &tempfile::TempDir,
    files   : &[(&str, u64)],
    copies  : &[&[&str]],
) -> io::Result<()>
{
    let root_path = tempdir.path();

    for ((src,_size),dst_list) in files.iter().zip(copies.iter()) {
        for dst in dst_list.iter() {
            std::fs::hard_link(root_path.join(src), root_path.join(dst))?;
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub fn create_null_entry(
    dir   : &tempfile::TempDir,
    file  : &str,
    size  : u64,
    links : Option<&[&str]>,
) -> FSEntry {
    let mut result = FSEntry::new(
        0, 0, size, 0,
        dir.path().join(file).to_str().unwrap().to_string()
    );

    if let Some(file_links) = links {
        for link in file_links.iter() {
            result.add_path(
                dir.path().join(link).to_str().unwrap().to_string()
            );
        }
    }

    result.paths.sort();

    result
}

