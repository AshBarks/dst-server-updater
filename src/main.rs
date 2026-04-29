use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use chrono::Local;
use dst_update::{fetch_latest_update, AppError, UpdateStatus};
use log::{error, info, warn, LevelFilter};

struct Config {
    server_root: PathBuf,
    steamcmd_dir: PathBuf,
    log_dir: Option<PathBuf>,
}

fn check_dir_readable(path: &PathBuf, var_name: &str) -> Result<(), String> {
    if !path.is_dir() {
        return Err(format!("{} 指向的目录不存在: {}", var_name, path.display()));
    }
    if fs::read_dir(path).is_err() {
        return Err(format!(
            "{} 指向的目录无读取权限: {}",
            var_name,
            path.display()
        ));
    }
    Ok(())
}

fn check_dir_writable(path: &PathBuf, var_name: &str) -> Result<(), String> {
    if !path.is_dir() {
        return Err(format!("{} 指向的目录不存在: {}", var_name, path.display()));
    }
    let test_file = path.join(".dst_updater_write_test");
    if File::create(&test_file).is_err() {
        return Err(format!(
            "{} 指向的目录无写入权限: {}",
            var_name,
            path.display()
        ));
    }
    let _ = fs::remove_file(&test_file);
    Ok(())
}

impl Config {
    fn from_env() -> Result<Self, String> {
        let server_root = env::var("DST_SERVER__ROOT").map_err(|_| {
            "环境变量 DST_SERVER__ROOT 未设置".to_string()
        })?;
        let server_root = PathBuf::from(&server_root);
        check_dir_readable(&server_root, "DST_SERVER__ROOT")?;

        let steamcmd_dir = env::var("STEAMCMD__DIR").map_err(|_| {
            "环境变量 STEAMCMD__DIR 未设置".to_string()
        })?;
        let steamcmd_dir = PathBuf::from(&steamcmd_dir);
        check_dir_readable(&steamcmd_dir, "STEAMCMD__DIR")?;

        let log_dir = env::var("DST_UPDATER__LOG__DIR").ok().map(PathBuf::from);
        if let Some(ref dir) = log_dir {
            check_dir_writable(dir, "DST_UPDATER__LOG__DIR")?;
        }

        Ok(Config {
            server_root,
            steamcmd_dir,
            log_dir,
        })
    }
}

struct MultiLogger {
    file: Option<File>,
}

impl log::Log for MultiLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let msg = format!("[{} {}] {}", timestamp, record.level(), record.args());
        println!("{}", msg);
        if let Some(ref mut file) = self.file.as_ref() {
            let _ = writeln!(file, "{}", msg);
        }
    }

    fn flush(&self) {
        if let Some(ref file) = self.file {
            let _ = file.sync_all();
        }
    }
}

fn init_logger(log_dir: Option<&PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let file = match log_dir {
        Some(dir) => {
            let date = Local::now().format("%Y-%m-%d");
            let path = dir.join(format!("dst-updater-{}.log", date));
            Some(File::create(path)?)
        }
        None => None,
    };

    let logger = Box::new(MultiLogger { file });
    log::set_boxed_logger(logger)?;
    log::set_max_level(LevelFilter::Info);

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("配置错误: {}", e);
            return Err(e.into());
        }
    };

    init_logger(config.log_dir.as_ref())?;

    let latest = match fetch_latest_update(3, Some(UpdateStatus::Release)) {
        Ok(Some(u)) => u,
        Ok(None) => {
            error!("未找到最新版本信息");
            return Err(AppError::NoUpdateFound.into());
        }
        Err(e) => {
            error!("获取最新版本失败: {}", e);
            return Err(e.into());
        }
    };
    let latest_build = latest.build_number;
    info!("最新版本: build {}", latest_build);

    let version_path = config.server_root.join("version.txt");

    let current_build: u32 = if version_path.exists() {
        let content = fs::read_to_string(&version_path)?;
        content.trim().parse()?
    } else {
        info!("version.txt 不存在，默认当前版本为 0");
        0
    };
    info!("当前版本: build {}", current_build);

    if current_build >= latest_build {
        info!("服务器已是最新版本，无需更新");
        return Ok(());
    }

    info!("发现新版本，开始更新...");

    let steamcmd_path = config.steamcmd_dir.join("steamcmd.sh");
    let output = Command::new(&steamcmd_path)
        .arg("+force_install_dir")
        .arg(&config.server_root)
        .arg("+login")
        .arg("anonymous")
        .arg("+app_update")
        .arg("343050")
        .arg("validate")
        .arg("+quit")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("Success") {
        info!("更新成功! build {} -> {}", current_build, latest_build);
    } else {
        error!("更新可能失败，输出中未包含 Success");
        warn!("{}", stdout);
    }

    Ok(())
}
