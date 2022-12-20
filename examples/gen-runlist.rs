use std::{fs, io};

use anyhow::{anyhow, bail, Context, Result};
use itertools::Itertools;

use jr::test_impls::{read_ijs_dir, read_ijs_lines, RunList};

fn main() -> Result<()> {
    let files = std::env::args_os().skip(1).collect_vec();
    if files.len() != 2 {
        bail!("usage: ijs-or-directory-to-read toml-to-create-or-update");
    }
    let ijs = &files[0];
    let toml = &files[1];

    let mut run_list = match fs::read_to_string(&toml) {
        Ok(content) => RunList::open(content)?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => RunList::empty(),
        Err(e) => return Err(e).with_context(|| anyhow!("unexpected error reading {toml:?}")),
    };

    // io::ErrorKind::IsADirectory isn't stable :(
    let contents = if fs::metadata(&ijs)
        .with_context(|| anyhow!("touching {ijs:?}"))?
        .is_dir()
    {
        read_ijs_dir(ijs)?
    } else {
        read_ijs_lines(&fs::read_to_string(&ijs)?)
    };

    for (ctx, content) in contents {
        run_list
            .add(&content)
            .with_context(|| anyhow!("content:\n{content:?}"))
            .with_context(|| anyhow!("unable to capture result for {ctx}"))
            ?;
    }

    fs::write(toml, run_list.save()?)?;
    Ok(())
}
