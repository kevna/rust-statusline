use std::process::Command;
use std::fmt;

const ICON: &str = "\x1b[38;5;202m\u{E0A0}\x1b[m";

#[derive(Debug, PartialEq)]
struct AheadBehind {
    ahead: usize,
    behind: usize,
}

impl fmt::Display for AheadBehind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ahead = self.ahead > 0;
        let behind = self.behind > 0;
        if !(ahead || behind) {
            return write!(f, "");
        }
        if ahead {
            write!(f, "\x1b[32m↑{}", self.ahead)?;
        }
        if behind {
            write!(f, "\x1b[31m↓{}", self.behind)?;
        }

        return write!(f, "\x1b[m");
    }
}

#[derive(Debug, PartialEq)]
struct Status {
    unmerged: usize,
    staged: usize,
    unstaged: usize,
    untracked: usize,
}

impl Status {
    fn zero() -> Status {
        return Status{
            unmerged: 0,
            staged: 0,
            unstaged: 0,
            untracked: 0,
        };
    }
    fn has_changes(&self) -> bool {
        self.unstaged > 0 || self.untracked > 0 || self.staged > 0 || self.unmerged > 0
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.has_changes() {
            return write!(f, "");
        }

        if self.unmerged > 0 {
            write!(f, "\x1b[91;1m{}", self.unmerged)?;
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

#[derive(Debug, PartialEq)]
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
        let mut status = Status::zero();
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
                "u" => {
                    status.unmerged += 1
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(AheadBehind{ahead: 0, behind: 0}, "")]
    #[case(AheadBehind{ahead: 1, behind: 10}, "\x1b[32m↑1\x1b[31m↓10\x1b[m")]
    #[case(AheadBehind{ahead: 1, behind: 0}, "\x1b[32m↑1\x1b[m")]
    #[case(AheadBehind{ahead: 0, behind: 1}, "\x1b[31m↓1\x1b[m")]
    fn test_ab_fmt(#[case] ab: AheadBehind, #[case] expected: String) {
        let actual = format!("{}", ab);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case(Status::zero(), false)]
    #[case(Status{unmerged: 1, staged: 0, unstaged: 0, untracked: 0}, true)]
    #[case(Status{unmerged: 0, staged: 1, unstaged: 0, untracked: 0}, true)]
    #[case(Status{unmerged: 0, staged: 0, unstaged: 1, untracked: 0}, true)]
    #[case(Status{unmerged: 0, staged: 0, unstaged: 0, untracked: 1}, true)]
    #[case(Status{unmerged: 1, staged: 1, unstaged: 1, untracked: 1}, true)]
    fn test_status_has_changes(#[case] status: Status, #[case] expected: bool) {
        let actual = status.has_changes();
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case(Status::zero(), "")]
    #[case(Status{unmerged: 1, staged: 0, unstaged: 0, untracked: 0}, "\x1b[91;1m1\x1b[m")]
    #[case(Status{unmerged: 0, staged: 1, unstaged: 0, untracked: 0}, "\x1b[32m1\x1b[m")]
    #[case(Status{unmerged: 0, staged: 0, unstaged: 1, untracked: 0}, "\x1b[31m1\x1b[m")]
    #[case(Status{unmerged: 0, staged: 0, unstaged: 0, untracked: 1}, "\x1b[90m1\x1b[m")]
    #[case(Status{unmerged: 1, staged: 1, unstaged: 1, untracked: 1}, "\x1b[91;1m1\x1b[32m1\x1b[31m1\x1b[90m1\x1b[m")]
    fn test_status_fmt(#[case] status: Status, #[case] expected: String) {
        let actual = format!("{}", status);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case(
        "",
        Repo{
            branch: "".to_owned(),
            ab: None,
            status: Status::zero(),
            stashes: 0,
        },
    )]
    #[case(
        "
# branch.oid (initial)
# branch.head (detached)
1 MM N... 100644 100644 100644 3e2ceb914cf9be46bf235432781840f4145363fd 3e2ceb914cf9be46bf235432781840f4145363fd README.md
        ",
        Repo{
            branch: "(detached)".to_owned(),
            ab: None,
            status: Status{unmerged: 0, staged: 1, unstaged: 1, untracked: 0},
            stashes: 0,
        },
    )]
    #[case(
        "
# branch.oid 51c9c58e2175b768137c1e38865f394c76a7d49d
# branch.head master
# branch.upstream origin/master
# branch.ab +1 -10
# stash 3
1 .M N... 100644 100644 100644 3e2ceb914cf9be46bf235432781840f4145363fd 3e2ceb914cf9be46bf235432781840f4145363fd Gopkg.lock
1 .M N... 100644 100644 100644 cecb683e6e626bcba909ddd36d3357d49f0cfd09 cecb683e6e626bcba909ddd36d3357d49f0cfd09 Gopkg.toml
1 .M N... 100644 100644 100644 aea984b7df090ce3a5826a854f3e5364cd8f2ccd aea984b7df090ce3a5826a854f3e5364cd8f2ccd porcelain.go
1 .D N... 100644 100644 000000 6d9532ba55b84ec4faf214f9cdb9ce70ec8f4f5b 6d9532ba55b84ec4faf214f9cdb9ce70ec8f4f5b porcelain_test.go
2 R. N... 100644 100644 100644 44d0a25072ee3706a8015bef72bdd2c4ab6da76d 44d0a25072ee3706a8015bef72bdd2c4ab6da76d R100 hm.rb     hw.rb
u UU N... 100644 100644 100644 100644 ac51efdc3df4f4fd328d1a02ad05331d8e2c9111 36c06c8752c78d2aff89571132f3bf7841a7b5c3 e85207e04dfdd5eb0a1e9febbc67fd837c44a1cd hw.rb
? _porcelain_test.go
? git.go
? git_test.go
? goreleaser.yml
? vendor/
        ",
        Repo{
            branch: "master".to_owned(),
            ab: Some(AheadBehind{ahead: 1, behind: 10}),
            status: Status{unmerged: 1, staged: 1, unstaged: 4, untracked: 5},
            stashes: 3,
        },
    )]
    fn test_repo_from_string(#[case] string: String, #[case] expected: Repo) {
        let actual: Repo = string.parse().unwrap();
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case(
        Repo{
            branch: "master".to_owned(),
            ab: None,
            status: Status::zero(),
            stashes: 0,
        },
        ICON.to_owned() + "master\x1b[91;1m↯\x1b[m",
    )]
    #[case(
        Repo{
            branch: "master".to_owned(),
            ab: Some(AheadBehind{ahead: 0, behind: 0}),
            status: Status::zero(),
            stashes: 0,
        },
        ICON.to_owned() + "master",
    )]
    #[case(
        Repo{
            branch: "master".to_owned(),
            ab: None,
            status: Status{unmerged: 1, staged: 1, unstaged: 1, untracked: 1},
            stashes: 0,
        },
        ICON.to_owned() + "master\x1b[91;1m↯\x1b[m(\x1b[91;1m1\x1b[32m1\x1b[31m1\x1b[90m1\x1b[m)",
    )]
    #[case(
        Repo{
            branch: "master".to_owned(),
            ab: Some(AheadBehind{ahead: 1, behind: 10}),
            status: Status{unmerged: 1, staged: 1, unstaged: 4, untracked: 5},
            stashes: 3,
        },
        ICON.to_owned() + "master\x1b[32m↑1\x1b[31m↓10\x1b[m(\x1b[91;1m1\x1b[32m1\x1b[31m4\x1b[90m5\x1b[m){3}",
    )]
    fn test_repo_stat(#[case] repo: Repo, #[case] expected: String) {
        let actual = repo.stat();
        assert_eq!(expected, actual);
    }
}
