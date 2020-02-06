use yaml_rust::{YamlLoader, Yaml};
use clap::Clap;
use regex::Regex;

// use std::io::prelude::*;
// use std::io::{self, BufRead, Read};

/// Query path over yaml file
#[derive(Clap)]
#[clap(version = "1.0", author = "Pavlikov V.")]
struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    // #[clap(short = "c", long = "condition")]
    // condition: Option<String>,
    /// Query path
    #[clap(required = true)]
    path: String,
}

fn main() -> Result<(), std::io::Error> {
    let opts: Opts = Opts::parse();
    
    let s =
"
services:
    db:
        image: postgres
        scale: 1
    front:
        image: nginx
        scale: 0
";
    let docs = YamlLoader::load_from_str(s).unwrap();
    let doc = &docs[0];

    let splitted_path = opts.path.split("/");
    let path: Vec<&str> = splitted_path.collect();


    // let mut cur_element = &doc;
    // let mut path = String::new();
    // for s in splitted_path {
    //     println!("{}", s)
    // }
    find(&doc, path.as_slice(), &[]);
    Ok(())
}

fn find(doc: &Yaml, path: &[&str], sp: &[Yaml]) {
    if path.len() == 0 {
        println!("{:?} = {:?}", &sp, &doc);
    } else {
        let key = path[0];
        match &doc {
            Yaml::Hash(ref _map) if !key.contains("*") => {
                find(&doc[key], &path[1..], &[sp, &[Yaml::String(key.to_owned())]].concat())
            },
            Yaml::Array(ref _array) if key != "*" => {
                if let Ok(intkey) = key.parse::<usize>() {
                    find(&doc[intkey], &path[1..], &[sp, &[Yaml::Integer(intkey as i64)]].concat())
                }
            },
            Yaml::Hash(ref map) if key == "*" => {
                for entry in map.iter() {
                    find(&entry.1, &path[1..], &[sp, &[entry.0.clone()]].concat());
                }
            },
            Yaml::Hash(ref map) => {
                let re_str = key.replace("*", ".*?");
                let re_str = format!("^{}$", re_str);
                let re = Regex::new(&re_str).unwrap();

                for entry in map.iter() {
                    if let Yaml::String(s) = entry.0 {
                        if re.is_match(s) {
                            find(&entry.1, &path[1..], &[sp, &[entry.0.clone()]].concat());
                        }
                    }
                }
            },
            Yaml::Array(ref array) => {
                for (i, v) in array.iter().enumerate() {
                    find(&v, &path[1..], &[sp, &[Yaml::Integer(i as i64)]].concat());
                }
            },
            _ => {

            }
        }
    }
}