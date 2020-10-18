extern crate fdedup;
#[macro_use] mod utils;

use std::cmp::Ordering;
use fdedup::fs_entry::FSEntry;
use utils::{
    collect_all_files, create_basic_fs_structure, hardlink_files, copy_files,
    create_null_entry
};

use utils::fs_skel::*;

#[macro_export]
macro_rules! compare_entries {
    ( $test:expr, $null:expr ) => {
        assert_eq!(
            $test.len(), $null.len(),
            "Sizes of actual and expected fs entry lists differ.\"
             \nActual: {:?}\nExpected: {:?}", $test, $null
        );

        sort_entries(&mut $test);
        sort_entries(&mut $null);

        for (t,n) in $test.iter().zip($null.iter()) {
            assert_eq!(
                t.size,  n.size,
                "Actual and expected fs entries have different sizes.\
                 \nActual entry: {:?}\nExpected entry: {:?}", t, n
            );
            assert_eq!(
                t.paths, n.paths,
                "Actual and expected fs entries have different paths.\
                 \nActual entry: {:?}\nExpected entry: {:?}", t, n
            );
        }
    };
}

pub fn sort_entries(entries : &mut Vec<FSEntry>) {
    for entry in entries.iter_mut() {
        entry.paths.sort();
    }

    entries.sort_by(
        |a, b| {
            match a.size.cmp(&b.size) {
                Ordering::Equal => a.paths[0].cmp(&b.paths[0]),
                other           => other,
            }
        }
    );
}

pub fn create_null_entries(
    dir    : &tempfile::TempDir,
    files  : &[(&str, u64)],
    links  : Option<&[&[&str]]>,
    copies : Option<&[&[&str]]>,
) -> Vec<FSEntry>
{
    let mut result : Vec<FSEntry> = Vec::new();

    for (idx,(file,size)) in files.iter().enumerate() {
        result.push(create_null_entry(
            dir, file, *size, links.map( |x| x[idx] )
        ));

        if let Some(file_copies) = copies {
            for copy in file_copies[idx].iter() {
                result.push(create_null_entry(dir, copy, *size, None));
            }
        }
    }

    result
}

#[test]
fn test_file_collection_single_file() {
    let files = &FILES[..1];
    let dir   = create_basic_fs_structure(&DIRS, files).unwrap();

    let mut test_entries =
        collect_all_files(&[dir.path().to_str().unwrap()]).unwrap();

    let mut null_entries = create_null_entries(&dir, files, None, None);

    compare_entries!(test_entries, null_entries);

    dir.close().unwrap();
}

#[test]
fn test_file_collection_multiple_files() {
    let dir = create_basic_fs_structure(&DIRS, &FILES).unwrap();

    let mut test_entries =
        collect_all_files(&[dir.path().to_str().unwrap()]).unwrap();

    let mut null_entries = create_null_entries(&dir, &FILES, None, None);

    compare_entries!(test_entries, null_entries);

    dir.close().unwrap();
}

#[test]
fn test_file_collection_repeated_search() {
    let dir = create_basic_fs_structure(&DIRS, &FILES).unwrap();

    let mut test_entries = collect_all_files(&[
        dir.path().to_str().unwrap(),
        dir.path().to_str().unwrap(),
        dir.path().to_str().unwrap(),
    ]).unwrap();

    let mut null_entries = create_null_entries(&dir, &FILES, None, None);

    compare_entries!(test_entries, null_entries);

    dir.close().unwrap();
}

#[test]
fn test_file_collection_multiple_files_with_hardlinks() {

    let dir = create_basic_fs_structure(&DIRS, &FILES).unwrap();
    hardlink_files(&dir, &FILES, &COPIES).unwrap();

    let mut test_entries =
        collect_all_files(&[dir.path().to_str().unwrap()]).unwrap();

    let mut null_entries =
        create_null_entries(&dir, &FILES, Some(&COPIES), None);

    compare_entries!(test_entries, null_entries);

    dir.close().unwrap();
}

#[test]
fn test_file_collection_multiple_files_with_copies() {
    let dir = create_basic_fs_structure(&DIRS, &FILES).unwrap();
    copy_files(&dir, &FILES, &COPIES).unwrap();

    let mut test_entries =
        collect_all_files(&[dir.path().to_str().unwrap()]).unwrap();

    let mut null_entries =
        create_null_entries(&dir, &FILES, None, Some(&COPIES));

    compare_entries!(test_entries, null_entries);

    dir.close().unwrap();
}

#[test]
fn test_file_collection_complex() {
    let dir = create_basic_fs_structure(&DIRS, &FILES).unwrap();
    hardlink_files(&dir, &FILES, &LINKS).unwrap();
    copy_files    (&dir, &FILES, &COPIES).unwrap();

    let mut test_entries =
        collect_all_files(&[dir.path().to_str().unwrap()]).unwrap();

    let mut null_entries =
        create_null_entries(&dir, &FILES, Some(&LINKS), Some(&COPIES));

    compare_entries!(test_entries, null_entries);

    dir.close().unwrap();
}

