use regex::Regex;
use std::collections::HashSet;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let zsh_history_path = get_zsh_history_path();
    let unique_commands = unique_commands(&zsh_history_path).unwrap_or_else(|err| {
        eprintln!("读取 ~/.zsh_history 失败: {err}");
        process::exit(1)
    });
    if let Err(err) = rewrite(zsh_history_path, unique_commands) {
        eprintln!("重写 ~/.zsh_history 失败: {err}");
        process::exit(1)
    };
}

fn get_zsh_history_path() -> PathBuf {
    let home_path = &match dirs::home_dir() {
        Some(home_path) => home_path,
        None => {
            eprintln!("获取 $HOME 路径失败.");
            process::exit(1)
        }
    };
    Path::new(home_path).join(".zsh_history")
}

fn unique_commands(zsh_history_path: &PathBuf) -> Result<HashSet<String>, Box<dyn Error>> {
    let mut tmp_command = String::new();
    let mut unique_commands = HashSet::new();
    let is_new_command = Regex::new(r"^:\s\d{10,13}:\d;").unwrap();

    let file = File::open(zsh_history_path)?;
    for line in BufReader::new(file).lines() {
        let line = line?;

        // 兼容多行命令, 例如:
        // docker run \
        // -d \
        // hello-world
        if is_new_command.is_match(&line) {
            if !tmp_command.is_empty() {
                let command = parse_command(&tmp_command);
                unique_commands.insert(command);
                tmp_command.clear();
            }
        } else {
            tmp_command.push_str("\n");
        }
        tmp_command.push_str(&line);
    }
    // 检查剩余的 tmp_command
    if !tmp_command.is_empty() {
        let command = parse_command(&tmp_command);
        unique_commands.insert(command);
    }

    Ok(unique_commands)
}

fn parse_command(tmp_command: &str) -> String {
    let command = &tmp_command.split(":0;").collect::<Vec<&str>>()[1];
    command.to_string()
}

fn rewrite(
    zsh_history_path: PathBuf,
    unique_commands: HashSet<String>,
) -> Result<(), Box<dyn Error>> {
    let current_timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&zsh_history_path)?;
    for (i, command) in unique_commands.iter().enumerate() {
        writeln!(file, ": {}:0;{}", current_timestamp + (i as u64), command)?;
    }
    Ok(())
}
