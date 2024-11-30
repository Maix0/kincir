use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Duration,
};

use itertools::Itertools;
use tokio::time::Instant;

/// An instance of a runner.
/// This will allow the spawing of [`Run`]s
#[derive(Debug)]
struct Runner {
    /// The id of the runner, not equal on different instanciation of the program
    id: uuid::Uuid,
    /// the manifest as parsed from the manifest.yml
    manifest: RunnerManifest,
    /// the resolved binary dependencies
    ///
    /// they are formated like this: <bin_name> -> <host_location>
    ///
    /// they'll be present in /bin/<bin_name> when looking from inside the sandbox
    bin_deps: HashMap<String, PathBuf>,
    /// the resolved files dependencies
    ///
    /// note that they are fomated like this:
    ///
    /// <host_path> -> <guest_path>
    ///
    /// but the guest_path doesn't have the random directory prefixed here
    file_deps: HashMap<PathBuf, PathBuf>,
}

#[derive(Debug)]
struct RunOutput {
    trace: String,
    status: String,
    successful: bool,
}

/// The State of the [`Run`]
#[derive(Debug)]
enum RunState {
    /// The Run isn't yet launched
    NotLaunched,

    /// The run is started, the [`Instant`] represent when the run was started
    Running(Instant),

    /// The run was completed. [`RunOutput`] is given
    Complete(RunOutput),

    /// The [`Run`] timed out, and as such was killed. No [`RunOutput`] will be given in this case (could
    /// be corrupted)
    TimedOut,
}

/// A single run, associated with a specific runner.
#[derive(Debug)]
struct Run {
    /// The id of the [`Run`]. use this to refer to the Run istead of giving away references.
    id: uuid::Uuid,

    /// Show the user the trace that was created or not.
    show_trace: bool,

    /// The current state of the [`Run`]
    state: RunState,

    /// the runner associated with the [`Run`]
    runner_id: uuid::Uuid,
}

/// Describe an [`Runner`], which will then be able to execute [`Run`]s.
///
/// These should be written in `./runners/<name>/manifest.yml`.
///
/// Every runner will be loaded at startup, checked for missing depency.
///
/// If any error are found the service will fail before accepting any request
#[serde_with::serde_as]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RunnerManifest {
    /// Wheither the user will be given a trace to be shown or not.
    /// Do note that the `TRACE_FILE` envirment variable and the file associated with it will always be present.
    /// This will only changed the fact that the trace will be given to the user.
    pub show_trace: bool,

    /// The name of the runner
    pub name: String,

    /// List of program that should be avaiable in the $PATH of the sandbox
    /// This is optional, and if not set, then only coreutils and bash will be added to the `PATH`
    #[serde(default)]
    pub bin_deps: Vec<String>,

    /// List of path that will be included into the sandbox. these will be prefixed with a random
    /// hash and the root of folder will be given in the `FILES_ROOT` env var
    ///
    /// On the host side, the files will be located next to the manifest.yml, meaning that if they
    /// are absolute paths or relative path they will start at the directory containing the
    /// manifest.yml.
    ///
    /// It is an error to have path that contains the `..` directory inside them as it could lead
    /// to security issues.
    ///
    /// # Note
    ///
    /// This is optional, and if you truly don't need files to be passed into the sandbox, then it
    /// can be left out/not written in the manifest.
    ///
    /// All the files that will be passed as read only since other [`Run`]s may be launched
    /// afterwards, and there could be issues if the files are to be modified
    ///
    #[serde(default)]
    pub files_deps: HashMap<PathBuf, PathBuf>,

    /// The program that will be launched inside the sandbox (right after a simple wrapper that
    /// will do more work inside the sandbox such as limiting the number of processes to a
    /// reasonable limit).
    /// it will be given some envirment variable:
    /// - `FILES_ROOT`: the directory which will contain every requested files in the manifest
    ///
    /// - `SUBMITTED_ROOT`: the directory which will contain the submitted files (from the user, DO NOT TRUST !)
    ///
    /// - `PATH`: pretty obvious. Will only contain the needed binary specified in the sandbox plus some by default (coreutils and bash).
    ///
    /// - `TRACE_FILE`: Where you should log stuff if you want to give a trace
    ///
    /// For security, do note that these should NOT be passed down to the tested program.
    /// a utility named `safe-launch` can be used to call the program with these variable
    /// sanitized.
    pub entry: PathBuf,

    /// The time after which the sandbox (and every processes inside) will be killed.
    /// This defaults to 10s if not present
    #[serde(default = "RunnerManifest::default_timeout_value")]
    #[serde_as(as = "serde_with::DurationSeconds<u64, serde_with::formats::Flexible>")]
    pub timeout: Duration,

    /// Do not include default binaries into the $PATH
    /// by default this is false, meaning that if you do not specify a value it WILL include the
    /// default binaries
    ///
    /// the list of default binary are listed at [`RunnerManifest::DEFAULT_COMMANDS`]
    #[serde(default)]
    pub no_default_binary: bool,

    /// List of exit status and their meaning
    ///
    /// This key *is* optional and will default to no known exit code except for 0 which is a
    /// successful run.
    ///
    /// Do note that 0 will *ALWAYS* be a successful run.
    ///
    /// For example this could be something like:
    ///  - 0: Successful (by default, and implied. It is an error to have a manifest with a message for exitcode 0)
    ///
    ///  - 1: missing files
    ///
    ///  - 2: Code formatting error
    ///
    ///  - 3: Doesn't compile
    ///
    ///  - 4: Using illegal functions
    #[serde(default)]
    pub exit_status: HashMap<i32, String>,
}

