use std::process::Command;
use std::fmt;

struct AheadBehind {
    ahead: usize,
    behind: usize,
}

impl fmt::Display for AheadBehind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ahead = self.ahead > 0;
        let behind = self.behind > 0;
        if ahead && behind {
            return write!(f, "↕{}", self.ahead+self.behind);
        }
        if ahead {
            return write!(f, "↑{}", self.ahead);
        }
        if behind {
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
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.has_changes() {
            return write!(f, "");
        }

        if self.staged > 0 {
            write!(f, "\x1b[32m{}", self.staged)?;
        }
        if self.unstaged > 0 {
            write!(f, "\x1b[31m{}", self.unstaged)?;
        }
        if self.untracked > 0 {
            write!(f, "\x1b[90m{}", self.untracked)?;
        }
        return write!(f, "\x1b[m");
    }
}

pub trait VCS {
	fn stat(&self) -> String;
}

const ICON: &str = "\x1b[38;5;202m\u{E0A0}\x1b[m";

pub struct Repo {
    branch: String,
    ab: Option<AheadBehind>,
    status: Status,
    stashes: usize,
}

impl Repo {
    fn run_command(args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .output()
            .expect("failed to execute process");
        return String::from_utf8(output.stdout).unwrap().trim_end().to_string();
    }

    pub fn new() -> Repo {
        Repo::run_command(&["status", "--porcelain=v2", "--branch", "--show-stash"]).parse().unwrap()
    }
}

impl std::str::FromStr for Repo {
    type Err = &'static str;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut branch = "".to_owned();
        let mut ab = None;
        let mut status = Status{
            staged: 0,
            unstaged: 0,
            untracked: 0,
        };
        let mut stashes = 0;

        for line in string.split("\n") {
            if line == "" {
                continue;
            }
            match &line[0..1] {
                "#" => {
                    let mut fields = line[2..].split(" ");
                    match fields.next() {
                        Some("branch.head") => {
                            if let Some(value) = fields.next() {
                                branch += value;
                            }
                        }
                        Some("branch.ab") => {
                            if let Some(ahead) = fields.next() {
                                if let Some(behind) = fields.next() {
                                    ab = Some(AheadBehind{
                                        ahead: ahead.parse().unwrap(),
                                        behind: behind[1..].parse().unwrap()
                                    });
                                }
                            }
                        }
                        Some("stash") => {
                            if let Some(value) = fields.next() {
                                stashes = value.parse().unwrap();
                            }
                        }
                        Some(&_) | None => {}
                    }
                }
                "1"|"2" => {
                    if &line[2..3] != "." {
                        status.staged += 1;
                    }
                    if &line[3..4] != "." {
                        status.unstaged += 1;
                    }
                },
                "?" => {
                    status.untracked += 1;
                }
                _ => {}
            }
        }
        return Ok(Repo{branch, ab, status, stashes})
    }
}

impl VCS for Repo {
    fn stat(&self) -> String {
        let mut result = ICON.to_owned();
        result += &self.branch;
        result += &match &self.ab {
            Some(ab) => format!("{}", ab),
            None => "\x1b[91;1m↯\x1b[m".to_owned(),
        };
        if self.status.has_changes() {
            result += &format!("({})", &self.status);
        }
        if &self.stashes > &0 {
            result += &format!("{{{}}}", &self.stashes);
        }
        return result;
    }
}
