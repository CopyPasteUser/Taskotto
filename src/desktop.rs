//! Opt-in, user-local XDG desktop entry installation. Never writes system paths.
use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const DESKTOP: &str = include_str!("../packaging/io.github.copypasteuser.taskotto.desktop");
const ICON: &str = include_str!("../assets/io.github.copypasteuser.taskotto.svg");
const ICON_PNG: &[u8] = include_bytes!("../assets/io.github.copypasteuser.taskotto.png");
const APP_ID: &str = "io.github.copypasteuser.taskotto";

pub fn install_for_current_user() -> io::Result<PathBuf> {
    install(&data_home()?, &env::current_exe()?)
}

pub fn uninstall_for_current_user() -> io::Result<(bool, bool)> {
    uninstall(&data_home()?)
}

fn data_home() -> io::Result<PathBuf> {
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
    Ok(data)
}

/// Remove only entries Taskotto recognizes as its own, never modified or
/// symlinked paths. False means that entry was already absent.
pub fn uninstall(data_home: &Path) -> io::Result<(bool, bool)> {
    if !data_home.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "absolute XDG data path required",
        ));
    }
    let desktop = data_home.join(format!("applications/{APP_ID}.desktop"));
    let icon = data_home.join(format!("icons/hicolor/scalable/apps/{APP_ID}.svg"));
    let png = data_home.join(format!("icons/hicolor/256x256/apps/{APP_ID}.png"));
    let old_desktop = read_regular_entry(&desktop)?;
    let old_icon = read_regular_entry(&icon)?;
    let old_png = read_regular_entry(&png)?;
    // Check all three before removing anything, preserving customized files.
    if old_desktop.as_deref().is_some_and(|bytes| {
        std::str::from_utf8(bytes)
            .map(|text| !matches_desktop_template(text, &png))
            .unwrap_or(true)
    }) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "Refusing to delete customized desktop entry: {}",
                desktop.display()
            ),
        ));
    }
    if old_icon
        .as_deref()
        .is_some_and(|bytes| bytes != ICON.as_bytes())
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("Refusing to delete customized icon: {}", icon.display()),
        ));
    }
    if old_png.as_deref().is_some_and(|bytes| bytes != ICON_PNG) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("Refusing to delete customized icon: {}", png.display()),
        ));
    }
    if old_desktop.is_some() {
        fs::remove_file(desktop)?;
    }
    if old_icon.is_some() {
        fs::remove_file(icon)?;
    }
    if old_png.is_some() {
        fs::remove_file(png)?;
    }
    Ok((
        old_desktop.is_some(),
        old_icon.is_some() || old_png.is_some(),
    ))
}

fn read_regular_entry(path: &Path) -> io::Result<Option<Vec<u8>>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if !metadata.file_type().is_file() || metadata.len() > 65_536 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "Refusing to delete non-regular or oversized entry: {}",
                path.display()
            ),
        ));
    }
    fs::read(path).map(Some)
}

fn matches_desktop_template(installed: &str, png: &Path) -> bool {
    let expected = DESKTOP.lines().collect::<Vec<_>>();
    let actual = installed.lines().collect::<Vec<_>>();
    actual.len() == expected.len()
        && actual.iter().zip(expected).all(|(observed, template)| {
            if template == "Exec=taskotto" {
                observed.starts_with("Exec=") && observed.len() > "Exec=".len()
            } else if template == "Icon=io.github.copypasteuser.taskotto" {
                observed == &template || *observed == format!("Icon={}", png.display())
            } else {
                observed == &template
            }
        })
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
    let png_path = data_home.join(format!("icons/hicolor/256x256/apps/{APP_ID}.png"));
    let png = png_path
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "icon path is not UTF-8"))?;
    fs::create_dir_all(desktop_path.parent().unwrap())?;
    fs::create_dir_all(icon_path.parent().unwrap())?;
    fs::create_dir_all(png_path.parent().unwrap())?;
    // Desktop entry Exec parsing is not a shell. Quotes are needed for spaces.
    let exec = if binary.contains(' ') {
        format!("\"{binary}\"")
    } else {
        binary.to_owned()
    };
    let desktop_contents = DESKTOP
        .replace("Exec=taskotto", &format!("Exec={exec}"))
        .replace(&format!("Icon={APP_ID}"), &format!("Icon={png}"));
    // Atomic replacement keeps a previously working launcher intact if disk
    // writes fail. Renaming over a leaf symlink also avoids writing through it.
    write_atomic(&icon_path, ICON.as_bytes())?;
    write_atomic(&png_path, ICON_PNG)?;
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
        assert!(ICON_PNG.starts_with(b"\x89PNG\r\n\x1a\n"));
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
        assert!(text.contains(&format!(
            "Icon={}",
            folder
                .join("icons/hicolor/256x256/apps/io.github.copypasteuser.taskotto.png")
                .display()
        )));
        assert_eq!(
            fs::read(
                folder.join("icons/hicolor/256x256/apps/io.github.copypasteuser.taskotto.png")
            )
            .unwrap(),
            ICON_PNG
        );
        assert!(
            folder
                .join("icons/hicolor/scalable/apps/io.github.copypasteuser.taskotto.svg")
                .exists()
        );
        fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn uninstall_removes_only_owned_entries_and_is_idempotent() {
        let folder = std::env::temp_dir().join(format!(
            "taskotto-uninstall-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        install(&folder, Path::new("/opt/Taskotto/taskotto")).unwrap();
        let unrelated = folder.join("applications/another-app.desktop");
        fs::write(&unrelated, "Not Taskotto").unwrap();
        assert_eq!(uninstall(&folder).unwrap(), (true, true));
        assert_eq!(uninstall(&folder).unwrap(), (false, false));
        assert_eq!(fs::read_to_string(unrelated).unwrap(), "Not Taskotto");
        fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn uninstall_preserves_customized_entries_and_symlinks() {
        use std::os::unix::fs::symlink;
        let folder = std::env::temp_dir().join(format!(
            "taskotto-uninstall-safety-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let desktop = install(&folder, Path::new("/usr/local/bin/taskotto")).unwrap();
        let icon = folder.join(format!("icons/hicolor/scalable/apps/{APP_ID}.svg"));
        fs::write(&icon, "custom icon").unwrap();
        assert_eq!(
            uninstall(&folder).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
        assert!(desktop.exists());
        fs::write(&icon, ICON).unwrap();
        let png = folder.join(format!("icons/hicolor/256x256/apps/{APP_ID}.png"));
        fs::write(&png, "custom png").unwrap();
        assert_eq!(
            uninstall(&folder).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
        fs::write(&png, ICON_PNG).unwrap();
        fs::write(&desktop, "handwritten desktop entry").unwrap();
        assert_eq!(
            uninstall(&folder).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
        assert_eq!(fs::read_to_string(&icon).unwrap(), ICON);
        fs::remove_file(&desktop).unwrap();
        let unrelated = folder.join("unrelated.txt");
        fs::write(&unrelated, "preserve me").unwrap();
        symlink(&unrelated, &desktop).unwrap();
        assert_eq!(
            uninstall(&folder).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
        assert_eq!(fs::read_to_string(unrelated).unwrap(), "preserve me");
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
