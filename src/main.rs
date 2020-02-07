use yaml_rust::{YamlLoader, YamlEmitter, Yaml};
use clap::Clap;
use regex::Regex;
use quyaml;

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

    find(&doc, path.as_slice(), &[]);
    Ok(())
}

fn find(doc: &Yaml, path: &[&str], sp: &[Yaml]) {
    if path.len() == 0 {
        println!("{:?} = {:?}", &sp, &doc);
        // Dump the YAML object
        let mut out_str = String::new();
        {
            let mut emitter = YamlEmitter::new(&mut out_str);
            emitter.dump(doc).unwrap(); // dump the YAML object to a String
        }
        println!("{}", out_str);
    } else {
        let key = path[0];
        let re_condition = Regex::new(r"^(.*?)(?:\((.*?)\))?$").unwrap();
        let cap = re_condition.captures(key).unwrap();
        let mut key = cap.get(1).map_or("*", |m| m.as_str());
        if key == "" {
            key = "*";
        }
        let condition = &cap.get(2);

        match &doc {
            // Yaml::Hash(ref _map) if !key.contains("*") => {
            //     find(&doc[key], &path[1..], &[sp, &[Yaml::String(key.to_owned())]].concat())
            // },
            Yaml::Array(ref _array) if key != "*" => {
                if let Ok(intkey) = key.parse::<usize>() {
                    if !check(&doc[intkey], &condition) {
                        return;
                    }
                    find(&doc[intkey], &path[1..], &[sp, &[Yaml::Integer(intkey as i64)]].concat())
                }
            },

            // Yaml::Hash(ref map) if key == "*" => {
            //     for entry in map.iter() {
            //         find(&entry.1, &path[1..], &[sp, &[entry.0.clone()]].concat());
            //     }
            // },

            Yaml::Hash(ref map) => {
                let re_str = key.replace("*", ".*?");
                let re_str = format!("^{}$", re_str);
                let re = Regex::new(&re_str).unwrap();

                for entry in map.iter() {
                    if let Yaml::String(s) = entry.0 {
                        if re.is_match(s) {
                            if !check(&entry.1, &condition) {
                                continue;
                            }
                            find(&entry.1, &path[1..], &[sp, &[entry.0.clone()]].concat());
                        }
                    }
                }
            },

            Yaml::Array(ref array) => {
                for (i, v) in array.iter().enumerate() {
                    if !check(&v, &condition) {
                        continue;
                    }
                    find(&v, &path[1..], &[sp, &[Yaml::Integer(i as i64)]].concat());
                }
            },
            _ => {

            }
        }
    }
}

fn check(doc: &Yaml, condition: &Option<regex::Match>) -> bool {
    if let Some(condition_match) = condition {
        let condition_str = condition_match.as_str();
        let re = Regex::new(r"^\s*(.*?)\s*(==|=|!=|>|<)\s*(.*?)$").unwrap();
        let cap = re.captures(&condition_str).unwrap();

        let l = cap.get(1).map_or("", |m| m.as_str());
        let e = cap.get(2).map_or("", |m| m.as_str());
        let r = cap.get(3).map_or("", |m| m.as_str());

        let l = normalize(&doc, l);
        let r = normalize(&doc, r);

        match l {
            Yaml::Integer(a) => {
                if let Yaml::Integer(b) = r {
                    match e {
                        "=" | "==" => {
                            return a == b
                        },
                        "!=" => {
                            return a != b
                        },
                        ">" => {
                            return a > b
                        },
                        "<" => {
                            return a < b
                        },
                        _ => {
                            return false;
                        }
                    }
                } else {
                    return false;
                }
            },
            Yaml::String(a) => {
                if let Yaml::String(b) = r {
                    match e {
                        "=" | "==" => {
                            return a == b
                        },
                        "!=" => {
                            return a != b
                        },
                        _ => {
                            return false;
                        }
                    }
                } else {
                    return false;
                }
            },
            _ => {
                return false;
            },
        }
    }
    return true;
}

fn normalize(doc: &Yaml, str: &str) -> Yaml {
    match str.chars().next() {
        Some('"') => {
            let mut s = str.chars().next().map(|c| &str[c.len_utf8()..]).unwrap().to_string();
            s.pop();
            Yaml::String(s.to_owned())
        },
        Some('-') |
        Some('0') |
        Some('1') |
        Some('2') |
        Some('3') |
        Some('4') |
        Some('5') |
        Some('6') |
        Some('7') |
        Some('8') |
        Some('9') => {
            Yaml::Integer(str.parse::<i64>().unwrap())
        }
        _ => {
            let path: Vec<&str> = str.split(".").collect();
            let value = get(&doc, path.as_slice());
            value.clone()
        }
    }
}

fn get<'a>(doc: &'a Yaml, path: &[&str]) -> &'a Yaml {
    if path.len() == 0 {
        &doc
    } else {
        let key = path[0];
        match &doc {
            Yaml::Array(ref _array) if key != "*" => {
                if let Ok(intkey) = key.parse::<usize>() {
                    get(&doc[intkey], &path[1..])
                } else {
                    &Yaml::BadValue
                }
            },
            Yaml::Hash(ref _map) if !key.contains("*") => {
                get(&doc[key], &path[1..])
            },
            _ => {
                &Yaml::BadValue
            }
        }
    }
}
