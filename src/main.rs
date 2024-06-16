use once_cell::sync::Lazy;
use regex::Regex;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

static IS_NEW_COMMAND_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^:\s\d{10}:\d;").unwrap());

fn main() {
    let zsh_history_path = get_zsh_history_path();
    let unique_commands = unique_commands(&zsh_history_path).unwrap_or_else(|err| {
        eprintln!("读取 ~/.zsh_history 失败: {err}");
        process::exit(1)
    });
    if let Err(err) = rewrite(&zsh_history_path, unique_commands) {
        eprintln!("重写 ~/.zsh_history 失败: {err}");
        process::exit(1)
    };
}

fn get_zsh_history_path() -> PathBuf {
    dirs::home_dir()
        .map(|home| Path::new(&home).join(".zsh_history"))
        .unwrap_or_else(|| {
            eprintln!("获取 $HOME 路径失败.");
            process::exit(1)
        })
}

fn unique_commands(zsh_history_path: &PathBuf) -> Result<Vec<String>, Box<dyn Error>> {
    let mut tmp_command = String::new();
    let mut unique_commands = Vec::new();

    let file = File::open(zsh_history_path)?;
    for line in BufReader::new(file).lines() {
        let line = line?;

        // 兼容多行命令, 例如:
        // docker run \
        //   -d \
        //   hello-world
        if IS_NEW_COMMAND_RE.is_match(&line) {
            if !tmp_command.is_empty() {
                insert_if_unique(&tmp_command, &mut unique_commands);
                tmp_command.clear();
            }
        } else {
            tmp_command.push('\n');
        }
        tmp_command.push_str(&line);
    }
    // 检查剩余的 tmp_command
    if !tmp_command.is_empty() {
        insert_if_unique(&tmp_command, &mut unique_commands);
    }

    Ok(unique_commands)
}

fn insert_if_unique(tmp_command: &str, unique_commands: &mut Vec<String>) {
    let command = parse_command(tmp_command);
    if unique_commands.contains(&command) {
        return;
    }
    unique_commands.push(command);
}

/// 解析命令: 从 `: 1718500660:0;cargo build -r` 中解析出 `cargo build -r`
fn parse_command(tmp_command: &str) -> String {
    (&tmp_command[15..]).to_string()
}

fn rewrite(zsh_history_path: &PathBuf, unique_commands: Vec<String>) -> Result<(), Box<dyn Error>> {
    let current_timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(zsh_history_path)?;
    for (i, command) in unique_commands.iter().enumerate() {
        writeln!(file, ": {}:0;{}", current_timestamp + (i as u64), command)?;
    }
    Ok(())
}
