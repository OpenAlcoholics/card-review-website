use std::path::Path;
use std::process::{Command, Output};

use crate::GitError;

pub(crate) struct Branch {}

pub(crate) struct Repo {
    remote: String,
    path: String,
    branch: String,
}

impl Repo {
    pub(crate) fn new(remote: String, path: String, branch: String) -> Result<Repo, GitError> {
        let repo = Repo {
            remote: remote.clone(),
            path: path.clone(),
            branch,
        };

        if !Path::exists(path.as_ref()) {
            repo.cmd()
                .arg("clone")
                .arg(&remote)
                .arg(&path)
                .output()?;
        };

        Ok(repo)
    }

    fn cmd(&self) -> Command {
        let mut command = Command::new("git");
        command.current_dir(self.path.as_str());

        command
    }

    pub(crate) fn reset(&self) -> std::io::Result<Output> {
        self.cmd()
            .arg("reset")
            .output()
    }

    pub(crate) fn clean_all(&self) -> std::io::Result<Output> {
        self.cmd()
            .arg("clean")
            .arg("-fdx")
            .output()
    }

    pub(crate) fn stash(&mut self) -> std::io::Result<Output> {
        self.cmd()
            .arg("stash")
            .output()
    }

    pub(crate) fn stash_pop(&mut self) -> std::io::Result<Output> {
        self.cmd()
            .arg("stash")
            .arg("pop")
            .output()
    }

    pub(crate) fn checkout(&mut self, branch: String) -> std::io::Result<Output> {
        self.cmd()
            .arg("checkout")
            .arg(branch)
            .output()
    }
}
