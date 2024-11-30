#![warn(clippy::pedantic)]
#![deny(unsafe_code)]

//! This is a crate that aims to wrap bubblewrap cli tool to be used in rust without forcing the
//! use of raw CLI api

mod command;
mod fs_options;
mod namespace;
use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::os::fd::AsFd;
use std::path::Path;

pub use command::Command;
pub use fs_options::FsOptions;
pub use namespace::NsFlags;
pub use namespace::NsOptions;

#[derive(Debug)]
pub struct BwrapCommand<'fd> {
    bwrap: Option<OsString>,
    clear_env: bool,
    env: HashMap<OsString, OsString>,
    fs_options: Vec<fs_options::FsOptions<'fd>>,
    unset_env: HashSet<OsString>,
    ns_options: NsOptions,
    command: command::Command,
}

impl<'fd> BwrapCommand<'fd> {
    pub fn bwrap(&mut self, bwrap: Option<impl AsRef<OsStr>>) -> &mut Self {
        self.bwrap = bwrap.map(|i| i.as_ref().to_os_string());
        self
    }

    #[must_use]
    /// Create a new bwrap command.
    ///
    /// ```
    ///     # use kincir_bwrap::*;
    ///
    ///     let builder = BwrapCommand::new("echo");
    /// ```
    pub fn new(cmd: impl Into<command::Command>) -> Self {
        Self {
            bwrap: None,
            clear_env: false,
            env: HashMap::new(),
            unset_env: HashSet::new(),
            fs_options: Vec::new(),
            ns_options: NsOptions::new(),
            command: cmd.into(),
        }
    }

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

    pub fn add_env(&mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> &mut Self {
        self.env
            .insert(key.as_ref().to_os_string(), value.as_ref().to_os_string());
        self
    }

    pub fn add_unset_env(&mut self, key: impl AsRef<OsStr>) -> &mut Self {
        self.unset_env.insert(key.as_ref().to_os_string());
        self
    }

    pub fn remove_env(&mut self, key: impl AsRef<OsStr>) -> &mut Self {
        self.env.remove(&key.as_ref().to_os_string());
        self
    }

    pub fn remove_unset_env(&mut self, key: impl AsRef<OsStr>) -> &mut Self {
        self.unset_env.remove(key.as_ref());
        self
    }

    pub fn add_fs_options(&mut self, option: fs_options::FsOptions<'fd>) -> &mut Self {
        self.fs_options.push(option);
        self
    }

    pub fn set_cwd(&mut self, cwd: impl AsRef<std::path::Path>) -> &mut Self {
        self.ns_options.set_cwd(cwd);
        self
    }

    pub fn unset_cwd(&mut self) -> &mut Self {
        self.ns_options.unset_cwd();
        self
    }

    pub fn new_session(&mut self, enable: bool) -> &mut Self {
        self.ns_options.flags.set(NsFlags::NEW_SESSION, enable);
        self
    }

    /// this takes the `flags` and add them to the existing flagss
    pub fn add_namespace_flags(&mut self, flags: NsFlags) -> &mut Self {
        self.ns_options.flags.insert(flags);
        self
    }

    /// this takes the `flags` and directly set the value to that
    pub fn set_namespace_flags(&mut self, flags: NsFlags) -> &mut Self {
        self.ns_options.flags = flags;
        self
    }

    /// this takes the `flags` and remove them from the existing flagss
    pub fn remove_namespace_flags(&mut self, flags: NsFlags) -> &mut Self {
        self.ns_options.flags.remove(flags);
        self
    }

    /// Add an argument to the command (not an bwrap argument)
    pub fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.command.args.push(arg.as_ref().to_os_string());
        self
    }

    pub fn bind(&mut self, host: impl AsRef<Path>, guest: impl AsRef<Path>) -> &mut Self {
        self.add_fs_options(FsOptions::Bind {
            read_only: false,
            source: host.as_ref().as_os_str().to_os_string(),
            destination: guest.as_ref().as_os_str().to_os_string(),
            permission: None,
            try_: false,
        })
    }

    pub fn try_bind(&mut self, host: impl AsRef<Path>, guest: impl AsRef<Path>) -> &mut Self {
        self.add_fs_options(FsOptions::Bind {
            read_only: false,
            source: host.as_ref().as_os_str().to_os_string(),
            destination: guest.as_ref().as_os_str().to_os_string(),
            permission: None,
            try_: true,
        })
    }

