use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use colored::Colorize as _;
use jr::Word;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::hint::{Hint, Hinter};
use rustyline::Context;
use rustyline_derive::{Completer, Helper, Highlighter, Validator};

pub fn drive() -> Result<()> {
    let data_dir = match directories::ProjectDirs::from("github", "AshyIsMe", "jr") {
        Some(dirs) => dirs.data_dir().to_path_buf(),
        None => PathBuf::new(),
    };
    fs::create_dir_all(&data_dir)?;
    let hist_file = data_dir.join("jhistory");

    let h = DIYHinter {};
    let mut rl = rustyline::Editor::<DIYHinter>::new()?;
    rl.set_helper(Some(h));
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

#[derive(Completer, Helper, Validator, Highlighter)]
struct DIYHinter {}

struct CommandHint(String);

impl Hint for CommandHint {
    fn display(&self) -> &str {
        &self.0
    }

    fn completion(&self) -> Option<&str> {
        Some(&self.0)
    }
}

impl CommandHint {
    fn new(val: impl ToString) -> Self {
        CommandHint(val.to_string())
    }
}

impl Hinter for DIYHinter {
    type Hint = CommandHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<CommandHint> {
        if line.is_empty() || pos < line.len() {
            return None;
        }

        let v = match jr::scan(line) {
            Ok(v) => v,
            Err(_) => return None,
        };

        let mut it = v.into_iter().rev().filter_map(|w| match w {
            Word::Verb(token, _) => help(&token),
            _ => None,
        });

        let mut buf = String::with_capacity(64);

        while let Some(word) = it.next() {
            if buf.len() > 100 {
                break;
            }
            buf.push_str(&word);
            buf.push_str("; ");
        }

        if !buf.is_empty() {
            buf.truncate(buf.len() - "; ".len());
        }

        if buf.is_empty() {
            return None;
        }
        Some(CommandHint::new(format!("  {}", buf).bright_black()))
    }
}

fn help(token: &str) -> Option<String> {
    let primitive = |n, m, d, rm, rdx, rdy| format!("{n:?}: '{m}' or '{d}' ({rm} {rdx} {rdy})");
    Some(match token {
        "=" => primitive("=", "self classify", "equal", "_", "0", "0"),
        "<" => primitive("<", "box", "less than", "_", "0", "0"),
        "<." => primitive("<.", "floor", "lesser of min", "0", "0", "0"),
        "<:" => primitive("<:", "decrement", "less or equal", "0", "0", "0"),
        ">" => primitive(">", "open", "larger than", "0", "0", "0"),
        ">." => primitive(">.", "ceiling", "larger of max", "0", "0", "0"),
        ">:" => primitive(">:", "increment", "larger or equal", "0", "0", "0"),

        "+" => primitive("+", "conjugate", "plus", "0", "0", "0"),
        "+." => primitive("+.", "real imaginary", "gcd or", "0", "0", "0"),
        "+:" => primitive("+:", "double", "not or", "0", "0", "0"),
        "*" => primitive("*", "signum", "times", "0", "0", "0"),
        "*." => primitive("*.", "lengthangle", "lcm and", "0", "0", "0"),
        "*:" => primitive("*:", "square", "not and", "0", "0", "0"),
        "-" => primitive("-", "negate", "minus", "0", "0", "0"),
        "-." => primitive("-.", "not", "less", "0", "_", "_"),
        "-:" => primitive("-:", "halve", "match", "_", "_", "0"),
        "%" => primitive("%", "reciprocal", "divide", "0", "0", "0"),
        "%." => primitive("%.", "matrix inverse", "matrix divide", "2", "_", "2"),
        "%:" => primitive("%:", "square root", "root", "0", "0", "0"),

        "^" => primitive("^", "exponential", "power", "0", "0", "0"),
        "^." => primitive("^.", "natural log", "logarithm", "0", "0", "0"),
        "$" => primitive("$", "shape of", "shape", "_", "1", "_"),
        "~:" => primitive("~:", "nub sieve", "not equal", "_", "0", "0"),
        "|" => primitive("|", "magnitude", "residue", "0", "0", "0"),
        "|." => primitive("|.", "reverse", "rotate shift", "_", "_", "_"),

        "," => primitive(",", "ravel", "append", "_", "_", "_"),
        ",." => primitive(",.", "ravel items", "stitch", "_", "_", "_"),
        ",:" => primitive(",:", "itemize", "laminate", "_", "_", "_"),
        ";" => primitive(";", "raze", "link", "_", "_", "_"),
        ";:" => primitive(";:", "words", "sequential machine", "1", "_", "_"),

        "#" => primitive("#", "tally", "copy", "_", "1", "_"),
        "#." => primitive("#.", "base ", "base", "1", "1", "1"),
        "#:" => primitive("#:", "antibase ", "antibase", "_", "1", "0"),
        "!" => primitive("!", "factorial", "out of", "0", "0", "0"),
        "/:" => primitive("/:", "grade up", "sort", "_", "_", "_"),
        "\\:" => primitive("\\:", "grade down", "sort", "_", "_", "_"),

        "[" => primitive("[", "same", "left", "_", "_", "_"),
        "]" => primitive("]", "same", "right", "_", "_", "_"),
        "{" => primitive("{", "catalogue", "from", "1", "0", "_"),
        "{." => primitive("{.", "head", "take", "_", "1", "_"),
        "{:" => primitive("{:", "tail", "not implemented dyad", "_", "_", "_"),
        "{::" => primitive("{:", "map", "fetch", "_", "1", "_"),
        "}." => primitive("}.", "behead", "drop", "_", "1", "_"),

        "\"." => primitive("\".", "do", "numbers", "1", "_", "_"),
        "\":" => primitive("\":", "default format", "format", "_", "1", "_"),
        "?" => primitive("?", "roll", "deal", "0", "0", "0"),
        "?." => primitive("?.", "roll", "deal fixed seed", "_", "0", "0"),

        "A." => primitive("A.", "anagram index", "anagram", "1", "0", "_"),
        "C." => primitive("C.", "cycledirect", "permute", "1", "1", "_"),
        "e." => primitive("e.", "raze in", "member in", "_", "_", "_"),

        "i." => primitive("i.", "integers", "index of", "1", "_", "_"),
        "i:" => primitive("i:", "steps", "index of last", "0", "_", "_"),
        "I." => primitive("I.", "indices", "interval index", "1", "_", "_"),
        "j." => primitive("j.", "imaginary", "complex", "0", "0", "0"),
        "o." => primitive("o.", "pi times", "circle function", "0", "0", "0"),
        "p." => primitive("p.", "roots", "polynomial", "1", "1", "0"),
        "p.." => primitive("p..", "poly deriv", "poly integral", "1", "0", "1"),

        "q:" => primitive("q:", "prime factors", "prime exponents", "0", "0", "0"),
        "r." => primitive("r.", "angle", "polar", "0", "0", "0"),
        "x:" => primitive("x:", "extend precision", "num denom", "_", "_", "_"),
        _ => return None,
    })
}
