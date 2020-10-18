pub mod parse;

use std::str::FromStr;

pub const EXCLUDES : [&str; 2] = [ ".git", ".svn" ];

#[derive(Clone)]
#[derive(Copy)]
#[derive(PartialEq)]
pub enum DedupAction {
    Symlink, Hardlink, Print,
}

pub struct Args {
    pub paths           : Vec<String>,
    pub action          : DedupAction,
    pub result_path     : Option<String>,
    pub includes        : Vec<String>,
    pub excludes        : Vec<String>,
    pub abort_on_error  : bool,
    pub show_progress   : bool,
    pub verbosity       : String,
    pub one_file_system : bool,
    pub hash            : String,
    pub n_read          : usize,
    pub min_file_size   : Option<u64>,
    pub max_file_size   : Option<u64>,
    pub dry_run         : bool,
}

impl FromStr for DedupAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hardlink" => Ok(DedupAction::Hardlink),
            "symlink"  => Ok(DedupAction::Symlink),
            "print"    => Ok(DedupAction::Print),
            _          => Err(format!("Cannot parse dedup action: {}", s)),
        }
    }
}

