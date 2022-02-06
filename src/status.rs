#[path = "git.rs"] mod git;

use std::env;
use regex::Regex;

fn minify_dir(name: &str) -> String {
    let regexp = Regex::new(r"(\W*\w)").unwrap();
    if let Some(mat) = regexp.find(name) {
        return name[mat.start()..mat.end()].to_owned();
    }
    return name.to_owned();
}

fn minify_path(path: &str, keep: usize) -> String {
    let mut result: Vec<String> = vec![];
    // if let Some(home_path) = env::home_dir() {
    //     if let Some(home) = home_path.to_str() {
    //         path = path.replace(home, "~");
    //     }
    // }
    let dirs: Vec<&str> = path.split("/").collect();
    let limit = dirs.len() - keep;
    for (i, name) in dirs.iter().enumerate() {
        if i < limit {
            result.push(minify_dir(name));
        } else {
            result.push(name.to_string());
        }
    }
    return result.join("/");
}

fn apply_vcs(path: &str) -> String {
    let root = git::Git::root_dir();
    let common = &path[0..root.len()];
    let remainder = &path[root.len()..];
    return minify_path(&common, 1) + &git::Git::stat() + &minify_path(&remainder, 1);
}

pub fn statusline() -> String {
    if let Some(path) = env::current_dir().unwrap().to_str() {
        return apply_vcs(&path);
    }
    return "".to_owned();
}
