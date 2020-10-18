extern crate fdedup;
use std::process;

fn main()
{
    if let Err(e) = fdedup::run() {
        println!("Application error: {}", e);

        process::exit(1);
    }
}

