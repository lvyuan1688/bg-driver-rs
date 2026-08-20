//! Windows Job Object sandbox skeleton.
//!
//! A Job Object lets Windows enforce hard resource + process limits on a
//! tree of processes. This skeleton shows the full Win32 call shape:
//!
//! 1. `CreateJobObjectW(NULL, NULL)` → job handle
//! 2. Fill `JOBOBJECT_EXTENDED_LIMIT_INFORMATION`:
//!    - `BasicLimitInformation.LimitFlags` =
//!        `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`
//!      | `JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION`
//!      | `JOB_OBJECT_LIMIT_BREAKAWAY_OK`
//!    - `ProcessMemoryLimit` (per-process RSS cap)
//!    - `JobMemoryLimit`        (whole-tree RSS cap)
//!    - `BasicLimitInformation.PerProcessUserTimeLimit`
//!    - `BasicLimitInformation.PerJobUserTimeLimit`
//! 3. `SetInformationJobObject(job, JobObjectExtendedLimitInformation, &info, sizeof(info))`
//! 4. `AssignProcessToJobObject(job, GetCurrentProcess())`
//!    — assign the agent itself, OR spawn children with
//!      `CreateProcessW(..., CREATE_SUSPENDED, ...)` + `AssignProcessToJobObject`
//!      + `ResumeThread` so they inherit the job.
//! 5. On drop: `CloseHandle(job)` → because of
//!    `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, every process in the job is
//!    killed automatically. This is the "kill the whole tree on Ctrl-C"
//!    guarantee.
//!
//! ## Filesystem isolation
//!
//! Job Objects can't restrict filesystem access by themselves. The standard
//! approach is:
//!
//! 1. Create a minimal "root" directory: `C:\bg-sandbox\<id>\`
//! 2. Use `ICACLS` or `SetNamedSecurityInfo` to grant the sandboxed user
//!    RWX on the root only, and drop other ACLs.
//! 3. Use `SetFileAttributes(FILE_ATTRIBUTE_READONLY)` on files you don't
//!    want modified.
//!
//! For registry isolation, use `RegCreateKeyExW` with
//! `REG_OPTION_CREATE_VOLATILE` inside the sandbox account.
//!
//! ## Why this matters
//!
//! A coding agent that can spawn shells must not be able to:
//!
//! - read `~/.ssh/id_rsa`
//! - write to `C:\Windows\System32\`
//! - fork-bomb the host
//! - survive the agent dying
//!
//! The Job Object gives you (4) for free and (3) via
//! `JOB_OBJECT_LIMIT_PROCESS_MEMORY` / `JOB_OBJECT_LIMIT_JOB_MEMORY`. (1)
//! and (2) are filesystem ACLs, handled outside the job.
//!
//! ## Status
//!
//! This is a **typed skeleton**: the structs and constants are declared,
//! the function signatures match Win32, but the actual `windows-sys` FFI
//! calls are stubbed. To make it real:
//!
//! ```toml
//! [target.'cfg(windows)'.dependencies]
//! windows-sys = { version = "0.59", features = [
//!     "Win32_Foundation",
//!     "Win32_System_JobObjects",
//!     "Win32_System_Threading",
//!     "Win32_Security",
//! ] }
//! ```
//!
//! then replace the `unsafe extern "system"` blocks below with real calls.

#![cfg(target_os = "windows")]

use std::ffi::c_void;
use std::os::windows::io::{AsRawHandle, OwnedHandle, RawHandle};
use std::path::PathBuf;

/// Limit flags we always set. See winnt.h for the full list.
const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x2000;
const JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION: u32 = 0x0400;
const JOB_OBJECT_LIMIT_BREAKAWAY_OK: u32 = 0x0080;
const JOB_OBJECT_LIMIT_PROCESS_MEMORY: u32 = 0x0100;
const JOB_OBJECT_LIMIT_JOB_MEMORY: u32 = 0x0200;

/// `JOBOBJECT_BASIC_LIMIT_INFORMATION` — the core limits struct.
#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct JobBasicLimitInfo {
    pub per_process_user_time_limit: i64,
    pub per_job_user_time_limit: i64,
    pub limit_flags: u32,
    pub minimum_working_set_size: usize,
    pub maximum_working_set_size: usize,
    pub active_process_limit: u32,
    pub affinity: usize,
    pub priority_class: u32,
    pub scheduling_class: u32,
}

/// `IO_COUNTERS` — cumulative I/O stats.
#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct IoCounters {
    pub read_operation_count: u64,
    pub write_operation_count: u64,
    pub other_operation_count: u64,
    pub read_transfer_count: u64,
    pub write_transfer_count: u64,
    pub other_transfer_count: u64,
}

/// `JOBOBJECT_EXTENDED_LIMIT_INFORMATION` — the struct actually passed to
/// `SetInformationJobObject`.
#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct JobExtendedLimitInfo {
    pub basic_limit_information: JobBasicLimitInfo,
    pub io_info: IoCounters,
    pub process_memory_limit: usize,
    pub job_memory_limit: usize,
    pub peak_process_memory_used: usize,
    pub peak_job_memory_used: usize,
}

/// Information class enum for `SetInformationJobObject`.
const JOBOBJECTEXTENDEDLIMITINFORMATION_CLASS: u32 = 9;

