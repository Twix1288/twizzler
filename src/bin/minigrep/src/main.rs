use std::env;
use std::process;
use minigrep::{run, Config}; 

fn main() {
    let args: Vec<String> = env::args().collect(); //Stores the values from the command line

    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    }); //Saves a config instance

    if let Err(e) = run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}