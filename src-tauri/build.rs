use std::{
    env,
    ffi::OsString,
    fs,
    io,
    path::{Path, PathBuf},
    process::Command,
};

const OFFLINE_RUNTIME_DIR: &str = "offline-runtime";

fn main() {
    if let Err(error) = prepare_offline_runtime() {
        println!("cargo:warning=failed to prepare offline runtime: {error}");
    }

    tauri_build::build()
}

fn prepare_offline_runtime() -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    let runtime_dir = manifest_dir.join(OFFLINE_RUNTIME_DIR);

    if runtime_dir.exists() {
        fs::remove_dir_all(&runtime_dir)?;
    }
    fs::create_dir_all(&runtime_dir)?;
    fs::write(runtime_dir.join(".keep"), b"offline-runtime placeholder")?;

    let node_path = locate_node_binary()?;
    let npm_root = locate_npm_root()?;
    let codex_package = npm_root.join("@openai").join("codex");

    if !node_path.exists() || !codex_package.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "node or @openai/codex not found (node: {}, codex: {})",
                node_path.display(),
                codex_package.display()
            ),
        ));
    }

    let node_target_name = node_path
        .file_name()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("node.exe"));
    fs::copy(&node_path, runtime_dir.join(node_target_name))?;

    let target_package_dir = runtime_dir.join("node_modules").join("@openai").join("codex");
    copy_dir_all(&codex_package, &target_package_dir)?;

    println!("cargo:warning=offline runtime prepared at {}", runtime_dir.display());
    Ok(())
}

fn locate_node_binary() -> io::Result<PathBuf> {
    if cfg!(target_os = "windows") {
        if let Ok(program_files) = env::var("ProgramFiles") {
            let default = PathBuf::from(program_files).join("nodejs").join("node.exe");
            if default.exists() {
                return Ok(default);
            }
        }
        let output = Command::new("where.exe").arg("node").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "node executable not found"))
    } else {
        let output = Command::new("which").arg("node").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "node executable not found"))
    }
}

fn locate_npm_root() -> io::Result<PathBuf> {
    if cfg!(target_os = "windows") {
        if let Ok(app_data) = env::var("APPDATA") {
            let default = PathBuf::from(app_data).join("npm").join("node_modules");
            if default.exists() {
                return Ok(default);
            }
        }
        let output = Command::new("npm.cmd").arg("root").arg("-g").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "npm global root not found"))
    } else {
        let output = Command::new("npm").arg("root").arg("-g").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "npm global root not found"))
    }
}

fn copy_dir_all(source: &Path, target: &Path) -> io::Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_path = entry.path();
        let destination = target.join(OsString::from(entry.file_name()));
        if entry_path.is_dir() {
            copy_dir_all(&entry_path, &destination)?;
        } else {
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&entry_path, &destination)?;
        }
    }
    Ok(())
}