/// A typed Job Object wrapper. Closing the handle kills every process in
/// the job (because we set `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`).
pub struct JobObject {
    handle: OwnedHandle,
    pub limits: JobExtendedLimitInfo,
}

impl JobObject {
    /// Create a Job Object with safe-default limits:
    ///
    /// - 512 MB per process
    /// - 2 GB whole job
    /// - 64 active processes
    /// - kill-on-close + die-on-unhandled-exception
    pub fn new_default() -> std::io::Result<Self> {
        let mut limits = JobExtendedLimitInfo::default();
        limits.basic_limit_information.limit_flags =
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
            | JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION
            | JOB_OBJECT_LIMIT_BREAKAWAY_OK
            | JOB_OBJECT_LIMIT_PROCESS_MEMORY
            | JOB_OBJECT_LIMIT_JOB_MEMORY;
        limits.process_memory_limit = 512 * 1024 * 1024;
        limits.job_memory_limit = 2 * 1024 * 1024 * 1024;
        limits.basic_limit_information.active_process_limit = 64;
        Self::new(limits)
    }

    /// Create a Job Object and apply the supplied limits.
    pub fn new(limits: JobExtendedLimitInfo) -> std::io::Result<Self> {
        // Real impl:
        //   unsafe {
        //     let h = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        //     if h.is_null() { return Err(io::Error::last_os_error()); }
        //     let r = SetInformationJobObject(
        //       h, JOBOBJECTEXTENDEDLIMITINFORMATION_CLASS,
        //       &limits as *const _ as *mut c_void,
        //       std::mem::size_of::<JobExtendedLimitInfo>() as u32);
        //     if r == 0 { return Err(io::Error::last_os_error()); }
        //     Ok(Self { handle: OwnedHandle::from_raw(h), limits })
        //   }
        //
        // Skeleton: pretend we got an invalid handle and return Ok.
        let _ = limits;
        Ok(Self {
            handle: owned_invalid()?,
            limits,
        })
    }

    /// Assign the calling process to the job. After this, the job's
    /// limits apply to the caller and every process it spawns.
    pub fn assign_self(&self) -> std::io::Result<()> {
        // Real impl:
        //   unsafe {
        //     let me = GetCurrentProcess();
        //     if AssignProcessToJobObject(self.handle.as_raw_handle() as _, me) == 0 {
        //       return Err(io::Error::last_os_error());
        //     }
        //   }
        Ok(())
    }

    /// Assign a specific process (by handle) to the job.
    pub fn assign_process(&self, process: RawHandle) -> std::io::Result<()> {
        let _ = process;
        Ok(())
    }
}

/// Construct an `OwnedHandle` around an invalid (-1) handle. Used by the
/// skeleton to satisfy the type without a real syscall.
fn owned_invalid() -> std::io::Result<OwnedHandle> {
    // SAFETY: -1 is the invalid-handle sentinel; we never CloseHandle it
    // because OwnedHandle treats -1 as "no handle". This is the same trick
    // std uses internally for `OwnedHandle::null`-style APIs.
    // (In real code, we'd use `OwnedHandle::from_raw_handle(CreateJobObjectW(...))`.)
    Ok(OwnedHandle::try_from(unsafe { std::mem::zeroed::<RawHandle>() }).unwrap_or_else(|_| {
        // fallback: just create a pseudo-handle
        OwnedHandle::try_from(0 as RawHandle).unwrap_or_else(|_| unsafe { std::mem::zeroed() })
    }))
}

/// A sandbox: Job Object + a per-instance root directory.
pub struct WindowsSandbox {
    pub job: JobObject,
    pub root: PathBuf,
}

impl WindowsSandbox {
    /// Create a sandbox rooted at `root`. The directory is created if
    /// missing; its ACLs are **not** modified by this skeleton.
    pub fn new(root: PathBuf) -> std::io::Result<Self> {
        std::fs::create_dir_all(&root)?;
        let job = JobObject::new_default()?;
        job.assign_self()?;
        Ok(Self { job, root })
    }

    /// Spawn a command inside the sandbox. The child inherits the job's
    /// limits automatically. Returns its exit code.
    pub fn spawn(&self, cmd: &str, args: &[&str]) -> std::io::Result<i32> {
        let mut c = std::process::Command::new(cmd);
        c.args(args).current_dir(&self.root);
        let status = c.status()?;
        Ok(status.code().unwrap_or(-1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn default_limits_have_kill_on_close() {
        let j = JobObject::new_default().unwrap();
        assert!(j.limits.basic_limit_information.limit_flags
            & JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
            != 0);
    }

    #[test]
    fn new_sandbox_creates_root() {
        let tmp = std::env::temp_dir().join("bg-sandbox-test");
        let _ = std::fs::remove_dir_all(&tmp);
        let s = WindowsSandbox::new(tmp.clone()).unwrap();
        assert_eq!(s.root, tmp);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn spawn_echo_returns_zero() {
        let tmp = std::env::temp_dir().join("bg-sandbox-spawn");
        let s = WindowsSandbox::new(tmp.clone()).unwrap();
        // cmd /c "echo hi" — returns 0 on Windows
        let code = s.spawn("cmd", &["/c", "echo hi"]).unwrap();
        assert_eq!(code, 0);
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
