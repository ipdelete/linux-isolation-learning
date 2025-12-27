//! Error types for ns-tool
//!
//! This module demonstrates idiomatic Rust error handling for systems programming:
//! - Custom error types using thiserror for precise error matching
//! - Automatic conversion from syscall errors (nix::Error, std::io::Error)
//! - Contextual information to help users understand what failed
//!
//! # Error Design Principles
//!
//! 1. **Be specific**: "Failed to create PID namespace" > "syscall failed"
//! 2. **Include context**: Which namespace? Which file path? What operation?
//! 3. **Preserve the cause**: Chain errors so users can see the root cause
//! 4. **Suggest fixes**: When possible, hint at solutions (e.g., "try running as root")
//!
//! # Example
//!
//! ```rust,ignore
//! use ns_tool::{NsError, NamespaceKind, NsResult};
//! use nix::sched::{unshare, CloneFlags};
//!
//! fn create_pid_namespace() -> NsResult<()> {
//!     unshare(CloneFlags::CLONE_NEWPID)
//!         .map_err(|e| NsError::create_namespace(NamespaceKind::Pid, e))
//! }
//! ```

use std::path::PathBuf;
use thiserror::Error;

/// The namespace types we work with
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamespaceKind {
    Pid,
    Uts,
    Ipc,
    Mount,
    Net,
    User,
    Cgroup,
    Time,
}

impl std::fmt::Display for NamespaceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NamespaceKind::Pid => write!(f, "PID"),
            NamespaceKind::Uts => write!(f, "UTS"),
            NamespaceKind::Ipc => write!(f, "IPC"),
            NamespaceKind::Mount => write!(f, "mount"),
            NamespaceKind::Net => write!(f, "network"),
            NamespaceKind::User => write!(f, "user"),
            NamespaceKind::Cgroup => write!(f, "cgroup"),
            NamespaceKind::Time => write!(f, "time"),
        }
    }
}

/// Errors that can occur when working with namespaces
#[derive(Debug, Error)]
pub enum NsError {
    /// Failed to create a new namespace with unshare(2)
    #[error("failed to create {kind} namespace")]
    CreateNamespace {
        kind: NamespaceKind,
        #[source]
        source: nix::Error,
    },

    /// Failed to join an existing namespace with setns(2)
    #[error("failed to join {kind} namespace from {path}")]
    JoinNamespace {
        kind: NamespaceKind,
        path: PathBuf,
        #[source]
        source: nix::Error,
    },

    /// Failed to fork a child process
    #[error("failed to fork child process")]
    Fork(#[source] nix::Error),

    /// Failed to read from /proc filesystem
    #[error("failed to read {path}")]
    ProcRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to set hostname (UTS namespace operation)
    #[error("failed to set hostname to '{hostname}'")]
    SetHostname {
        hostname: String,
        #[source]
        source: nix::Error,
    },

    /// Operation requires root privileges
    #[error("{operation} requires root privileges (try: sudo)")]
    PermissionDenied { operation: String },

    /// A namespace file does not exist
    #[error("namespace file not found: {path}")]
    NamespaceNotFound { path: PathBuf },
}

impl NsError {
    /// Create a CreateNamespace error with the given kind and source
    ///
    /// This constructor intelligently converts EPERM/EACCES errors into
    /// a PermissionDenied variant with a helpful message.
    pub fn create_namespace(kind: NamespaceKind, source: nix::Error) -> Self {
        // Check if this is a permission error and provide a better message
        if source == nix::Error::EPERM || source == nix::Error::EACCES {
            return NsError::PermissionDenied {
                operation: format!("creating {} namespace", kind),
            };
        }
        NsError::CreateNamespace { kind, source }
    }

    /// Create a JoinNamespace error
    ///
    /// This constructor handles common error cases:
    /// - EPERM/EACCES -> PermissionDenied with helpful message
    /// - ENOENT -> NamespaceNotFound with the path
    pub fn join_namespace(kind: NamespaceKind, path: PathBuf, source: nix::Error) -> Self {
        if source == nix::Error::EPERM || source == nix::Error::EACCES {
            return NsError::PermissionDenied {
                operation: format!("joining {} namespace", kind),
            };
        }
        if source == nix::Error::ENOENT {
            return NsError::NamespaceNotFound { path };
        }
        NsError::JoinNamespace { kind, path, source }
    }

    /// Create a ProcRead error
    pub fn proc_read(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        NsError::ProcRead {
            path: path.into(),
            source,
        }
    }

    /// Create a Fork error
    pub fn fork(source: nix::Error) -> Self {
        if source == nix::Error::EPERM {
            return NsError::PermissionDenied {
                operation: "forking child process".to_string(),
            };
        }
        NsError::Fork(source)
    }

