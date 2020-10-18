use std::collections::HashMap;
use std::fs::{self, DirEntry, Metadata};
use std::io;
use std::os::unix::prelude::*;
use std::path::Path;

use log::warn;

use fs_entry::{FSEntry, Dev, INode, Priority};
use fs_entry::search_state::SearchState;

fn add_path_to_entry(
    path     : &Path,
    meta     : &Metadata,
    priority : Priority,
    files    : &mut HashMap<(Dev, INode), FSEntry>,
)
{
    let inode : INode  = meta.ino();
    let dev   : Dev    = meta.dev();
    let path  : String = path.to_str().unwrap().to_string();

    match files.get_mut(&(dev, inode)) {
        Some(fs_entry) => fs_entry.add_path(path),
        None           => {
            let size : u64 = meta.size();
            files.insert(
                (dev, inode),
                FSEntry::new(dev, inode, size, priority, path)
            );
        },
    }
}

fn check_dir_entry(
    entry : &DirEntry,
    meta  : &Metadata,
    state : &mut SearchState,
) -> bool
{
    if meta.file_type().is_symlink() {
        return false;
    }

    if let Some(d) = state.dev {
        if meta.dev() != d {
            return false;
        }
    }

    let path = entry.path();

    if ! state.passes_filters(&path, meta) {
        return false;
    }

    state.tick(&path);

    true
}

fn recurse_into_directory(
    path  : &Path,
    files : &mut HashMap<(Dev, INode), FSEntry>,
    state : &mut SearchState,
) -> io::Result<()>
{
    if ! path.is_dir() {
        return Ok(());
    }

    for dir_entry in verbose_question_mark!(
        fs::read_dir(path),
        state, format!("Failed to read directory: {}", path.display())
    )
    {
        let entry = sloppy_unwrap_or_continue!(
            dir_entry, state,
            format!("Failed to read directory entry in: {}", path.display())
        );

        let entry_path = entry.path();

        let meta  = sloppy_unwrap_or_continue!(
            entry.metadata(),
            state, format!("Failed to stat entry: {}", entry_path.display())
        );

        if ! check_dir_entry(&entry, &meta, state) {
            continue;
        }

        if entry_path.is_file() {
            add_path_to_entry(&entry_path, &meta, state.priority, files);
        }
        else if entry_path.is_dir() {
            sloppy_unwrap_or_continue!(
                recurse_into_directory(&entry_path, files, state), state, ""
            );
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn collect_files(
    root           : &str,
    files_map      : &mut HashMap<(Dev, INode), FSEntry>,
    abort_on_error : bool,
    verbose        : bool,
    one_fs         : bool,
    inc_patterns   : &[String],
    ex_patterns    : &[String],
    min_file_size  : Option<u64>,
    max_file_size  : Option<u64>,
    priority       : Priority,
) -> io::Result<()>
{
    let path  = Path::new(root);
    let mut state = SearchState::new(
        path, abort_on_error, verbose, one_fs, inc_patterns, ex_patterns,
        min_file_size, max_file_size, priority
    )?;

    let result = recurse_into_directory(
        Path::new(root), files_map, &mut state
    );

    state.finish();

    if abort_on_error {
        return result;
    }

    Ok(())
}


