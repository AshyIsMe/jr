use std::io::Read;
use std::{fs, io};

use anyhow::{anyhow, Context, Result};

use jr::test_impls::{read_ijs_lines, RunList};

fn main() -> Result<()> {
    let file = std::env::args_os()
        .nth(1)
        .expect("usage: file-to-create-or-update");
    let mut run_list = match fs::read_to_string(&file) {
        Ok(content) => RunList::open(content)?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => RunList::empty(),
        Err(e) => return Err(e).with_context(|| anyhow!("unexpected error reading {file:?}")),
    };

    let mut from = io::stdin().lock();
    let mut input = String::new();
    from.read_to_string(&mut input)?;
    for line in read_ijs_lines(&input) {
        run_list.add(&line).context(line)?;
    }
    fs::write(file, run_list.save()?)?;
    Ok(())
}