    /// Create a SetHostname error
    pub fn set_hostname(hostname: impl Into<String>, source: nix::Error) -> Self {
        if source == nix::Error::EPERM {
            return NsError::PermissionDenied {
                operation: "setting hostname".to_string(),
            };
        }
        NsError::SetHostname {
            hostname: hostname.into(),
            source,
        }
    }
}

/// Convenience type alias for functions that return our error type
pub type NsResult<T> = Result<T, NsError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_kind_display() {
        assert_eq!(NamespaceKind::Pid.to_string(), "PID");
        assert_eq!(NamespaceKind::Net.to_string(), "network");
        assert_eq!(NamespaceKind::Mount.to_string(), "mount");
        assert_eq!(NamespaceKind::Uts.to_string(), "UTS");
        assert_eq!(NamespaceKind::Ipc.to_string(), "IPC");
        assert_eq!(NamespaceKind::User.to_string(), "user");
        assert_eq!(NamespaceKind::Cgroup.to_string(), "cgroup");
        assert_eq!(NamespaceKind::Time.to_string(), "time");
    }

    #[test]
    fn test_create_namespace_error_display() {
        let err = NsError::CreateNamespace {
            kind: NamespaceKind::Pid,
            source: nix::Error::EINVAL,
        };
        assert_eq!(err.to_string(), "failed to create PID namespace");
    }

    #[test]
    fn test_permission_error_suggests_sudo() {
        let err = NsError::PermissionDenied {
            operation: "creating PID namespace".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("root"), "Message should mention root: {}", msg);
        assert!(msg.contains("sudo"), "Message should mention sudo: {}", msg);
    }

    #[test]
    fn test_proc_read_error_includes_path() {
        let err = NsError::proc_read(
            "/proc/self/ns/pid",
            std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        );
        let msg = err.to_string();
        assert!(
            msg.contains("/proc/self/ns/pid"),
            "Message should include path: {}",
            msg
        );
    }

    #[test]
    fn test_eperm_becomes_permission_denied() {
        let err = NsError::create_namespace(NamespaceKind::Pid, nix::Error::EPERM);
        match err {
            NsError::PermissionDenied { operation } => {
                assert!(
                    operation.contains("PID"),
                    "Operation should mention PID: {}",
                    operation
                );
            }
            _ => panic!("Expected PermissionDenied, got {:?}", err),
        }
    }

    #[test]
    fn test_eacces_becomes_permission_denied() {
        let err = NsError::create_namespace(NamespaceKind::Net, nix::Error::EACCES);
        match err {
            NsError::PermissionDenied { operation } => {
                assert!(
                    operation.contains("network"),
                    "Operation should mention network: {}",
                    operation
                );
            }
            _ => panic!("Expected PermissionDenied, got {:?}", err),
        }
    }

    #[test]
    fn test_join_namespace_error_with_path() {
        let err = NsError::JoinNamespace {
            kind: NamespaceKind::Net,
            path: PathBuf::from("/proc/1234/ns/net"),
            source: nix::Error::EINVAL,
        };
        let msg = err.to_string();
        assert!(
            msg.contains("network"),
            "Message should mention network: {}",
            msg
        );
        assert!(
            msg.contains("/proc/1234/ns/net"),
            "Message should include path: {}",
            msg
        );
    }

    #[test]
    fn test_join_namespace_enoent_becomes_not_found() {
        let path = PathBuf::from("/proc/99999/ns/pid");
        let err = NsError::join_namespace(NamespaceKind::Pid, path.clone(), nix::Error::ENOENT);
        match err {
            NsError::NamespaceNotFound { path: p } => {
                assert_eq!(p, path);
            }
            _ => panic!("Expected NamespaceNotFound, got {:?}", err),
        }
    }

    #[test]
    fn test_fork_error() {
        let err = NsError::fork(nix::Error::EAGAIN);
        match err {
            NsError::Fork(source) => {
                assert_eq!(source, nix::Error::EAGAIN);
            }
            _ => panic!("Expected Fork, got {:?}", err),
        }
    }

    #[test]
    fn test_set_hostname_error() {
        let err = NsError::set_hostname("test-host", nix::Error::EINVAL);
        match err {
            NsError::SetHostname { hostname, source } => {
                assert_eq!(hostname, "test-host");
                assert_eq!(source, nix::Error::EINVAL);
            }
            _ => panic!("Expected SetHostname, got {:?}", err),
        }
    }

    #[test]
    fn test_error_source_chain() {
        use std::error::Error;

        let err = NsError::CreateNamespace {
            kind: NamespaceKind::Pid,
            source: nix::Error::EINVAL,
        };

        // The source() method should return the underlying nix::Error
        let source = err.source();
        assert!(source.is_some(), "Error should have a source");
    }
}
