use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;

pub fn drive() -> Result<()> {
    let data_dir = match directories::ProjectDirs::from("github", "AshyIsMe", "jr") {
        Some(dirs) => dirs.data_dir().to_path_buf(),
        None => PathBuf::new(),
    };
    fs::create_dir_all(&data_dir)?;
    let hist_file = data_dir.join("jhistory");

    let mut rl = rustyline::Editor::<()>::new()?;
    if hist_file.exists() {
        rl.load_history(&hist_file)?;
    }
    rl.set_auto_add_history(true);

    let mut names = HashMap::new();

    loop {
        let line = match rl.readline("   ") {
            Ok(line) => line,
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => break,
            Err(other) => Err(other)?,
        };
        if super::eval(&line, &mut names)? {
            break;
        }
    }

    rl.save_history(&hist_file)?;
    Ok(())
}