    pub fn bind_read_only(&mut self, host: impl AsRef<Path>, guest: impl AsRef<Path>) -> &mut Self {
        self.add_fs_options(FsOptions::Bind {
            read_only: true,
            source: host.as_ref().as_os_str().to_os_string(),
            destination: guest.as_ref().as_os_str().to_os_string(),
            permission: None,
            try_: false,
        })
    }

    pub fn try_bind_ready_only(
        &mut self,
        host: impl AsRef<Path>,
        guest: impl AsRef<Path>,
    ) -> &mut Self {
        self.add_fs_options(FsOptions::Bind {
            read_only: true,
            source: host.as_ref().as_os_str().to_os_string(),
            destination: guest.as_ref().as_os_str().to_os_string(),
            permission: None,
            try_: true,
        })
    }

    pub fn proc_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.add_fs_options(FsOptions::Proc {
            destination: path.as_ref().as_os_str().to_os_string(),
            permission: None,
        })
    }

    pub fn dev_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.add_fs_options(FsOptions::Dev {
            destination: path.as_ref().as_os_str().to_os_string(),
            permission: None,
        })
    }

    pub fn dev_bind(&mut self, host: impl AsRef<Path>, guest: impl AsRef<Path>) -> &mut Self {
        self.add_fs_options(FsOptions::DevBind {
            source: host.as_ref().as_os_str().to_os_string(),
            destination: guest.as_ref().as_os_str().to_os_string(),
            permission: None,
            try_: false,
        })
    }

    pub fn try_dev_bind(&mut self, host: impl AsRef<Path>, guest: impl AsRef<Path>) -> &mut Self {
        self.add_fs_options(FsOptions::DevBind {
            source: host.as_ref().as_os_str().to_os_string(),
            destination: guest.as_ref().as_os_str().to_os_string(),
            permission: None,
            try_: true,
        })
    }

    pub fn tmpfs(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.add_fs_options(FsOptions::TempFs {
            destination: path.as_ref().as_os_str().to_os_string(),
            permission: None,
            size: None,
        })
    }

    pub fn dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.add_fs_options(FsOptions::Dir {
            destination: path.as_ref().as_os_str().to_os_string(),
            permission: None,
        })
    }

    pub fn symlink(
        &mut self,
        source: impl AsRef<Path>,
        destination: impl AsRef<Path>,
    ) -> &mut Self {
        self.add_fs_options(FsOptions::Symlink {
            destination: destination.as_ref().as_os_str().to_os_string(),
            source: source.as_ref().as_os_str().to_os_string(),
        })
    }

    pub fn file(&mut self, file: &'fd impl AsFd, destination: impl AsRef<Path>) -> &mut Self {
        self.add_fs_options(FsOptions::File {
            destination: destination.as_ref().as_os_str().to_os_string(),
            source: file.as_fd(),
            permission: None,
        })
    }

    pub fn data(&mut self, file: &'fd impl AsFd, destination: impl AsRef<Path>) -> &mut Self {
        self.add_fs_options(FsOptions::File {
            destination: destination.as_ref().as_os_str().to_os_string(),
            source: file.as_fd(),
            permission: None,
        })
    }
}

impl<'fd> BwrapCommand<'fd> {
    /// create an [`Vec<OsString>`] that will be the exact argument given to the bwrap binary
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

    #[must_use = "This is only the description of the command\nIt must be used to launch the program"]
    pub fn command(&mut self) -> std::process::Command {
        let mut cmd = std::process::Command::new("bwrap");
        cmd.args(self.build_args());
        cmd
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn env1() {
        let args = crate::BwrapCommand::new("echo")
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
        let args = crate::BwrapCommand::new("echo")
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
        let args = crate::BwrapCommand::new("echo")
            .set_cwd("/my/super/path")
            .build_args();
        assert_eq!(args, vec!["--chdir", "/my/super/path", "--", "echo"]);
    }
    #[test]
    fn cwd2() {
        let args = crate::BwrapCommand::new("echo")
            .set_cwd("/my/super/path")
            .set_cwd("/my/better/path")
            .build_args();
        assert_eq!(args, vec!["--chdir", "/my/better/path", "--", "echo"]);
    }
    #[test]
    fn cwd3() {
        let args = crate::BwrapCommand::new("echo")
            .set_cwd("/my/super/path")
            .set_cwd("/my/better/path")
            .unset_cwd()
            .build_args();
        assert_eq!(args, vec!["--", "echo"]);
    }
}
