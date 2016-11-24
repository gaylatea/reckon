//! Low-level subprocess management.
//!
//! This is about as raw an implementation as it gets - it's a thin, blocking I/O layer over Rust's
//! built-in subprocess tools. It might be the simplest version of "expect" I've yet seen aside
//! from some Bash scripts.
use std::io::{Error, ErrorKind};
use std::io::prelude::*;
use std::process::{Command, Stdio, Child};
use std::result::Result;
use std::time::Instant;

use regex::RegexSet;

// Rexport this for calling programs.
pub use std::time::Duration;

/// Provids necessary lifetime management for subprocess resources.
pub struct Process {
    child: Child,
}

impl Process {
    /// Starts a subprocess for later interaction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use reckon::base::Process;
    /// Process::new("true", vec![]).expect("Your UNIX is broken.");
    /// ```
    ///
    /// Processes that cannot be invoked will denote this in a way that can be handled properly:
    ///
    /// ```rust,should_panic
    /// # use reckon::base::Process;
    /// Process::new("nope-i-don't-exist", vec![]).expect("This should fail.");
    /// ```
    pub fn new(exe: &str, args: Vec<&str>) -> Result<Process, Error> {
        let command = Command::new(exe)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        match command {
            Ok(c) => Ok(Process { child: c }),
            Err(v) => Err(v),
        }
    }

    /// Writes some data to the subprocess.
    ///
    /// This doesn't do any special processing of the data; it just shovels it onto the subprocess
    /// as fast as it can, and propagates any errors that occurred during this operation.
    ///
    /// Of note is the fact that `reckon` assumes that strings are being sent betwixt processes;
    /// future support for raw bytes might come if needed.
    ///
    /// ```rust
    /// # use reckon::base::Process;
    /// # use std::time::Duration;
    /// let mut p = Process::new("cat", vec![]).unwrap();
    /// p.emit("Hello");
    /// # let (m, _) = p.expect(vec!["Hello"], Duration::from_secs(1)).unwrap();
    /// # assert_eq!(m, 0);
    /// ```
    pub fn emit(&mut self, data: &str) -> Result<(), Error> {
        let mut stdin = self.child.stdin.as_mut().unwrap();
        stdin.write_all(data.as_bytes())
    }

    /// Searches for some marker in data from the subprocess.
    ///
    /// This follows a similar format to other `libexpect`-alikes; you can specify a set of regular
    /// expressions to try and match on, and it will return back which one of them matched first.
    ///
    /// Seemingly unique to this particular implementation is that it always returns the data that
    /// matched, for later processing/matching by callers, without having to keep a buffer around
    /// after the call.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use reckon::base::Process;
    /// # use std::time::Duration;
    /// let mut p = Process::new("bash", vec!["test.sh"]).unwrap();
    /// let (m, _) = p.expect(vec!["Hello"], Duration::from_secs(1)).unwrap();
    /// # assert_eq!(m, 0);
    /// ```
    ///
    /// The matcher supports timeouts, as well:
    ///
    /// ```rust,should_panic
    /// # use reckon::base::Process;
    /// # use std::time::Duration;
    /// # let mut p = Process::new("bash", vec!["test.sh"]).unwrap();
    /// p.expect(vec!["Hello"], Duration::from_secs(0)).unwrap();
    /// ```
    pub fn expect(&mut self,
                  needles: Vec<&str>,
                  timeout: Duration)
                  -> Result<(usize, String), Error> {
        let start_time = Instant::now();

        let stdout = self.child.stdout.as_mut().unwrap();
        let rs = RegexSet::new(&needles).unwrap();

        let mut b = String::new();
        let mut c = stdout.chars();
        loop {
            let e = start_time.elapsed();
            if e >= timeout {
                break;
            }

            b.push(c.next().unwrap().unwrap());

            for n in rs.matches(&b).into_iter() {
                return Ok((n, b));
            }
        }

        return Err(Error::new(ErrorKind::TimedOut, ""));
    }
}

impl Drop for Process {
    /// Destructor to automatically clean up the subprocess.
    ///
    /// This prevents the child process sticking around when the parent dies, which apparently can
    /// happen when you capture all `std{io,err,out}` pipes.
    fn drop(&mut self) {
        self.child.wait().expect("could not kill the process!");
    }
}
