//! File-descriptor evidence: where the player's command sent stdout and stderr.
//!
//! Section 10 teaches redirection, so the game must see what `./buggy.sh &> /dev/null`
//! did to the script's output. The shell runs a tiny hook *inside* the redirected command,
//! and the hook looks at its own file descriptors 1 and 2. Comparing device and inode
//! numbers with those of `/dev/null` needs no `/proc`, so it works on macOS too.

use std::io;
use std::os::fd::AsFd;

/// Identity of whatever a file descriptor points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FdTarget {
    /// The device holding the file (`st_dev`).
    pub dev: u64,
    /// The inode number (`st_ino`).
    pub ino: u64,
    /// For a device node, the device it stands for (`st_rdev`); 0 otherwise.
    pub rdev: u64,
}

impl FdTarget {
    /// Inspect an open file descriptor.
    pub fn from_fd(fd: impl AsFd) -> io::Result<Self> {
        Ok(Self::from_stat(&rustix::fs::fstat(fd)?))
    }

    /// Inspect a path (following symlinks).
    pub fn from_path(path: &str) -> io::Result<Self> {
        Ok(Self::from_stat(&rustix::fs::stat(path)?))
    }

    // The field types of `Stat` differ between platforms (`i32` device numbers on macOS),
    // so the casts are needed there even though they are no-ops on Linux.
    #[allow(clippy::unnecessary_cast)]
    fn from_stat(stat: &rustix::fs::Stat) -> Self {
        Self {
            dev: stat.st_dev as u64,
            ino: stat.st_ino as u64,
            rdev: stat.st_rdev as u64,
        }
    }

    /// True when this is `/dev/null`: the same inode, or the same (non-zero) device number.
    /// False when `/dev/null` cannot be inspected.
    pub fn is_dev_null(&self) -> bool {
        let Ok(null) = Self::from_path("/dev/null") else {
            return false;
        };
        (self.dev == null.dev && self.ino == null.ino) || (self.rdev != 0 && self.rdev == null.rdev)
    }
}

/// Where stdout and stderr of the hooked command point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fds {
    /// File descriptor 1.
    pub stdout: FdTarget,
    /// File descriptor 2.
    pub stderr: FdTarget,
}

impl Fds {
    /// Inspect this process's own file descriptors 1 and 2.
    pub fn from_process() -> io::Result<Self> {
        Ok(Self {
            stdout: FdTarget::from_fd(rustix::stdio::stdout())?,
            stderr: FdTarget::from_fd(rustix::stdio::stderr())?,
        })
    }

    /// True when stdout goes to `/dev/null`.
    pub fn stdout_is_null(&self) -> bool {
        self.stdout.is_dev_null()
    }

    /// True when stderr goes to `/dev/null`.
    pub fn stderr_is_null(&self) -> bool {
        self.stderr.is_dev_null()
    }

    /// True when both streams go to `/dev/null`, as `&> /dev/null` does.
    pub fn both_null(&self) -> bool {
        self.stdout_is_null() && self.stderr_is_null()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    fn target_of(file: &File) -> FdTarget {
        FdTarget::from_fd(file).unwrap()
    }

    #[test]
    fn dev_null_is_recognised_through_any_descriptor_on_it() {
        let null = File::options().write(true).open("/dev/null").unwrap();
        assert!(target_of(&null).is_dev_null());
        let again = File::open("/dev/null").unwrap();
        assert!(target_of(&again).is_dev_null());
    }

    #[test]
    fn regular_files_pipes_and_directories_are_not_dev_null() {
        let dir = tempfile::tempdir().unwrap();
        let file = File::create(dir.path().join("out.log")).unwrap();
        assert!(!target_of(&file).is_dev_null());
        let directory = File::open(dir.path()).unwrap();
        assert!(!target_of(&directory).is_dev_null());
        let (reader, writer) = std::io::pipe().unwrap();
        assert!(!FdTarget::from_fd(&reader).unwrap().is_dev_null());
        assert!(!FdTarget::from_fd(&writer).unwrap().is_dev_null());
    }

    #[test]
    fn a_zero_rdev_never_matches_by_device_number() {
        let target = FdTarget {
            dev: 1,
            ino: 2,
            rdev: 0,
        };
        assert!(!target.is_dev_null());
    }

    #[test]
    fn fds_report_each_stream_separately() {
        let dir = tempfile::tempdir().unwrap();
        let null = File::options().write(true).open("/dev/null").unwrap();
        let log = File::create(dir.path().join("errors.log")).unwrap();
        let fds = Fds {
            stdout: target_of(&null),
            stderr: target_of(&log),
        };
        assert!(fds.stdout_is_null());
        assert!(!fds.stderr_is_null());
        assert!(!fds.both_null());

        let both = Fds {
            stdout: target_of(&null),
            stderr: target_of(&null),
        };
        assert!(both.both_null());
    }

    #[test]
    fn the_process_descriptors_can_be_inspected() {
        // Whatever the test harness did with fds 1 and 2, inspecting them works.
        let fds = Fds::from_process().unwrap();
        let _ = fds.both_null();
    }

    #[test]
    fn from_path_reads_the_same_identity_as_from_fd() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("f");
        let file = File::create(&path).unwrap();
        assert_eq!(
            FdTarget::from_path(path.to_str().unwrap()).unwrap(),
            target_of(&file)
        );
        assert!(FdTarget::from_path("/nonexistent/path/xyz").is_err());
    }
}
