//! Opt-in, user-local XDG desktop entry installation. Never writes system paths.
use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const DESKTOP: &str = include_str!("../packaging/io.github.copypasteuser.taskotto.desktop");
const ICON: &str = include_str!("../assets/io.github.copypasteuser.taskotto.svg");
const APP_ID: &str = "io.github.copypasteuser.taskotto";

pub fn install_for_current_user() -> io::Result<PathBuf> {
    let data = if let Some(home) = env::var_os("XDG_DATA_HOME").filter(|value| !value.is_empty()) {
        let path = PathBuf::from(home);
        if !path.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "XDG_DATA_HOME must be absolute",
            ));
        }
        path
    } else {
        PathBuf::from(
            env::var_os("HOME")
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME not set"))?,
        )
        .join(".local/share")
    };
    install(&data, &env::current_exe()?)
}

pub fn install(data_home: &Path, executable: &Path) -> io::Result<PathBuf> {
    if !data_home.is_absolute() || !executable.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "absolute paths required",
        ));
    }
    let binary = executable
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "binary path is not UTF-8"))?;
    if binary
        .chars()
        .any(|c| matches!(c, '\n' | '\r' | '"' | '\\' | '%'))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "binary path contains unsupported desktop entry characters",
        ));
    }
    let desktop_path = data_home.join(format!("applications/{APP_ID}.desktop"));
    let icon_path = data_home.join(format!("icons/hicolor/scalable/apps/{APP_ID}.svg"));
    fs::create_dir_all(desktop_path.parent().unwrap())?;
    fs::create_dir_all(icon_path.parent().unwrap())?;
    // Desktop entry Exec parsing is not a shell. Quotes are needed for spaces.
    let exec = if binary.contains(' ') {
        format!("\"{binary}\"")
    } else {
        binary.to_owned()
    };
    let desktop_contents = DESKTOP.replace("Exec=taskotto", &format!("Exec={exec}"));
    // Atomic replacement keeps a previously working launcher intact if disk
    // writes fail. Renaming over a leaf symlink also avoids writing through it.
    write_atomic(&icon_path, ICON.as_bytes())?;
    write_atomic(&desktop_path, desktop_contents.as_bytes())?;
    Ok(desktop_path)
}

fn write_atomic(path: &Path, contents: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing parent directory"))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = parent.join(format!(".taskotto-{}-{nonce}.tmp", std::process::id()));
    let result = (|| {
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o644);
        }
        let mut file = options.open(&temporary)?;
        file.write_all(contents)?;
        file.sync_all()?;
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_template_has_expected_metadata() {
        assert!(DESKTOP.contains("Categories=System;Monitor;"));
        assert!(DESKTOP.contains("Icon=io.github.copypasteuser.taskotto"));
        assert!(ICON.contains("<svg"));
    }

    #[test]
    fn installs_user_local_entry_with_absolute_binary() {
        let folder = std::env::temp_dir().join(format!(
            "taskotto-install-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let desktop = install(&folder, Path::new("/opt/Test Taskotto/taskotto")).unwrap();
        let text = fs::read_to_string(desktop).unwrap();
        assert!(text.contains("Exec=\"/opt/Test Taskotto/taskotto\""));
        assert!(
            folder
                .join("icons/hicolor/scalable/apps/io.github.copypasteuser.taskotto.svg")
                .exists()
        );
        fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn installing_over_a_symlink_does_not_overwrite_its_target() {
        use std::os::unix::fs::symlink;

        let folder = std::env::temp_dir().join(format!(
            "taskotto-symlink-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let desktop_path = folder.join(format!("applications/{APP_ID}.desktop"));
        fs::create_dir_all(desktop_path.parent().unwrap()).unwrap();
        let unrelated = folder.join("unrelated.txt");
        fs::write(&unrelated, "must not change").unwrap();
        symlink(&unrelated, &desktop_path).unwrap();

        install(&folder, Path::new("/usr/local/bin/taskotto")).unwrap();
        assert_eq!(fs::read_to_string(unrelated).unwrap(), "must not change");
        assert!(
            !desktop_path
                .symlink_metadata()
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert!(
            fs::read_to_string(&desktop_path)
                .unwrap()
                .contains("Exec=/usr/local/bin/taskotto")
        );
        fs::remove_dir_all(folder).unwrap();
    }
}
