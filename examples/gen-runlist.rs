use std::io::BufRead;
use std::{fs, io};

use anyhow::{anyhow, Context, Result};

use jr::test_impls::RunList;

fn main() -> Result<()> {
    let file = std::env::args_os()
        .nth(1)
        .expect("usage: file-to-create-or-update");
    let mut run_list = match fs::read_to_string(&file) {
        Ok(content) => RunList::open(content)?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => RunList::empty(),
        Err(e) => return Err(e).with_context(|| anyhow!("unexpected error reading {file:?}")),
    };
    let from = io::stdin().lock();
    for (i, line) in from.lines().enumerate() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line.starts_with("NB. ") {
            continue;
        }
        run_list.add(line).with_context(|| anyhow!("input {i}"))?;
    }
    fs::write(file, run_list.save()?)?;
    Ok(())
}
