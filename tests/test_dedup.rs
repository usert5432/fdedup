extern crate fdedup;
#[macro_use] mod utils;

use std::fs;
use std::os::unix::prelude::*;

use fdedup::args::DedupAction;
use fdedup::dups::dedup::deduplicate;
use utils::{
    create_basic_fs_structure, hardlink_files, copy_files, find_duplicates,
};
use utils::fs_skel::*;

const N_READ : usize = 200;
const ALGO   : crypto_hash::Algorithm = crypto_hash::Algorithm::SHA512;

#[macro_export]
macro_rules! test_dedup {
    ( $dir:expr, $files:expr, $copies:expr, $is_sym:expr ) => {
        for ((file,_size),copies) in $files.iter().zip($copies.iter()) {
            let file_path = $dir.path().join(file);
            let file_meta = fs::metadata(file_path).unwrap();
            let file_inode = file_meta.ino();

            for copy in copies.iter() {
                let copy_path  = $dir.path().join(copy);
                let copy_meta  = fs::metadata(&copy_path).unwrap();
                let copy_inode = copy_meta.ino();

                let link = fs::read_link(copy_path);
                assert_eq!(
                    link.is_ok(), $is_sym,
                    "Deduplicated copy '{}'.is_symlink() != '{}'",
                    copy, $is_sym
                );

                assert_eq!(
                    copy_inode, file_inode,
                    "Deduplicated copy '{}' does not point to the original\
                     file {}", copy, file
                );
            }
        }
    };
}

#[test]
fn test_dedup_hardlinks() {
    let dir = create_basic_fs_structure(&DIRS, &FILES).unwrap();
    copy_files    (&dir, &FILES, &COPIES).unwrap();
    hardlink_files(&dir, &FILES, &LINKS).unwrap();

    let duplicates = find_duplicates(
        &[dir.path().to_str().unwrap()], false, N_READ, ALGO
    ).unwrap();

    deduplicate(
        &duplicates, DedupAction::Hardlink, true, false, false
    ).unwrap();

    test_dedup!(dir, FILES, COPIES, false);

    dir.close().unwrap();
}

#[test]
fn test_dedup_symlinks() {
    let dir = create_basic_fs_structure(&DIRS, &FILES).unwrap();
    copy_files    (&dir, &FILES, &COPIES).unwrap();
    hardlink_files(&dir, &FILES, &LINKS).unwrap();

    let duplicates = find_duplicates(
        &[dir.path().to_str().unwrap()], false, N_READ, ALGO
    ).unwrap();

    deduplicate(
        &duplicates, DedupAction::Symlink, true, false, false
    ).unwrap();

    test_dedup!(dir, FILES, COPIES, true);

    dir.close().unwrap();
}

