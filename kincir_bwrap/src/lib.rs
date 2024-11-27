#![warn(clippy::pedantic)]
#![deny(unsafe_code)]

//! This is a crate that aims to wrap bubblewrap cli tool to be used in rust without forcing the
//! use of raw CLI api

mod command;
mod fs_options;
mod namespace;
use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};

pub use command::Command;
pub use fs_options::FsOptions;
pub use namespace::NsFlags;
pub use namespace::NsOptions;

#[derive(Debug)]
pub struct BWrapBuilder {
    clear_env: bool,
    env: HashMap<OsString, OsString>,
    fs_options: Vec<fs_options::FsOptions>,
    unset_env: HashSet<OsString>,
    ns_options: NsOptions,
    command: command::Command,
}

impl BWrapBuilder {
    #[must_use]
    pub fn new(cmd: impl Into<command::Command>) -> Self {
        Self {
            clear_env: false,
            env: HashMap::new(),
            unset_env: HashSet::new(),
            fs_options: Vec::new(),
            ns_options: NsOptions::new(),
            command: cmd.into(),
        }
    }

    #[must_use]
    pub fn clear_env(&mut self, clear_env: bool) -> &mut Self {
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
    pub fn add_env(&mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> &mut Self {
        self.env
            .insert(key.as_ref().to_os_string(), value.as_ref().to_os_string());
        self
    }
    #[must_use]
    pub fn add_unset_env(&mut self, key: impl AsRef<OsStr>) -> &mut Self {
        self.unset_env.insert(key.as_ref().to_os_string());
        self
    }
    #[must_use]
    pub fn remove_env(&mut self, key: impl AsRef<OsStr>) -> &mut Self {
        self.env.remove(&key.as_ref().to_os_string());
        self
    }
    #[must_use]
    pub fn remove_unset_env(&mut self, key: impl AsRef<OsStr>) -> &mut Self {
        self.unset_env.remove(key.as_ref());
        self
    }
    #[must_use]
    pub fn add_fs_options(&mut self, option: fs_options::FsOptions) -> &mut Self {
        self.fs_options.push(option);
        self
    }
    #[must_use]
    pub fn set_cwd(&mut self, cwd: impl AsRef<std::path::Path>) -> &mut Self {
        self.ns_options.set_cwd(cwd);
        self
    }
    #[must_use]
    pub fn unset_cwd(&mut self) -> &mut Self {
        self.ns_options.unset_cwd();
        self
    }

    #[must_use]
    pub fn new_session(&mut self, enable: bool) -> &mut Self {
        self.ns_options.flags.set(NsFlags::NEW_SESSION, enable);
        self
    }

    /// this takes the `flags` and add them to the existing flagss
    #[must_use]
    pub fn add_namespace_flags(&mut self, flags: NsFlags) -> &mut Self {
        self.ns_options.flags.insert(flags);
        self
    }

    /// this takes the `flags` and directly set the value to that
    #[must_use]
    pub fn set_namespace_flags(&mut self, flags: NsFlags) -> &mut Self {
        self.ns_options.flags = flags;
        self
    }

    /// this takes the `flags` and remove them from the existing flagss
    #[must_use]
    pub fn remove_namespace_flags(&mut self, flags: NsFlags) -> &mut Self {
        self.ns_options.flags.remove(flags);
        self
    }

    #[must_use]
    pub fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.command.args.push(arg.as_ref().to_os_string());
        self
    }
}

impl BWrapBuilder {
    #[must_use]
    pub fn build_args(&mut self) -> Vec<OsString> {
        let mut v: Vec<OsString> = Vec::new();
        if self.clear_env {
            v.push(OsStr::new("--clearenv").to_os_string());
        }
        for (key, value) in &self.env {
            v.push(OsStr::new("--setenv").to_os_string());
            v.push(key.clone());
            v.push(value.clone());
        }
        for key in &self.unset_env {
            v.push(OsStr::new("--unsetenv").to_os_string());
            v.push(key.clone());
        }
        for opts in &self.fs_options {
            v.extend(opts.to_option());
        }
        v.extend(self.ns_options.to_options());
        v.push(OsStr::new("--").to_os_string());
        v.push(self.command.program.clone());
        v.extend(self.command.args.clone());
        v
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn env1() {
        let args = crate::BWrapBuilder::new("echo")
            .add_env("MY_ENV_VAR", "some value")
            .add_unset_env("PATH")
            .add_env("MY_ENV_VAR", "override")
            .build_args();
        assert_eq!(
            args,
            vec![
                "--setenv",
                "MY_ENV_VAR",
                "override",
                "--unsetenv",
                "PATH",
                "--",
                "echo"
            ]
        );
    }
    #[test]
    fn env2() {
        let args = crate::BWrapBuilder::new("echo")
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
                "MANPAGER",
                "--",
                "echo"
            ]
        );
    }
    #[test]
    fn cwd1() {
        let args = crate::BWrapBuilder::new("echo")
            .set_cwd("/my/super/path")
            .build_args();
        assert_eq!(args, vec!["--chdir", "/my/super/path", "--", "echo"]);
    }
    #[test]
    fn cwd2() {
        let args = crate::BWrapBuilder::new("echo")
            .set_cwd("/my/super/path")
            .set_cwd("/my/better/path")
            .build_args();
        assert_eq!(args, vec!["--chdir", "/my/better/path", "--", "echo"]);
    }
    #[test]
    fn cwd3() {
        let args = crate::BWrapBuilder::new("echo")
            .set_cwd("/my/super/path")
            .set_cwd("/my/better/path")
            .unset_cwd()
            .build_args();
        assert_eq!(args, vec!["--", "echo"]);
    }
}
