use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let zsh_history_path: PathBuf = Path::new(&dirs::home_dir().unwrap()).join(".zsh_history");
    let unique_commands = unique_commands(&zsh_history_path);
    rewrite(zsh_history_path, unique_commands);
}

fn unique_commands(zsh_history_path: &PathBuf) -> Vec<String> {
    let mut timestamp_and_command = String::new();
    let mut unique_commands = Vec::new();

    for line in BufReader::new(File::open(zsh_history_path).expect("读取文件失败")).lines() {
        let line = line.unwrap();
        timestamp_and_command.push_str(&line);

        if !line.ends_with('\\') {
            let command = parse_command(&timestamp_and_command);
            if !unique_commands.contains(&command) {
                unique_commands.push(command);
            }
            timestamp_and_command.clear();
        } else {
            timestamp_and_command.push_str("\n");
        }
    }
    unique_commands
}

fn parse_command(timestamp_and_command: &String) -> String {
    let command = &timestamp_and_command.split(":0;").collect::<Vec<&str>>()[1];
    command.to_string()
}

fn rewrite(zsh_history_path: PathBuf, unique_commands: Vec<String>) {
    let current_timestamp = current_timestamp();
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&zsh_history_path)
        .expect("写入文件失败");
    for (i, command) in unique_commands.iter().enumerate() {
        writeln!(file, ": {}:0;{}", current_timestamp + (i as u64), command).expect("写入行失败");
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
