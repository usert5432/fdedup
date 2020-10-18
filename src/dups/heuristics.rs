use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

use crypto_hash::{Algorithm, Hasher};
use fs_entry::{FSEntry, Heuristic};

pub type HeuristicFn = dyn Fn(&FSEntry) -> io::Result<Heuristic>;

pub fn fn_file_hash(entry : &FSEntry, algo : Algorithm)
    -> io::Result<Heuristic>
{
    let mut f      = File::open(&entry.paths[0])?;
    let mut hasher = Hasher::new(algo);

    io::copy(&mut f, &mut hasher)?;

    Ok(Heuristic::Hash(hasher.finish()))
}

pub fn fn_file_size(entry : &FSEntry) -> io::Result<Heuristic> {
    Ok(Heuristic::Size(entry.size))
}

pub fn fn_file_device(entry : &FSEntry) -> io::Result<Heuristic> {
    Ok(Heuristic::Device(entry.dev))
}

pub fn fn_first_bytes(entry : &FSEntry, number : usize)
    -> io::Result<Heuristic>
{
    let f = File::open(&entry.paths[0])?;
    let mut result : Vec<u8> = Vec::with_capacity(number);

    f.take(number as u64).read_to_end(&mut result)?;

    Ok(Heuristic::Bytes(result))
}

pub fn fn_last_bytes(entry : &FSEntry, number : usize)
    -> io::Result<Heuristic>
{
    let mut f = File::open(&entry.paths[0])?;

    if entry.size > (number as u64) {
        f.seek(SeekFrom::End(-(number as i64)))?;
    }

    let mut result : Vec<u8> = Vec::with_capacity(number);
    f.read_to_end(&mut result)?;

    Ok(Heuristic::Bytes(result))
}

