//! Linux process-control smoke test using only a disposable child owned by CI.
use std::{
    fs,
    os::unix::process::ExitStatusExt,
    process::{Child, Command},
    thread,
    time::{Duration, Instant},
};
use taskotto::{
    actions::{SignalAction, send_signal},
    monitor::{Process, start_ticks},
};

struct DisposableChild(Child);

impl DisposableChild {
    fn spawn() -> Self {
        Self(Command::new("sleep").arg("60").spawn().unwrap())
    }

    fn process(&self) -> Process {
        let pid = self.0.id() as i32;
        let mut process = Process::default();
        process.pid = pid;
        process.start_ticks = start_ticks(pid).unwrap();
        process
    }
}

impl Drop for DisposableChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn is_stopped(pid: i32) -> Option<bool> {
    let status = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    let state = status
        .lines()
        .find_map(|line| line.strip_prefix("State:"))?;
    Some(state.trim().starts_with('T'))
}

fn await_stopped(pid: i32, expected: bool) {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if is_stopped(pid) == Some(expected) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "PID {pid} did not reach stopped={expected}"
        );
        thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn pidfd_actions_target_only_the_original_disposable_process() {
    let mut child = DisposableChild::spawn();
    let process = child.process();

    let mut stale = process.clone();
    stale.start_ticks += 1;
    let error = send_signal(&stale, SignalAction::ForceKill).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    assert!(child.0.try_wait().unwrap().is_none());

    send_signal(&process, SignalAction::Stop).unwrap();
    await_stopped(process.pid, true);

    send_signal(&process, SignalAction::Continue).unwrap();
    await_stopped(process.pid, false);

    send_signal(&process, SignalAction::Terminate).unwrap();
    let exit = child.0.wait().unwrap();
    assert_eq!(exit.signal(), Some(libc::SIGTERM));

    let mut force_kill_child = DisposableChild::spawn();
    let force_kill_process = force_kill_child.process();
    send_signal(&force_kill_process, SignalAction::ForceKill).unwrap();
    let exit = force_kill_child.0.wait().unwrap();
    assert_eq!(exit.signal(), Some(libc::SIGKILL));
}
