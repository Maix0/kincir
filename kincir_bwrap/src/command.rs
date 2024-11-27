use std::ffi::{OsStr, OsString};
use std::{os::unix::ffi::OsStrExt, process::Stdio};

/// A process builder, providing fine-grained control
/// over how a new process should be spawned.
///
/// A default configuration can be generated using `Command::new(program)`,
/// where `program` gives a path to the program to be executed. Additional
/// builder methods allow the configuration to be changed (for example, by
/// adding arguments) prior to spawning:
///
/// ```no_run
/// # use kincir_bwrap::Command;
///
/// Command::new("sh").arg("-c").arg("echo hello");
/// ```
#[derive(Debug)]
pub struct Command {
    pub(crate) program: OsString,
    pub(crate) args: Vec<OsString>,
    pub(crate) stdin: Stdio,
    pub(crate) stdout: Stdio,
    pub(crate) stderr: Stdio,
}

impl Command {
    /// Constructs a new `Command` for launching the program at
    /// path `program`, with the following default configuration:
    ///
    /// * No arguments to the program
    /// * Inherit the current process's working directory
    /// * Inherit stdin/stdout/stderr
    ///
    /// Builder methods are provided to change these defaults and
    /// otherwise configure the process.
    ///
    /// If `program` is not an absolute path, the `PATH` will be searched in
    /// an OS-defined way.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_run
    /// # use kincir_bwrap::Command;
    ///
    /// Command::new("sh");
    /// ```
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        let program = program.as_ref().to_os_string();
        Self {
            program,
            stdout: Stdio::inherit(),
            stderr: Stdio::inherit(),
            stdin: Stdio::inherit(),
            args: Vec::default(),
        }
    }

    /// Adds an argument to pass to the program.
    ///
    /// Only one argument can be passed per use. So instead of:
    ///
    /// ```no_run
    /// # use kincir_bwrap::Command;
    /// # Command::new("sh")
    /// .arg("-C /path/to/repo")
    /// # ;
    /// ```
    ///
    /// usage would be:
    ///
    /// ```no_run
    /// # use kincir_bwrap::Command;
    /// # Command::new("sh")
    /// .arg("-C")
    /// .arg("/path/to/repo")
    /// # ;
    /// ```
    ///
    /// To pass multiple arguments see [`args`].
    ///
    /// [`args`]: Command::args
    ///
    /// Note that the argument is not passed through a shell, but given
    /// literally to the program. This means that shell syntax like quotes,
    /// escaped characters, word splitting, glob patterns, variable
    /// substitution, etc. have no effect.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_run
    /// # use kincir_bwrap::Command;
    ///
    /// Command::new("ls").arg("-l").arg("-a");
    /// ```
    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        let arg = arg.as_ref().to_os_string();
        self.args.push(arg);
        self
    }

    /// Adds multiple arguments to pass to the program.
    ///
    /// To pass a single argument see [`arg`].
    ///
    /// [`arg`]: Command::arg
    ///
    /// Note that the arguments are not passed through a shell, but given
    /// literally to the program. This means that shell syntax like quotes,
    /// escaped characters, word splitting, glob patterns, variable
    /// substitution, etc. have no effect.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_run
    /// # use kincir_bwrap::Command;
    ///
    /// Command::new("ls").args(["-l", "-a"]);
    /// ```
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for arg in args {
            let arg = arg.as_ref().to_os_string();
            self.args.push(arg);
        }
        self
    }

    /// Configuration for the child process's standard input (stdin) handle.
    ///
    /// Defaults to [`inherit`].
    ///
    /// [`inherit`]: Stdio::inherit
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_run
    /// # use kincir_bwrap::Command;
    /// # use std::process::Stdio;
    ///
    /// Command::new("ls").stdin(Stdio::null());
    /// ```
    pub fn stdin<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.stdin = cfg.into();
        self
    }

    /// Configuration for the child process's standard output (stdout) handle.
    ///
    /// Defaults to [`inherit`].
    ///
    /// [`inherit`]: Stdio::inherit
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_run
    /// # use kincir_bwrap::Command;
    /// # use std::process::Stdio;
    ///
    /// Command::new("ls").stdout(Stdio::null());
    /// ```
    pub fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.stdout = cfg.into();
        self
    }

    /// Configuration for the child process's standard error (stderr) handle.
    ///
    /// Defaults to [`inherit`].
    ///
    /// [`inherit`]: Stdio::inherit
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_run
    /// # use kincir_bwrap::Command;
    /// # use std::process::Stdio;
    ///
    /// Command::new("ls").stderr(Stdio::null());
    /// ```
    pub fn stderr<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.stderr = cfg.into();
        self
    }

    /// Returns the path to the program that was given to [`Command::new`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use kincir_bwrap::Command;
    /// # use std::process::Stdio;
    ///
    /// let cmd = Command::new("echo");
    /// assert_eq!(cmd.get_program(), "echo");
    /// ```
    #[must_use]
    pub fn get_program(&self) -> &OsStr {
        OsStr::from_bytes(self.program.as_bytes())
    }
}

impl From<Command> for std::process::Command {
    fn from(command: Command) -> Self {
        let mut std_command = std::process::Command::new(command.program);
        std_command.args(command.args);

        let stdin: Option<std::process::Stdio> = command.stdin.into();
        if let Some(stdin) = stdin {
            std_command.stdin(stdin);
        }

        let stdout: Option<std::process::Stdio> = command.stdout.into();
        if let Some(stdout) = stdout {
            std_command.stdout(stdout);
        }

        let stderr: Option<std::process::Stdio> = command.stderr.into();
        if let Some(stderr) = stderr {
            std_command.stderr(stderr);
        }

        std_command
    }
}

impl<T: AsRef<OsStr>> From<T> for Command {
    fn from(value: T) -> Self {
        Self {
            args: Vec::new(),
            program: value.as_ref().to_os_string(),
            stdin: Stdio::inherit(),
            stdout: Stdio::inherit(),
            stderr: Stdio::inherit(),
        }
    }
}
