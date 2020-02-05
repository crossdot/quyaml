use yaml_rust::{YamlLoader, Yaml};
use clap::Clap;

/// Query path over yaml file
#[derive(Clap)]
#[clap(version = "1.0", author = "Pavlikov V.")]
struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short = "c", long = "condition")]
    condition: Option<String>,
    /// Query path
    #[clap(required = true)]
    path: String,
}

fn main() {
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
    // Multi document support, doc is a yaml::Yaml
    let doc = &docs[0];
    // Debug support
    // println!("{:?}", doc);

    // println!("{:?}", doc["services"]["db"]);

    // Index access for map & array
    // assert_eq!(doc["foo"][0].as_str().unwrap(), "list1");
    // assert_eq!(doc["bar"][1].as_f64().unwrap(), 2.0);

    // Chained key/array access is checked and won't panic,
    // return BadValue if they are not exist.
    // assert!(doc["INVALID_KEY"][100].is_badvalue());

    // Dump the YAML object
    // let mut out_str = String::new();
    // {
    //     let mut emitter = YamlEmitter::new(&mut out_str);
    //     emitter.dump(doc).unwrap(); // dump the YAML object to a String
    // }
    // println!("{}", out_str);

    let splitted_path = opts.path.split("/");
    let path: Vec<&str> = splitted_path.collect();


    // let mut cur_element = &doc;
    // let mut path = String::new();
    // for s in splitted_path {
    //     println!("{}", s)
    // }
    find(&doc, path.as_slice(), &[]);
}

fn find(doc: &Yaml, path: &[&str], sp: &[&str]) {
    if path.len() == 0 {
        println!("{:?}", sp);
    } else {
        let search = path[0];
        match search {
            "*" => {
                println!("loop")
            },
            _ => find(&doc[search], &path[1..], &[sp, &[search]].concat()),
        }
    }
}