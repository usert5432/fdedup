use std::cmp::Ordering;
use std::io;
use indicatif::ProgressBar;

use fs_entry::{FSEntry, Heuristic};
use dups::heuristics::HeuristicFn;
use utils::progress::get_progress_bar;

pub struct Evaluator {
    pub func            : Box<HeuristicFn>,
    pub verbose         : bool,
    pub abort_on_error  : bool,
    pub pbar            : Option<ProgressBar>,
}

impl Evaluator {

    pub fn new(
        size : usize, title : &str, verbose : bool, abort_on_error : bool,
        func : Box<HeuristicFn>
    ) -> Self
    {
        let pbar = get_progress_bar(size as u64, title, verbose);
        Self{ func, verbose, abort_on_error, pbar }
    }

    fn eval(&mut self, entry : &FSEntry) -> io::Result<Heuristic> {
        let result = (*self.func)(entry);
        self.tick();

        result
    }

    pub fn tick(&mut self) {
        if let Some(pb) = &self.pbar {
            pb.inc(1);
        }
    }

    pub fn finish(&mut self) {
        if let Some(pb) = &self.pbar {
            pb.abandon();
        }
    }

    pub fn evaluate(&mut self, entries : &mut Vec<FSEntry>) -> io::Result<()> {
        // Sorting by inode helps speed up file reading on HDD for some FS
        entries.sort_by(
            | a, b | {
                match a.dev.cmp(&b.dev) {
                    Ordering::Equal => a.inode.cmp(&b.inode),
                    other           => other,
                }
            }
        );

        for entry in entries.iter_mut() {
            let value = sloppy_unwrap_or_continue!(
                self.eval(&entry), self, ""
            );

            entry.hvalue = value;
        }

        self.finish();

        Ok(())
    }
}

