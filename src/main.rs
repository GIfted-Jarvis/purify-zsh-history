use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

const ZSH_HISTORY: &str = ".zsh_history";

fn main() {
    let zsh_history_path = zsh_history_path();
    let unique_commands = unique_commands(&zsh_history_path).unwrap_or_else(|err| {
        eprintln!("读取 ~/.zsh_history 失败: {err}");
        process::exit(1)
    });
    if let Err(err) = rewrite(zsh_history_path, unique_commands) {
        eprintln!("重写 ~/.zsh_history 失败: {err}");
        process::exit(1)
    };
}

fn zsh_history_path() -> PathBuf {
    let home_path = &match dirs::home_dir() {
        Some(home_path) => home_path,
        None => {
            eprintln!("获取 $HOME 路径失败.");
            process::exit(1)
        }
    };
    Path::new(home_path).join(ZSH_HISTORY)
}

fn unique_commands(zsh_history_path: &PathBuf) -> Result<Vec<String>, Box<dyn Error>> {
    let mut timestamp_and_command = String::new();
    let mut unique_commands = Vec::new();

    let file = File::open(zsh_history_path)?;
    for line in BufReader::new(file).lines() {
        let line = line?;
        timestamp_and_command.push_str(&line);

        if line.ends_with('\\') {
            timestamp_and_command.push_str("\n");
        } else {
            let command = parse_command(&timestamp_and_command);
            if !unique_commands.contains(&command) {
                unique_commands.push(command);
            }
            timestamp_and_command.clear();
        }
    }
    Ok(unique_commands)
}

fn parse_command(timestamp_and_command: &str) -> String {
    let command = &timestamp_and_command.split(":0;").collect::<Vec<&str>>()[1];
    command.to_string()
}

fn rewrite(zsh_history_path: PathBuf, unique_commands: Vec<String>) -> Result<(), Box<dyn Error>> {
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
