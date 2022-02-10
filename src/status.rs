#[path = "git.rs"] pub mod git;

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
    return "\x1b[94m".to_owned() + &result.join("/") + "\x1b[m";
}

pub fn apply_vcs(path: &str, vcs: &dyn git::VCS) -> String {
    let root = vcs.root_dir();
    let common = &path[0..root.len()];
    let remainder = &path[root.len()..];
    return minify_path(&common, 1) + &vcs.stat() + &minify_path(&remainder, 1);
}

pub fn statusline() -> String {
    if let Some(path) = env::current_dir().unwrap().to_str() {
        return apply_vcs(&path, &git::Git{});
    }
    return "".to_owned();
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("~", "~")]
    #[case("~root", "~r")]
    #[case("private_dot_config", "p")]
    #[case("._shares", "._")]
    fn test_minify_dir(#[case] input: &str, #[case] expected: &str) {
        let actual = minify_dir(input);
        assert_eq!(expected, actual)
    }

    #[rstest]
    #[case("~", 1, "\x1b[94m~\x1b[m")]
    #[case("/etc/X11/xorg.conf.d", 1, "\x1b[94m/e/X/xorg.conf.d\x1b[m")]
    #[case("~/.local/share/chezmoi/private_dot_config/i3", 1, "\x1b[94m~/.l/s/c/p/i3\x1b[m")]
    #[case("~/.local/share/chezmoi/private_dot_config/i3", 2, "\x1b[94m~/.l/s/c/private_dot_config/i3\x1b[m")]
    fn test_minify_path(#[case] input: &str, #[case] keep: usize, #[case] expected: &str) {
        let actual = minify_path(input, keep);
        assert_eq!(expected, actual)
    }

    struct MockVCS {
        root: String,
        branch: String,
        stat: String,
    }

    impl git::VCS for MockVCS {
        fn root_dir(&self) -> String {
            return self.root.to_owned();
        }

        fn branch(&self) -> String {
            return self.branch.to_owned();
        }

        fn stat(&self) -> String {
            return self.stat.to_owned();
        }
    }

    #[rstest]
    #[case(
        "~/.local/share/chezmoi",
        "branch",
        "\u{E0A0}master",
        "~/.local/share/chezmoi/private_dot_config/i3",
        "\x1b[94m~/.l/s/chezmoi\x1b[m\u{E0A0}master\x1b[94m/p/i3\x1b[m",
    )]
    #[case(
        "~/Documents/python/statusline/master",
        "branch",
        "\u{E0A0}",
        "~/Documents/python/statusline/master/statusline",
        // "\x1b[94m~/D/p/statusline/master\x1b[m\u{E0A0}\x1b[94m/statusline\x1b[m",
        "\x1b[94m~/D/p/s/master\x1b[m\u{E0A0}\x1b[94m/statusline\x1b[m",
    )]
    #[case(
        "~/Documents/python/statusline-master",
        "branch",
        "\u{E0A0}",
        "~/Documents/python/statusline-master/statusline",
        "\x1b[94m~/D/p/statusline-master\x1b[m\u{E0A0}\x1b[94m/statusline\x1b[m",
    )]
    #[case(
        "~/Documents/python/statusline/feature/newfeature",
        "feature/newfeature",
        "\u{E0A0}",
        "~/Documents/python/statusline/feature/newfeature/statusline",
        "\x1b[94m~/D/p/s/f/newfeature\x1b[m\u{E0A0}\x1b[94m/statusline\x1b[m",
    )]
    fn test_apply_vcs(#[case] root: &str, #[case] branch: &str, #[case] stat: &str, #[case] input: &str, #[case] expected: &str) {
        let mock = MockVCS{
            root: root.to_owned(),
            branch: branch.to_owned(),
            stat: stat.to_owned(),
        };
        let actual = apply_vcs(input, &mock);
        assert_eq!(expected, actual)
    }
}
