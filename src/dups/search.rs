use std::cmp::Ordering;
use std::io;

use fs_entry::FSEntry;
use dups::heuristics::HeuristicFn;
use dups::eval::Evaluator;

pub fn compare_entries(a : &FSEntry, b : &FSEntry, cmp_dev : bool) -> Ordering
{
    if cmp_dev {
        let result = a.dev.cmp(&b.dev);
        if result != Ordering::Equal {
            return result;
        }
    }

    match a.size.cmp(&b.size) {
        Ordering::Equal => a.hvalue.cmp(&b.hvalue),
        other           => other,
    }
}

fn add_entry_to_list_if_nontrivial(
    entries : &mut Vec<FSEntry>, prev_entry : FSEntry, cmp_dev : bool
)
{
    if let Some(last_entry) = entries.last() {
        if compare_entries(&prev_entry, last_entry, cmp_dev) == Ordering::Equal
        {
            entries.push(prev_entry);
        }
    }
}

pub fn remove_unique_entries_by_heuristic(
    mut entries : Vec<FSEntry>, cmp_dev : bool
) -> Vec<FSEntry>
{
    if entries.is_empty() {
        return entries;
    }

    entries.sort_by( | a, b | compare_entries(a, b, cmp_dev) );

    let mut result : Vec<FSEntry> = Vec::new();
    let mut prev_entry : FSEntry = entries[0].clone();

    for entry in entries.into_iter().skip(1) {
        if compare_entries(&prev_entry, &entry, cmp_dev) == Ordering::Equal {
            result.push(prev_entry);
        }
        else {
            add_entry_to_list_if_nontrivial(&mut result, prev_entry, cmp_dev);
        }

        prev_entry = entry;
    }

    add_entry_to_list_if_nontrivial(&mut result, prev_entry, cmp_dev);

    result
}

pub fn remove_unique_entries_by_heuristic_fn(
    mut entries     : Vec<FSEntry>,
    cmp_dev         : bool,
    title           : &str,
    verbose         : bool,
    abort_on_error  : bool,
    func            : Box<HeuristicFn>
) -> io::Result<Vec<FSEntry>>
{
    let mut eval = Evaluator::new(
        entries.len(), title, verbose, abort_on_error, func
    );

    eval.evaluate(&mut entries)?;

    Ok(remove_unique_entries_by_heuristic(entries, cmp_dev))
}

pub fn count_duplicate_entries_files(
    entries : &[FSEntry], cmp_dev : bool
) -> (usize, usize)
{
    if entries.is_empty() {
        return (0, 0);
    }

    let mut n_entries : usize = 0;
    let mut n_files   : usize = 0;

    let mut prev_entry : &FSEntry = &entries[0];

    for entry in entries.iter().skip(1) {
        if compare_entries(prev_entry, entry, cmp_dev) == Ordering::Equal {
            n_entries += 1;
            n_files   += entry.paths.len();
        }

        prev_entry = entry;
    }

    (n_entries, n_files)
}

