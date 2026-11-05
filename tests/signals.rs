use std::{io, process::Command, thread, time::Duration};
use taskotto::{
    actions::{SignalAction, send_signal},
    monitor::Monitor,
};

#[test]
fn only_the_selected_process_instance_can_be_signalled() -> io::Result<()> {
    let mut child = Command::new("sleep").arg("30").spawn()?;
    let pid = child.id() as i32;
    let result = (|| {
        let mut process = Monitor::default()
            .sample()?
            .processes
            .into_iter()
            .find(|process| process.pid == pid)
            .expect("child must appear in /proc");
        let ticks = process.start_ticks;
        process.start_ticks = ticks.saturating_add(1);
        let error = send_signal(&process, SignalAction::Terminate).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        assert!(
            child.try_wait()?.is_none(),
            "stale selection must not terminate the child"
        );

        process.start_ticks = ticks;
        send_signal(&process, SignalAction::Terminate)?;
        let status = child.wait()?;
        assert!(!status.success(), "terminated child should report a signal");
        Ok(())
    })();
    // Also clean up if assertions/early errors interrupted the test.
    let _ = child.kill();
    let _ = child.wait();
    result
}

#[test]
fn pause_and_resume_disposable_child() -> io::Result<()> {
    let mut child = Command::new("sleep").arg("30").spawn()?;
    let pid = child.id() as i32;
    let result = (|| {
        let process = Monitor::default()
            .sample()?
            .processes
            .into_iter()
            .find(|process| process.pid == pid)
            .expect("child must appear in /proc");
        send_signal(&process, SignalAction::Stop)?;
        thread::sleep(Duration::from_millis(50));
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat"))?;
        let state = stat
            .split(')')
            .next_back()
            .unwrap_or("")
            .split_whitespace()
            .next();
        assert_eq!(state, Some("T"));
        send_signal(&process, SignalAction::Continue)?;
        send_signal(&process, SignalAction::Terminate)?;
        Ok(())
    })();
    let _ = child.kill();
    let _ = child.wait();
    result
}
