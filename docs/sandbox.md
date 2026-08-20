# Sandbox (Windows Job Object)

> v0.1.5 — `crates/bg-sandbox/src/windows_job.rs` adds a typed Windows
> Job Object skeleton.

## Why a Job Object

A coding agent that can spawn shells must not be able to:

1. read `~/.ssh/id_rsa`
2. write to `C:\Windows\System32\`
3. fork-bomb the host
4. survive the agent dying

A **Job Object** gives you (3) and (4) for free via two limit flags:

| Flag | Effect |
|------|--------|
| `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` | When the job handle is closed (incl. agent crash), every process in the job is killed. |
| `JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION` | An unhandled exception in any job process kills the whole job. |

And (3) again via:

| Flag | Effect |
|------|--------|
| `JOB_OBJECT_LIMIT_PROCESS_MEMORY` | Per-process RSS cap (`process_memory_limit`). |
| `JOB_OBJECT_LIMIT_JOB_MEMORY` | Whole-tree RSS cap (`job_memory_limit`). |
| `active_process_limit` | Hard cap on concurrent processes. |

## The 5-step Win32 dance

1. `CreateJobObjectW(NULL, NULL)` → job handle
2. Fill `JOBOBJECT_EXTENDED_LIMIT_INFORMATION` (see struct in `windows_job.rs`)
3. `SetInformationJobObject(job, JobObjectExtendedLimitInformation, &info, sizeof(info))`
4. `AssignProcessToJobObject(job, GetCurrentProcess())` — agent assigns itself, OR spawn children with `CREATE_SUSPENDED` + assign + `ResumeThread`
5. On drop: `CloseHandle(job)` → all processes die

## Filesystem isolation

Job Objects can't restrict filesystem access by themselves. The standard
approach:

1. Create a minimal root: `C:\bg-sandbox\<id>\`
2. `ICACLS` or `SetNamedSecurityInfo` to grant the sandboxed user RWX on
   the root only, dropping other ACLs
3. `SetFileAttributes(FILE_ATTRIBUTE_READONLY)` on files you don't want
   modified

For registry isolation, use `RegCreateKeyExW` with
`REG_OPTION_CREATE_VOLATILE` inside the sandbox account.

## The `WindowsSandbox` wrapper

```rust
use bg_sandbox::windows_job::WindowsSandbox;
use std::path::PathBuf;

let s = WindowsSandbox::new(PathBuf::from(r"C:\bg-sandbox\1"))?;
let code = s.spawn("cmd", &["/c", "echo hi"])?;
assert_eq!(code, 0);
```

`WindowsSandbox::new` creates the Job Object with safe defaults (512 MB
per process, 2 GB whole job, 64 active processes), assigns the calling
process, and creates the root directory.

## What's real vs stubbed

| Piece | Status |
|-------|--------|
| `JobBasicLimitInfo` / `JobExtendedLimitInfo` / `IoCounters` structs | ✅ Real, `repr(C)` matches Win32 |
| Limit flag constants | ✅ Real values from winnt.h |
| `CreateJobObjectW` / `SetInformationJobObject` / `AssignProcessToJobObject` calls | ⚠️ Stubbed — see "To make it real" in the source |
| `OwnedHandle` wrapping | ⚠️ Skeleton uses invalid handle |
| Default limits (512 MB / 2 GB / 64 procs / kill-on-close) | ✅ Real values, applied to struct |
| `WindowsSandbox::spawn` | ✅ Real — `std::process::Command` in `root` |
| `WindowsSandbox::new` creates root dir | ✅ Real |

## Tests

- `default_limits_have_kill_on_close` — verifies the flag is set
- `new_sandbox_creates_root` — verifies directory creation
- `spawn_echo_returns_zero` — end-to-end `cmd /c "echo hi"` returns 0

## Future work

- Real `windows-sys` FFI calls (the skeleton documents exactly which
  features to enable in `Cargo.toml`)
- ACL hardening (drop write to `C:\Windows`, drop read to `~/.ssh`)
- Registry isolation via volatile keys
- Cross-platform: macOS `sandbox-exec` profile, Linux `bubblewrap`/`seccomp`
