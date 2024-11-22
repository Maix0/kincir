//! This is a crate that aims to wrap bubblewrap cli tool to be used in rust without forcing the
//! use of raw CLI api

mod fs_options;
use std::collections::{HashMap, HashSet};

pub use fs_options::FsOptions;
type CowStr = std::borrow::Cow<'static, str>;

pub struct BWrapBuilder {
    clear_env: bool,
    env: HashMap<CowStr, CowStr>,
    unset_env: HashSet<CowStr>,
    fs_options: Vec<fs_options::FsOptions>,
    cwd: Option<CowStr>,
}

impl BWrapBuilder {
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
    pub fn add_env(mut self, key: CowStr, value: CowStr) -> Self {
        self.env.insert(key, value);
        self
    }
    pub fn add_unset_env(mut self, key: CowStr) -> Self {
        self.unset_env.insert(key);
        self
    }
    pub fn remove_env(mut self, key: CowStr) -> Self {
        self.env.remove(&key);
        self
    }
    pub fn remove_unset_env(mut self, key: CowStr) -> Self {
        self.unset_env.remove(&key);
        self
    }
    pub fn add_fs_options(mut self, option: fs_options::FsOptions) -> Self {
        self.fs_options.push(option);
        self
    }
    pub fn set_cwd(mut self, cwd: CowStr) -> Self {
        self.cwd = Some(cwd);
        self
    }
    pub fn unset_cwd(mut self) -> Self {
        self.cwd = None;
        self
    }
}

impl BWrapBuilder {
    pub fn build_args(&mut self) -> Vec<std::borrow::Cow<'static, str>> {
        let mut v: Vec<CowStr> = Vec::new();
        if self.clear_env {
            v.push("--clearenv".into());
        }
        for (key, value) in self.env.iter() {
            v.push("--setenv".into());
            v.push(key.clone());
            v.push(value.clone());
        }
        for key in self.unset_env.iter() {
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