impl RunnerManifest {
    /// All the binaries that will be installed by default onto the sandbox
    pub const DEFAULT_COMMANDS: &[&str] = &[
        "[",
        "arch",
        "b2sum",
        "base32",
        "base64",
        "basename",
        "basenc",
        "bash",
        "cat",
        "chcon",
        "chgrp",
        "chmod",
        "chown",
        "chroot",
        "cksum",
        "comm",
        "cp",
        "csplit",
        "cut",
        "date",
        "dd",
        "df",
        "dir",
        "dircolors",
        "dirname",
        "du",
        "echo",
        "env",
        "expand",
        "expr",
        "factor",
        "false",
        "fmt",
        "fold",
        "groups",
        "head",
        "hostid",
        "id",
        "install",
        "join",
        "link",
        "ln",
        "logname",
        "ls",
        "md5sum",
        "mkdir",
        "mkfifo",
        "mknod",
        "mktemp",
        "mv",
        "nice",
        "nl",
        "nohup",
        "nproc",
        "numfmt",
        "od",
        "paste",
        "pathchk",
        "pinky",
        "pr",
        "printenv",
        "printf",
        "ptx",
        "pwd",
        "readlink",
        "realpath",
        "rm",
        "rmdir",
        "runcon",
        "seq",
        "sha1sum",
        "sha224sum",
        "sha256sum",
        "sha384sum",
        "sha512sum",
        "shred",
        "shuf",
        "sleep",
        "sort",
        "split",
        "stat",
        "stdbuf",
        "stty",
        "sum",
        "sync",
        "tac",
        "tail",
        "tee",
        "test",
        "timeout",
        "touch",
        "tr",
        "true",
        "truncate",
        "tsort",
        "tty",
        "uname",
        "unexpand",
        "uniq",
        "unlink",
        "uptime",
        "users",
        "vdir",
        "wc",
        "who",
        "whoami",
        "yes",
    ];

    /// The default timeout value. Used by serde if the value is not specified in the manifest
    fn default_timeout_value() -> Duration {
        Duration::from_secs(10)
    }

    pub fn verify_bin_deps(&self) -> Result<HashMap<String, PathBuf>, RunnerBinaryDepError<'_>> {
        let mut output = HashMap::with_capacity(self.bin_deps.len());
        for bin in &self.bin_deps {
            if output
                .insert(
                    bin.clone(),
                    which::which(bin).map_err(|e| RunnerBinaryDepError::WhichError(bin, e))?,
                )
                .is_some()
            {
                return Err(RunnerBinaryDepError::Duplicate(bin));
            }
        }
        for &bin in Self::DEFAULT_COMMANDS {
            if !output.contains_key(bin) {
                output.insert(
                    bin.to_string(),
                    which::which(bin).map_err(|e| RunnerBinaryDepError::WhichError(bin, e))?,
                );
            }
        }
        Ok(output)
    }

    pub fn verify_files_deps(&self) -> Result<HashMap<PathBuf, PathBuf>, RunnerFilesDepError<'_>> {
        let mut out = HashMap::with_capacity(self.files_deps.len());
        let duplicates = self
            .files_deps
            .values()
            .duplicates()
            .map(|p| p.as_path())
            .collect::<Vec<_>>();
        if !duplicates.is_empty() {
            return Err(RunnerFilesDepError::Duplicates(duplicates));
        }
        for (host_path, guest_path) in &self.files_deps {
            let mut host_real_path = std::ffi::OsString::from(format!("./runners/{}/", self.name));
            host_real_path.push(host_path);
            let host_real_path = PathBuf::from(host_real_path);
            if !host_real_path.exists() {
                return Err(RunnerFilesDepError::Missing(host_path.as_path()));
            }
            if host_path.components().any(|s| {
                matches!(
                    s,
                    std::path::Component::ParentDir | std::path::Component::Prefix(_)
                )
            }) {
                return Err(RunnerFilesDepError::InvalidPath(host_path.as_path()));
            }
            if guest_path.components().any(|s| {
                matches!(
                    s,
                    std::path::Component::ParentDir | std::path::Component::Prefix(_)
                )
            }) {
                return Err(RunnerFilesDepError::InvalidPath(guest_path.as_path()));
            }
            out.insert(host_real_path, guest_path.clone());
        }
        Ok(out)
    }
}

#[derive(Debug)]
pub enum RunnerBinaryDepError<'a> {
    Duplicate(&'a str),
    WhichError(&'a str, which::Error),
}

#[derive(Debug)]
pub enum RunnerFilesDepError<'a> {
    Duplicates(Vec<&'a Path>),
    Missing(&'a Path),
    InvalidPath(&'a Path),
}

impl<'a> std::fmt::Display for RunnerBinaryDepError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Duplicate(bin) => writeln!(f, "duplicate binary dependency for: `{bin}`"),
            Self::WhichError(bin, e) => writeln!(f, "which error for dependency `{bin}`: {e}"),
        }
    }
}

impl<'a> std::error::Error for RunnerBinaryDepError<'a> {}

impl<'a> std::fmt::Display for RunnerFilesDepError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Duplicates(paths) => {
                writeln!(f, "duplicate guest paths")?;
                for p in paths {
                    writeln!(f, "- {}", p.display())?;
                }
                Ok(())
            }
            RunnerFilesDepError::Missing(p) => writeln!(f, "missing path: {}", p.display()),
            RunnerFilesDepError::InvalidPath(p) => writeln!(f, "invalid path {}", p.display()),
        }
    }
}
