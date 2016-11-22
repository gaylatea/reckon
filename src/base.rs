//! Low-level subprocess management.
//!
//! This is about as raw an implementation as it gets - it's a thin, blocking
//! I/O layer over Rust's built-in subprocess tools. It might be the simplest
//! version of "expect" I've yet seen aside from some Bash scripts.
use std::result::Result;
use std::process::{Command, Stdio, Child};
use std::io::Error;
use std::io::prelude::*;
use std::time::{Duration, Instant};

use regex::{RegexSet};

/// Necessary lifetime management for subprocess resources.
///
/// In faith, I'm not sure why this works better for the borrow checker than
/// module-level functions, but I'll take it.
pub struct Process {
    child: Child,
}

impl Process {
    /// Start a subprocess for later interaction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use reckon::base::Process;
    /// Process::new("true", vec![]);
    /// ```
    ///
    /// Processes that cannot be invoked will denote this in a way that
    /// can be handled properly:
    ///
    /// ```rust
    /// # use reckon::base::Process;
    /// match Process::new("nope-i-don't-exist", vec![]) {
    ///     Ok(_)  => panic!("This really should never happen."),
    ///     Err(_) => println!("This is expected."),
    /// }
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

    /// Write some data to the subprocess.
    ///
    /// This doesn't do any special processing of the data; it just shovels it
    /// onto the subprocess as fast as it can, and propagates any errors that
    /// occurred during this operation.
    pub fn emit(&mut self, data: &str) -> Result<(), Error> {
        let mut stdin = self.child.stdin.as_mut().unwrap();
        stdin.write_all(data.as_bytes())
    }

    /// Search for some marker in data from the subprocess.
    ///
    /// This follows a similar format to other `libexpect`-alikes; you can
    /// specify a set of regular expressions to try and match on, and it will
    /// return back which one of them matched first.
    ///
    /// Seemingly unique to this particular implementation is that it always
    /// returns the data that matched, for later processing/matching by
    /// callers, without having to keep a buffer around after the call.
    ///
    /// # Examples
    /// TODO(silversupreme): Fill this in.
    pub fn expect(&mut self, needles: Vec<&str>, timeout: Duration) -> (usize, String) {
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
                return (n, b);
            }
        }

        return (0, String::from(""));
    }
}

impl Drop for Process {
    /// Destructor to automatically clean up the subprocess.
    ///
    /// This prevents the child process sticking around when the parent
    /// dies, which apparently can happen when you capture all `std{io,err,out}`
    /// pipes.
    fn drop(&mut self) {
        self.child.wait().expect("could not kill the process!");
    }
}
