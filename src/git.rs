use std::process::Command;
use std::fmt;

struct AheadBehind {
    ahead: usize,
    behind: usize,
}

impl fmt::Display for AheadBehind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if (self.ahead - self.behind) != self.ahead {
            return write!(f, "↕{}", self.ahead+self.behind)
        }
        if self.ahead > 0 {
            return write!(f, "↑{}", self.ahead);
        }
        if self.behind > 0 {
            return write!(f, "↓{}", self.behind);
        }
        return write!(f, "");
    }
}

struct Status {
    staged: usize,
    unstaged: usize,
    untracked: usize,
}

impl Status {
    fn has_changes(&self) -> bool {
        return self.unstaged > 0 || self.untracked > 0 || self.staged >0
    }

    // fn to_string(&self) -> String {
    //     let mut result = "".to_owned();
    //     if self.staged > 0 {
    //         result += &format!("\x1b[32m{}", self.staged)
    //     }
    //     if self.unstaged > 0 {
    //         result += &format!("\x1b[31m{}", self.unstaged)
    //     }
    //     if self.untracked > 0 {
    //         result += &format!("\x1b[90m{}", self.untracked)
    //     }
    //     if result.len() > 0 {
    //         result += "\x1b[m"
    //     }
    //     return result
    // }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = "".to_owned();
        if self.staged > 0 {
            result += &format!("\x1b[32m{}", self.staged)
        }
        if self.unstaged > 0 {
            result += &format!("\x1b[31m{}", self.unstaged)
        }
        if self.untracked > 0 {
            result += &format!("\x1b[90m{}", self.untracked)
        }
        if result.len() > 0 {
            result += "\x1b[m"
        }
        return write!(f, "{}", result);
    }
}

pub trait VCS {
	fn root_dir(&self) -> String;
	fn branch(&self) -> String;
	fn stat(&self) -> String;
}

pub struct Git;

const ICON: &str = "\x1b[38;5;202m\u{E0A0}\x1b[m";

impl Git {
    fn run_command(args: &[&str]) -> String {
        // let args = ["rev-parse", "--symbolic-full-name", "--abbrev-ref", "HEAD"];
        let output = Command::new("git")
            .args(args)
            .output()
            .expect("failed to execute process");
        return String::from_utf8(output.stdout).unwrap().trim().to_string();
    }

    fn count(args: &[&str]) -> usize {
        let string = Git::run_command(args);
        let mut output: Vec<&str> = string.split("\n").collect();
        if output.last() == Some(&"") {
            output.pop();
        }
        return output.len();
    }

    fn ahead_behind() -> AheadBehind {
        return AheadBehind{
            ahead: Git::count(&["rev-list", "@{push}..HEAD"]),
            behind: Git::count(&["rev-list", "HEAD..@{upstream}"]),
        }
    }

    fn status() -> Status {
        let mut result = Status{
            staged: 0,
            unstaged: 0,
            untracked: 0,
        };
        for line in Git::run_command(&["status", "--porcelain"]).split("\n") {
            if str::starts_with(line, "??") {
                result.untracked += 1;
            } else {
                if &line[0..0] != " " {
                    result.staged += 1;
                }
                if &line[1..1] != " " {
                    result.unstaged += 1;
                }
            }
        }
        return result
    }

    fn stashes() -> usize {
        return Git::count(&["stash", "list"])
    }
}

impl VCS for Git {
    fn root_dir(&self) -> String {
        return Git::run_command(&["rev-parse", "--show-toplevel"]);
    }

    fn branch(&self) -> String {
        return Git::run_command(&["rev-parse", "--symbolic-full-name", "--abbrev-ref", "HEAD"]);
    }

    fn stat(&self) -> String {
        let mut result = ICON.to_owned();
        let branch = &self.branch();
        if !str::ends_with(&self.root_dir(), branch) {
            result += branch;
        }
        let ab = Git::ahead_behind();
        result += &format!("{ab}");
        let status = Git::status();
        if status.has_changes() {
            result += &format!("({status})");
        }
        let stashes = Git::stashes();
        if stashes > 0 {
            result += &format!("{{{stashes}}}");
        }
        return result;
    }
}
