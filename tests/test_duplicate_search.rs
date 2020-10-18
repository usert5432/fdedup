extern crate fdedup;
#[macro_use] mod utils;

use std::cmp::Ordering;

use fdedup::fs_entry::FSEntry;
use utils::{
    create_basic_fs_structure, hardlink_files, copy_files, find_duplicates,
    create_null_entry
};
use utils::fs_skel::*;

const N_READ : usize = 200;
const ALGO   : crypto_hash::Algorithm = crypto_hash::Algorithm::SHA512;

#[macro_export]
macro_rules! compare_duplicates {
    ( $test:expr, $null:expr ) => {
        assert_eq!(
            $test.len(), $null.len(),
            "Actual and expected number of duplicate groups differ.\
             \nActual: {:?}\nExpected: {:?}", $test, $null
        );

        sort_duplicates(&mut $test);
        sort_duplicates(&mut $null);

        for (t_group,n_group) in $test.iter().zip($null.iter()) {
            assert_eq!(
                t_group.len(), n_group.len(),
                "Actual and expected number of duplicates in group differ.\
                 \nActual group: {:?}\nExpected group: {:?}", t_group, n_group
            );
            for (t,n) in t_group.iter().zip(n_group.iter()) {
                assert_eq!(
                    t.size, n.size,
                    "Actual and expected duplicate fs entry sizes differ. \
                     \nActual entry: {:?}\nExpected entry: {:?}", t, n
                );
                assert_eq!(
                    t.paths, n.paths,
                    "Actual and expected duplicate fs entry sizes differ. \
                     \nActual entry: {:?}\nExpected entry: {:?}", t, n
                );
            }
        }
    };
}

pub fn sort_duplicates(duplicate_groups : &mut Vec<Vec<FSEntry>>) {
    for group in duplicate_groups.iter_mut() {
        for entry in group.iter_mut() {
            entry.paths.sort();
        }

        group.sort_by(
            |a, b| { a.paths[0].cmp(&b.paths[0]) }
        );
    }

    duplicate_groups.sort_by(
        |a, b| {
            match a[0].size.cmp(&b[0].size) {
                Ordering::Equal => a[0].paths[0].cmp(&b[0].paths[0]),
                other           => other,
            }
        }
    );
}

pub fn calculate_null_duplicates(
    dir    : &tempfile::TempDir,
    files  : &[(&str, u64)],
    links  : Option<&[&[&str]]>,
    copies : Option<&[&[&str]]>,
) -> Vec<Vec<FSEntry>>
{
    let mut result : Vec<Vec<FSEntry>> = Vec::new();

    for (idx,(file,size)) in files.iter().enumerate() {
        let mut group : Vec<FSEntry> = Vec::new();

        group.push(create_null_entry(
            dir, file, *size, links.map( |x| x[idx] )
        ));

        if let Some(file_copies) = copies {
            for copy in file_copies[idx].iter() {
                group.push(create_null_entry(dir, copy, *size, None));
            }
        }

        result.push(group)
    }

    result
}

#[test]
fn test_duplicate_search_simple() {
    let files = &FILES[..1];
    let dir   = create_basic_fs_structure(&DIRS, files).unwrap();

    let test_duplicates = find_duplicates(
        &[dir.path().to_str().unwrap()], false, N_READ, ALGO
    ).unwrap();

    let null_duplicates : Vec<Vec<FSEntry>> = Vec::new();

    assert_eq!(test_duplicates, null_duplicates);

    dir.close().unwrap();
}

#[test]
fn test_duplicate_search_basic_no_copies() {
    let dir = create_basic_fs_structure(&DIRS, &FILES).unwrap();

    let test_duplicates = find_duplicates(
        &[dir.path().to_str().unwrap()], false, N_READ, ALGO
    ).unwrap();

    let null_duplicates : Vec<Vec<FSEntry>> = Vec::new();

    assert_eq!(test_duplicates, null_duplicates);

    dir.close().unwrap();
}

