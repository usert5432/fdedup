use std::io;
use std::fs;

use args::DedupAction;
use fs_entry::FSEntry;
use utils::progress::get_progress_bar;
use utils::path::calculate_relative_path;

pub struct DedupState {
    pub action         : DedupAction,
    pub abort_on_error : bool,
    pub dry_run        : bool,
}

macro_rules! handle_dry_run {
    ( $e:expr, $state:expr, $($msg:expr),+ ) => {
        if $state.dry_run {
            println!($($msg),+);
        }
        else {
            debug!($($msg),+);
            $e
        }
    };
}

fn find_group_leader_index(group : &[FSEntry]) -> usize
{
    let lowest_priority =
        group.iter().map(|entry| entry.priority).min().unwrap();

    let result = group.iter()
        .enumerate()
        .filter(|(  _, entry)| { entry.priority == lowest_priority })
        .map(   |(idx, entry)| { (idx, entry.paths.len()) })
        .max_by(|a, b| { a.1.cmp(&b.1) });

    result.unwrap().0
}

fn deduplicate_file(src : &str, dst : &str, state : &DedupState)
    -> io::Result<()>
{
    match state.action {
        DedupAction::Symlink  => {
            let rel_src = calculate_relative_path(src, dst);
            handle_dry_run!(
                verbose_question_mark!(
                    std::os::unix::fs::symlink(&rel_src, dst), state,
                    format!("Failed to make symlink: {} -> {}", dst, rel_src)
                ),
                state, "  ln -s '{}' '{}'", rel_src, dst
            );
        },
        DedupAction::Hardlink => {
            handle_dry_run!(
                verbose_question_mark!(
                    fs::hard_link(&src, dst), state,
                    format!("Failed to make hardlink: {} -> {}", dst, src)
                ),
                state, "  ln '{}' '{}'", src, dst
            );
        },
        _ => {},
    };

    Ok(())
}

fn deduplicate_group(group : &[FSEntry], state : &DedupState)
    -> io::Result<()>
{
    let leader_index = find_group_leader_index(group);
    let leader_path  = &group[leader_index].paths[0];

    for (idx, entry) in group.iter().enumerate() {

        if idx == leader_index {
            continue;
        }

        for path in entry.paths.iter() {
            handle_dry_run!(
                sloppy_unwrap_or_continue!(
                    fs::remove_file(path), state,
                    format!("Failed to remove file {}", path)
                ),
                state, "  rm '{}'", path
            );

            deduplicate_file(leader_path, path, state)?;
        }
    }

    Ok(())
}

pub fn deduplicate(
    duplicate_groups : &[Vec<FSEntry>],
    action           : DedupAction,
    abort_on_error   : bool,
    dry_run          : bool,
    verbose          : bool,
) -> io::Result<()>
{
    if (action == DedupAction::Print) || duplicate_groups.is_empty() {
        return Ok(());
    }

    let state = DedupState{ action, abort_on_error, dry_run };
    let pbar  = get_progress_bar(
        duplicate_groups.len() as u64, "Deduplicating", verbose && (! dry_run)
    );

    if dry_run {
        println!("Dry run:")
    }

    for (group_index,group) in duplicate_groups.iter().enumerate() {
        if dry_run {
            println!("[{}]", group_index);
        }

        sloppy_unwrap_or_continue!(
            deduplicate_group(group, &state), state, ""
        );

        if let Some(x) = pbar.as_ref().as_mut() { x.inc(1) };
    }

    if let Some(x) = pbar { x.abandon() }

    Ok(())
}

