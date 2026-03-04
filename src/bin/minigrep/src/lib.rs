use std::error::Error;
use std::str;
use twizzler::object::{MapFlags, Object, TypedObject}; 
use twizzler::marker::BaseType;

pub struct Config {
    pub query: String,
    pub target_id: twizzler::object::ObjID,
    pub ignore_case: bool,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }

        let query = args[1].trim_matches('"').to_string();
        
        let id_val: u128 = u128::from_str_radix(&args[2], 16)
             .map_err(|_| "Invalid Object ID format (expected hex string)")?;
             
        let target_id = twizzler::object::ObjID::new(id_val);
        let ignore_case = std::env::var("IGNORE_CASE").is_ok();

        Ok(Config {
            query,
            target_id,
            ignore_case,
        })
    }
}

// Manual layout to match the OS (Length first, then Data)
#[repr(C)]
pub struct ManualTextObject {
    pub len: usize,         
    pub data: [u8; 256],    
}

// Tell Twizzler this struct is valid for object mapping
impl BaseType for ManualTextObject {}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let text_obj = Object::<ManualTextObject>::map(config.target_id, MapFlags::READ)?;
    let base = text_obj.base();

    // Clamp length to 256
    let valid_len = std::cmp::min(base.len, 256);
    let content_slice = &base.data[0..valid_len]; 
    let contents = str::from_utf8(content_slice)
        .map_err(|_| "Memory contained invalid UTF-8")?;
    // ----------------------------------------------------

    let results = if config.ignore_case { 
        search_case_insensitive(&config.query, contents)
    } else {
        search(&config.query, contents)
    };

    if results.is_empty() {
        println!("DIAGNOSTIC: Search returned no results.");
    }

    for line in results {
        println!("{line}");
    }

    Ok(())
}

pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents
        .lines()
        .filter(|line| line.contains(query))
        .collect()
}

pub fn search_case_insensitive<'a>(
    query: &str,
    contents: &'a str,
) -> Vec<&'a str> {
    let query = query.to_lowercase();
    contents
        .lines()
        .filter(|line| line.to_lowercase().contains(&query))
        .collect()
}