pub use self::search::collect_files;
use std::fmt;

pub mod search;
pub mod search_state;

pub type INode    = u64;
pub type Dev      = u64;
pub type Priority = u32;

#[derive(Clone)]
#[derive(PartialOrd)]
#[derive(Ord)]
#[derive(PartialEq)]
#[derive(Eq)]
pub enum Heuristic {
    Null,
    Device(u64),
    Size(u64),
    Bytes(Vec<u8>),
    Hash(Vec<u8>),
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct FSEntry {
    pub dev      : Dev,
    pub inode    : INode,
    pub size     : u64,
    pub priority : Priority,
    pub paths    : Vec<String>,
    pub hvalue   : Heuristic,
}

impl FSEntry {

    pub fn new(
        dev      : Dev,
        inode    : INode,
        size     : u64,
        priority : Priority,
        path     : String,
    ) -> Self
    {
        FSEntry{
            dev, inode, size, priority,
            paths : vec![path], hvalue : Heuristic::Null
        }
    }

    pub fn add_path(self : &mut Self, path : String) {
        let path_exists = self.paths.iter().any( |x| { **x == path } );

        if ! path_exists {
            self.paths.push(path);
        }
    }
}

impl fmt::Debug for Heuristic {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Heuristic::Size(s)   => write!(f, "Size({})", s),
            Heuristic::Device(d) => write!(f, "Dev({})", d),
            Heuristic::Bytes(b)  => {
                write!(f, "Bytes([ ")?;

                for byte in b.iter() {
                    write!(f, "{:02x} ", byte)?;
                }

                write!(f, "])")
            },
            Heuristic::Hash(h) => {
                write!(f, "Hash(")?;

                for byte in h.iter() {
                    write!(f, "{:02x}", byte)?;
                }

                write!(f, ")")
            },
            Heuristic::Null => write!(f, "Null"),
        }
    }
}
