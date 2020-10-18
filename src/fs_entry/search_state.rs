use std::fs::{self, Metadata};
use std::io;
use std::os::unix::prelude::*;
use std::path::Path;

use globset::{GlobBuilder, GlobSetBuilder, GlobSet};
use indicatif::{ProgressBar, ProgressStyle};

use fs_entry::{Dev, Priority};

pub struct SearchState {
    pub abort_on_error : bool,
    pub verbose  : bool,
    pub one_fs   : bool,
    pub dev      : Option<Dev>,
    pub spinner  : Option<ProgressBar>,
    pub excludes : Option<GlobSet>,
    pub includes : Option<GlobSet>,
    pub min_size : Option<u64>,
    pub max_size : Option<u64>,
    pub priority : Priority,
}

fn build_glob_set_impl(patterns : &[String])
    -> Result<Option<GlobSet>, globset::Error>
{
    if patterns.is_empty() {
        return Ok(None);
    }

    let mut builder = GlobSetBuilder::new();

    for pattern in patterns.iter() {
        if pattern.starts_with('/') || pattern.starts_with("**") {
            builder.add(
                GlobBuilder::new(pattern).literal_separator(true).build()?
            );
        }
        else {
            builder.add(
                GlobBuilder::new(format!("**/{}", pattern).as_str())
                    .literal_separator(true).build()?
            );
        }
    }

    builder.build().map( |r| { Some(r) } )
}

fn build_glob_set(patterns : &[String]) -> io::Result<Option<GlobSet>>
{
    build_glob_set_impl(patterns).map_err(
        | x | { io::Error::new(io::ErrorKind::Other, x.to_string() ) }
    )
}

impl SearchState {

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        path           : &Path,
        abort_on_error : bool,
        verbose        : bool,
        one_fs         : bool,
        inc_patterns   : &[String],
        ex_patterns    : &[String],
        min_size       : Option<u64>,
        max_size       : Option<u64>,
        priority       : Priority,
    ) -> io::Result<Self> {

        let mut dev     : Option<u64> = None;
        let mut spinner : Option<ProgressBar> = None;

        if one_fs {
            let metadata = fs::metadata(path)?;
            dev = Some(metadata.dev());
        }

        if verbose {
            let s = ProgressBar::new_spinner();
            s.set_style(ProgressStyle::default_bar()
                .template("[{elapsed}] {spinner} {prefix} {wide_msg}")
            );
            s.set_prefix("Scanning");

            spinner = Some(s);
        }

        let includes = build_glob_set(inc_patterns)?;
        let excludes = build_glob_set(ex_patterns)?;

        Ok(Self {
            abort_on_error, verbose, one_fs, dev, spinner, includes, excludes,
            min_size, max_size, priority
        })
    }

    pub fn passes_filters(&self, path : &Path, meta : &Metadata) -> bool {
        if let Some(ref inc) = self.includes {
            if inc.is_match(path) {
                return true;
            }
        }

        if let Some(ref ex) = self.excludes {
            if ex.is_match(path) {
                return false;
            }
        };

        if meta.is_file() {
            if let Some(min) = self.min_size {
                if meta.size() < min {
                    return false;
                }
            }

            if let Some(max) = self.max_size {
                if meta.size() >= max {
                    return false;
                }
            }
        }

        true
    }

    pub fn tick(&mut self, path : &Path) {
        if let Some(s) = &self.spinner {
            s.set_message(path.to_str().unwrap());
        }
    }

    pub fn finish(&self) {
        if let Some(s) = &self.spinner {
            s.finish_with_message("Done");
        }
    }
}


