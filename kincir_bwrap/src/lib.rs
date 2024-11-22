#![warn(clippy::pedantic)]

//! This is a crate that aims to wrap bubblewrap cli tool to be used in rust without forcing the
//! use of raw CLI api

mod fs_options;
use std::collections::{HashMap, HashSet};

pub use fs_options::FsOptions;
type CowStr = std::borrow::Cow<'static, str>;

pub struct BWrapBuilder {
    clear_env: bool,
    cwd: Option<CowStr>,
    env: HashMap<CowStr, CowStr>,
    fs_options: Vec<fs_options::FsOptions>,
    new_session: bool,
    unset_env: HashSet<CowStr>,
}

impl BWrapBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            new_session: false,
            clear_env: false,
            env: HashMap::new(),
            unset_env: HashSet::new(),
            fs_options: Vec::new(),
            cwd: None,
        }
    }

    #[must_use]
    pub fn clear_env(mut self, clear_env: bool) -> Self {
        if clear_env {
            self.clear_env = true;
            self.env.clear();
            self.unset_env.clear();
        } else {
            self.clear_env = false;
        }
        self
    }
    #[must_use]
    pub fn add_env(mut self, key: impl Into<CowStr>, value: impl Into<CowStr>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
    #[must_use]
    pub fn add_unset_env(mut self, key: impl Into<CowStr>) -> Self {
        self.unset_env.insert(key.into());
        self
    }
    #[must_use]
    pub fn remove_env(mut self, key: impl Into<CowStr>) -> Self {
        self.env.remove(&key.into());
        self
    }
    #[must_use]
    pub fn remove_unset_env(mut self, key: impl Into<CowStr>) -> Self {
        self.unset_env.remove(&key.into());
        self
    }
    #[must_use]
    pub fn add_fs_options(mut self, option: fs_options::FsOptions) -> Self {
        self.fs_options.push(option);
        self
    }
    #[must_use]
    pub fn set_cwd(mut self, cwd: impl Into<CowStr>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }
    #[must_use]
    pub fn unset_cwd(mut self) -> Self {
        self.cwd = None;
        self
    }

    #[must_use]
    pub fn new_session(mut self, enable: bool) -> Self {
        self.new_session = enable;
        self
    }
}

impl Default for BWrapBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BWrapBuilder {
    pub fn build_args(&mut self) -> Vec<std::borrow::Cow<'static, str>> {
        let mut v: Vec<CowStr> = Vec::new();
        if self.new_session {
            v.push("--new-session".into());
        }
        if self.clear_env {
            v.push("--clearenv".into());
        }
        for (key, value) in &self.env {
            v.push("--setenv".into());
            v.push(key.clone());
            v.push(value.clone());
        }
        for key in &self.unset_env {
            v.push("--unsetenv".into());
            v.push(key.clone());
        }
        if let Some(cwd) = self.cwd.as_ref() {
            v.push("--chdir".into());
            v.push(cwd.clone());
        }
        for opts in &self.fs_options {
            v.extend(opts.to_option());
        }


        v
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn env1() {
        let args = crate::BWrapBuilder::new()
            .add_env("MY_ENV_VAR", "some value")
            .add_unset_env("PATH")
            .add_env("MY_ENV_VAR", "override")
            .build_args();
        assert_eq!(
            args,
            vec!["--setenv", "MY_ENV_VAR", "override", "--unsetenv", "PATH"]
        );
    }
    #[test]
    fn env2() {
        let args = crate::BWrapBuilder::new()
            .add_env("MY_ENV_VAR", "some value")
            .add_unset_env("PATH")
            .add_env("MY_ENV_VAR", "override")
            .clear_env(true)
            .add_env("PAGER", "kess")
            .add_unset_env("MANPAGER")
            .add_env("EDITOR", "nano")
            .remove_env("PAGER")
            .build_args();
        assert_eq!(
            args,
            vec![
                "--clearenv",
                "--setenv",
                "EDITOR",
                "nano",
                "--unsetenv",
                "MANPAGER"
            ]
        );
    }
    #[test]
    fn cwd1() {
        let args = crate::BWrapBuilder::new()
            .set_cwd("/my/super/path")
            .build_args();
        assert_eq!(args, vec!["--chdir", "/my/super/path"]);
    }
    #[test]
    fn cwd2() {
        let args = crate::BWrapBuilder::new()
            .set_cwd("/my/super/path")
            .set_cwd("/my/better/path")
            .build_args();
        assert_eq!(args, vec!["--chdir", "/my/better/path"]);
    }
    #[test]
    fn cwd3() {
        let args = crate::BWrapBuilder::new()
            .set_cwd("/my/super/path")
            .set_cwd("/my/better/path")
            .unset_cwd()
            .build_args();
        assert_eq!(args, Vec::<crate::CowStr>::new());
    }
}