#[test]
fn test_duplicate_search_with_hardlinks_no_copies() {
    let dir = create_basic_fs_structure(&DIRS, &FILES).unwrap();
    hardlink_files(&dir, &FILES, &COPIES).unwrap();

    let test_duplicates = find_duplicates(
        &[dir.path().to_str().unwrap()], false, N_READ, ALGO
    ).unwrap();

    let null_duplicates : Vec<Vec<FSEntry>> = Vec::new();

    assert_eq!(test_duplicates, null_duplicates);

    dir.close().unwrap();
}

#[test]
fn test_duplicate_search_with_copies() {
    let dir = create_basic_fs_structure(&DIRS, &FILES).unwrap();
    copy_files(&dir, &FILES, &COPIES).unwrap();

    let mut test_duplicates = find_duplicates(
        &[dir.path().to_str().unwrap()], false, N_READ, ALGO
    ).unwrap();

    let mut null_duplicates = calculate_null_duplicates(
        &dir, &FILES, None, Some(&COPIES)
    );

    compare_duplicates!(test_duplicates, null_duplicates);

    dir.close().unwrap();
}

#[test]
fn test_duplicate_search_complex() {
    let dir = create_basic_fs_structure(&DIRS, &FILES).unwrap();
    copy_files    (&dir, &FILES, &COPIES).unwrap();
    hardlink_files(&dir, &FILES, &LINKS).unwrap();

    let mut test_duplicates = find_duplicates(
        &[dir.path().to_str().unwrap()], false, N_READ, ALGO
    ).unwrap();

    let mut null_duplicates = calculate_null_duplicates(
        &dir, &FILES, Some(&LINKS), Some(&COPIES)
    );

    compare_duplicates!(test_duplicates, null_duplicates);

    dir.close().unwrap();
}

#[test]
fn test_duplicate_search_complex_with_value_compression() {
    let dir = create_basic_fs_structure(&DIRS, &FILES).unwrap();
    copy_files    (&dir, &FILES, &COPIES).unwrap();
    hardlink_files(&dir, &FILES, &LINKS).unwrap();

    let mut test_duplicates = find_duplicates(
        &[dir.path().to_str().unwrap()], false, N_READ, ALGO
    ).unwrap();

    let mut null_duplicates = calculate_null_duplicates(
        &dir, &FILES, Some(&LINKS), Some(&COPIES)
    );

    compare_duplicates!(test_duplicates, null_duplicates);

    dir.close().unwrap();
}

#[test]
fn test_duplicate_search_complex_without_bytes() {
    let dir = create_basic_fs_structure(&DIRS, &FILES).unwrap();
    copy_files    (&dir, &FILES, &COPIES).unwrap();
    hardlink_files(&dir, &FILES, &LINKS).unwrap();

    let mut test_duplicates = find_duplicates(
        &[dir.path().to_str().unwrap()], false, 0, ALGO
    ).unwrap();

    let mut null_duplicates = calculate_null_duplicates(
        &dir, &FILES, Some(&LINKS), Some(&COPIES)
    );

    compare_duplicates!(test_duplicates, null_duplicates);

    dir.close().unwrap();
}

#[test]
fn test_duplicate_search_complex_with_dev() {
    let dir = create_basic_fs_structure(&DIRS, &FILES).unwrap();
    copy_files    (&dir, &FILES, &COPIES).unwrap();
    hardlink_files(&dir, &FILES, &LINKS).unwrap();

    let mut test_duplicates = find_duplicates(
        &[dir.path().to_str().unwrap()], true, N_READ, ALGO
    ).unwrap();

    let mut null_duplicates = calculate_null_duplicates(
        &dir, &FILES, Some(&LINKS), Some(&COPIES)
    );

    compare_duplicates!(test_duplicates, null_duplicates);

    dir.close().unwrap();
}