pub fn group_by_heuristic(
    mut entries : Vec<FSEntry>, cmp_dev : bool
) -> Vec<Vec<FSEntry>>
{
    if entries.is_empty() {
        return Vec::new();
    }

    // TODO: replace by is_sorted once it is stabilized
    entries.sort_by( | a, b | compare_entries(a, b, cmp_dev) );

    let mut result : Vec<Vec<FSEntry>> = Vec::new();
    let mut group  : Vec<FSEntry>      = vec![ entries[0].clone() ];

    for entry in entries.into_iter().skip(1) {
        #[allow(unused_parens)]
        if (
               compare_entries(group.last().unwrap(), &entry, cmp_dev)
            == Ordering::Equal
        ) {
            group.push(entry);
        }
        else {
            result.push(group);
            group = vec![ entry ];
        }
    }

    result.push(group);

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use fs_entry::{Heuristic, INode};

    fn init_test_entry(
        inode : INode, size : u64, hvalue : Heuristic
    ) -> FSEntry
    {
        let mut result = FSEntry::new(0, inode, size, 0, "path".to_string());
        result.hvalue  = hvalue;

        result
    }

    #[test]
    pub fn test_remove_unique_entries_single_collapse()
    {
        let entry = init_test_entry(0, 0, Heuristic::Null);

        let test_entries = remove_unique_entries_by_heuristic(
            vec![ entry ], false
        );
        let null_entries : Vec<FSEntry> = Vec::new();

        assert_eq!(test_entries, null_entries);
    }

    #[test]
    pub fn test_remove_unique_entries_no_unique()
    {
        let entries = vec![
            init_test_entry(0, 0, Heuristic::Null),
            init_test_entry(1, 0, Heuristic::Null),
            init_test_entry(2, 0, Heuristic::Null),
        ];

        let null_entries = entries.clone();
        let test_entries = remove_unique_entries_by_heuristic(entries, false);

        assert_eq!(test_entries, null_entries);
    }

    #[test]
    pub fn test_remove_unique_entries_by_size()
    {
        let entries = vec![
            init_test_entry(0, 0, Heuristic::Null),
            init_test_entry(1, 2, Heuristic::Null),
            init_test_entry(2, 1, Heuristic::Null),
            init_test_entry(3, 2, Heuristic::Null),
            init_test_entry(4, 0, Heuristic::Null),
        ];

        let null_entries = vec![
            init_test_entry(0, 0, Heuristic::Null),
            init_test_entry(4, 0, Heuristic::Null),
            init_test_entry(1, 2, Heuristic::Null),
            init_test_entry(3, 2, Heuristic::Null),
        ];

        let test_entries = remove_unique_entries_by_heuristic(entries, false);

        assert_eq!(test_entries, null_entries);
    }

    #[test]
    pub fn test_remove_unique_entries_by_hvalue()
    {
        let entries = vec![
            init_test_entry(0, 0, Heuristic::Size(1)),
            init_test_entry(1, 0, Heuristic::Size(0)),
            init_test_entry(2, 0, Heuristic::Size(2)),
            init_test_entry(3, 0, Heuristic::Size(1)),
            init_test_entry(4, 0, Heuristic::Size(1)),
        ];

        let null_entries = vec![
            init_test_entry(0, 0, Heuristic::Size(1)),
            init_test_entry(3, 0, Heuristic::Size(1)),
            init_test_entry(4, 0, Heuristic::Size(1)),
        ];

        let test_entries = remove_unique_entries_by_heuristic(entries, false);

        assert_eq!(test_entries, null_entries);
    }

    #[test]
    pub fn test_group_entries_by_size()
    {
        let entries = vec![
            init_test_entry(0, 0, Heuristic::Null),
            init_test_entry(1, 2, Heuristic::Null),
            init_test_entry(2, 1, Heuristic::Null),
            init_test_entry(3, 2, Heuristic::Null),
            init_test_entry(4, 0, Heuristic::Null),
            init_test_entry(5, 5, Heuristic::Null),
        ];

        let null_entries = vec![
            vec![
                init_test_entry(0, 0, Heuristic::Null),
                init_test_entry(4, 0, Heuristic::Null),
            ],
            vec![
                init_test_entry(2, 1, Heuristic::Null),
            ],
            vec![
                init_test_entry(1, 2, Heuristic::Null),
                init_test_entry(3, 2, Heuristic::Null),
            ],
            vec![
                init_test_entry(5, 5, Heuristic::Null),
            ],
        ];

        let test_entries = group_by_heuristic(entries, false);

        assert_eq!(test_entries, null_entries);
    }

    #[test]
    pub fn test_group_entries_by_hvalue()
    {
        let entries = vec![
            init_test_entry(0, 0, Heuristic::Size(0)),
            init_test_entry(1, 0, Heuristic::Size(2)),
            init_test_entry(2, 0, Heuristic::Size(1)),
            init_test_entry(3, 0, Heuristic::Size(2)),
            init_test_entry(4, 0, Heuristic::Size(0)),
            init_test_entry(5, 0, Heuristic::Size(5)),
        ];

        let null_entries = vec![
            vec![
                init_test_entry(0, 0, Heuristic::Size(0)),
                init_test_entry(4, 0, Heuristic::Size(0)),
            ],
            vec![
                init_test_entry(2, 0, Heuristic::Size(1)),
            ],
            vec![
                init_test_entry(1, 0, Heuristic::Size(2)),
                init_test_entry(3, 0, Heuristic::Size(2)),
            ],
            vec![
                init_test_entry(5, 0, Heuristic::Size(5)),
            ],
        ];

        let test_entries = group_by_heuristic(entries, false);

        assert_eq!(test_entries, null_entries);
    }

    #[test]
    pub fn test_count_duplicates()
    {
        let mut entries = vec![
            init_test_entry(0, 0, Heuristic::Null),
            init_test_entry(1, 2, Heuristic::Null),
            init_test_entry(2, 1, Heuristic::Null),
            init_test_entry(3, 2, Heuristic::Null),
            init_test_entry(4, 0, Heuristic::Null),
            init_test_entry(5, 5, Heuristic::Null),
        ];

        entries.sort_by( | a, b | compare_entries(a, b, false) );

        let null_counts = (2, 2);
        let test_counts = count_duplicate_entries_files(&entries, false);

        assert_eq!(test_counts, null_counts);
    }
}

