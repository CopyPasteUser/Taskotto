//! PID-reuse-aware signals. pidfd pins the exact process, and the start tick is
//! checked again before signalling. No sudo, polkit escalation or root daemon.
use crate::monitor::{Process, start_ticks};
use std::io;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignalAction {
    Terminate,
    ForceKill,
    Stop,
    Continue,
}

impl SignalAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Terminate => "Terminate (SIGTERM)",
            Self::ForceKill => "Force kill (SIGKILL)",
            Self::Stop => "Pause (SIGSTOP)",
            Self::Continue => "Resume (SIGCONT)",
        }
    }

    fn signal(self) -> libc::c_int {
        match self {
            Self::Terminate => libc::SIGTERM,
            Self::ForceKill => libc::SIGKILL,
            Self::Stop => libc::SIGSTOP,
            Self::Continue => libc::SIGCONT,
        }
    }
}

pub fn send_signal(process: &Process, action: SignalAction) -> io::Result<()> {
    if process.pid <= 1 || process.pid == std::process::id() as i32 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Taskotto refuses to signal PID 1 or itself",
        ));
    }
    // Linux 5.3+; a pidfd avoids signalling a recycled PID between checks.
    let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, process.pid, 0u32) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let result = (|| {
        if start_ticks(process.pid)? != process.start_ticks {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Process exited or its PID was reused; refresh the list",
            ));
        }
        let result = unsafe {
            libc::syscall(
                libc::SYS_pidfd_send_signal,
                fd,
                action.signal(),
                std::ptr::null::<libc::siginfo_t>(),
                0u32,
            )
        };
        if result == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    })();
    unsafe { libc::close(fd as libc::c_int) };
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protects_init_and_own_process() {
        let mut process = Process {
            pid: 1,
            ..Process::default()
        };
        assert_eq!(
            send_signal(&process, SignalAction::Terminate)
                .unwrap_err()
                .kind(),
            io::ErrorKind::PermissionDenied
        );
        process.pid = std::process::id() as i32;
        assert_eq!(
            send_signal(&process, SignalAction::ForceKill)
                .unwrap_err()
                .kind(),
            io::ErrorKind::PermissionDenied
        );
    }
}
