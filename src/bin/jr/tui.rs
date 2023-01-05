use std::borrow::Cow;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use colored::{Color, Colorize as _};
use rustyline::completion::Completer;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hint, Hinter};
use rustyline::Context;
use rustyline_derive::{Completer, Helper, Highlighter, Validator};

use crate::EvalState;
use jr::{scan_with_locations, Ctx, Word};

pub fn drive() -> Result<()> {
    let data_dir = match directories::ProjectDirs::from("github", "AshyIsMe", "jr") {
        Some(dirs) => dirs.data_dir().to_path_buf(),
        None => PathBuf::new(),
    };
    fs::create_dir_all(&data_dir)?;
    let hist_file = data_dir.join("jhistory");

    let h = DIYHinter::default();

    let mut rl = rustyline::Editor::<DIYHinter>::new()?;
    rl.set_helper(Some(h));
    if hist_file.exists() {
        rl.load_history(&hist_file)?;
    }
    rl.set_auto_add_history(true);

    let mut ctx = Ctx::root();

    let mut state = EvalState::Regular;

    loop {
        let line = match rl.readline(if state == EvalState::Regular {
            "   "
        } else {
            ""
        }) {
            Ok(line) => line,
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => break,
            Err(other) => Err(other)?,
        };

        state = super::eval(&line, &mut ctx)?;
        if state == EvalState::Done {
            break;
        }
    }

    rl.save_history(&hist_file)?;
    Ok(())
}

#[derive(Completer, Helper, Validator, Highlighter, Default)]
struct DIYHinter {
    #[rustyline(Highlighter)]
    highlighter: DiHigh,

    #[rustyline(Completer)]
    completer: DiComplete,
}

#[derive(Default)]
struct DiHigh;

impl Highlighter for DiHigh {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        let v = match scan_with_locations(line) {
            Ok(v) => v,
            // this is pretty much unreachable, scan never fails right now
            Err(_) => return line.red().to_string().into(),
        };

        let mut buf = line.to_string();
        for (pos, word) in v.into_iter().rev() {
            let colour = match word {
                Word::Verb(_) => Color::BrightGreen,
                Word::Noun(_) => Color::BrightBlue,
                Word::Adverb(_, _) => Color::BrightRed,
                Word::Name(_) => Color::BrightWhite,
                Word::Conjunction(_, _) => Color::BrightCyan,
                _ => continue,
            };

            let range = pos.0..=pos.1;
            buf.replace_range(range.clone(), &format!("{}", line[range].color(colour)));
        }

        buf.into()
    }

    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
        true
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        format!("{}", hint.bright_black()).into()
    }
}

struct CommandHint(String);

impl Hint for CommandHint {
    fn display(&self) -> &str {
        &self.0
    }

    fn completion(&self) -> Option<&str> {
        None
    }
}

impl CommandHint {
    fn new(val: impl ToString) -> Self {
        CommandHint(val.to_string())
    }
}

impl Hinter for DIYHinter {
    type Hint = CommandHint;

    fn hint(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<CommandHint> {
        if line.is_empty() {
            return None;
        }

        let v = match jr::scan(line) {
            Ok(v) => v,
            Err(_) => return None,
        };

        let mut helped = HashSet::with_capacity(4);
        let it = v.into_iter().rev().flat_map(|w| match w {
            Word::Verb(imp) => {
                if let Some(token) = imp.token() {
                    if helped.insert(token.to_string()) {
                        help(token).into_iter().collect()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
            Word::Name(name) if name.len() >= 3 => {
                search_help(&name).into_iter().map(|(_, c)| c).collect()
            }
            _ => vec![],
        });

        let mut buf = String::with_capacity(64);

        for word in it {
            if buf.len() > 70 {
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
        Some(CommandHint::new(format!("  {}", buf)))
    }
}

#[derive(Default)]
struct DiComplete;

impl Completer for DiComplete {
    type Candidate = &'static str;

    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let v = match jr::scan_with_locations(line) {
            Ok(v) => v,
            Err(_) => return Ok((0, vec![])),
        };
        let (pos, word) = match v.into_iter().rev().find_map(|(p, w)| match w {
            Word::Name(n) => Some((p, n)),
            _ => None,
        }) {
            Some(word) => word,
            None => return Ok((0, vec![])),
        };
        Ok((
            pos.0,
            search_help(&word).into_iter().map(|(c, _)| c).collect(),
        ))
    }
}

struct Primitive(
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
);

const fn primitive(
    n: &'static str,
    m: &'static str,
    d: &'static str,
    rm: &'static str,
    rdx: &'static str,
    rdy: &'static str,
) -> Primitive {
    Primitive(n, m, d, rm, rdx, rdy)
}

impl Primitive {
    fn help_str(&self) -> String {
        let Primitive(n, m, d, rm, rdx, rdy) = self;
        format!("{n:?}: '{m}' or '{d}' ({rm} {rdx} {rdy})")
    }
}

fn help(token: &str) -> Option<String> {
    for prim in PRIMITIVES {
        if prim.0 == token {
            return Some(prim.help_str());
        }
    }
    None
}

fn search_help(name: &str) -> Vec<(&'static str, String)> {
    PRIMITIVES
        .iter()
        .filter_map(|p| {
            let Primitive(n, m, d, _rm, _rdx, _rdy) = *p;
            if m.contains(name) || d.contains(name) {
                Some((n, p.help_str()))
            } else {
                None
            }
        })
        .collect()
}

const PRIMITIVES: [Primitive; 60] = [
    primitive("=", "self classify", "equal", "_", "0", "0"),
    primitive("<", "box", "less than", "_", "0", "0"),
    primitive("<.", "floor", "lesser of min", "0", "0", "0"),
    primitive("<:", "decrement", "less or equal", "0", "0", "0"),
    primitive(">", "open", "larger than", "0", "0", "0"),
    primitive(">.", "ceiling", "larger of max", "0", "0", "0"),
    primitive(">:", "increment", "larger or equal", "0", "0", "0"),
    primitive("+", "conjugate", "plus", "0", "0", "0"),
    primitive("+.", "real imaginary", "gcd or", "0", "0", "0"),
    primitive("+:", "double", "not or", "0", "0", "0"),
    primitive("*", "signum", "times", "0", "0", "0"),
    primitive("*.", "lengthangle", "lcm and", "0", "0", "0"),
    primitive("*:", "square", "not and", "0", "0", "0"),
    primitive("-", "negate", "minus", "0", "0", "0"),
    primitive("-.", "not", "less", "0", "_", "_"),
    primitive("-:", "halve", "match", "_", "_", "0"),
    primitive("%", "reciprocal", "divide", "0", "0", "0"),
    primitive("%.", "matrix inverse", "matrix divide", "2", "_", "2"),
    primitive("%:", "square root", "root", "0", "0", "0"),
    primitive("^", "exponential", "power", "0", "0", "0"),
    primitive("^.", "natural log", "logarithm", "0", "0", "0"),
    primitive("$", "shape of", "shape", "_", "1", "_"),
    primitive("~:", "nub sieve", "not equal", "_", "0", "0"),
    primitive("|", "magnitude", "residue", "0", "0", "0"),
    primitive("|.", "reverse", "rotate shift", "_", "_", "_"),
    primitive(",", "ravel", "append", "_", "_", "_"),
    primitive(",.", "ravel items", "stitch", "_", "_", "_"),
    primitive(",:", "itemize", "laminate", "_", "_", "_"),
    primitive(";", "raze", "link", "_", "_", "_"),
    primitive(";:", "words", "sequential machine", "1", "_", "_"),
    primitive("#", "tally", "copy", "_", "1", "_"),
    primitive("#.", "base ", "base", "1", "1", "1"),
    primitive("#:", "antibase ", "antibase", "_", "1", "0"),
    primitive("!", "factorial", "out of", "0", "0", "0"),
    primitive("/:", "grade up", "sort", "_", "_", "_"),
    primitive("\\:", "grade down", "sort", "_", "_", "_"),
    primitive("[", "same", "left", "_", "_", "_"),
    primitive("]", "same", "right", "_", "_", "_"),
    primitive("{", "catalogue", "from", "1", "0", "_"),
    primitive("{.", "head", "take", "_", "1", "_"),
    primitive("{:", "tail", "not implemented dyad", "_", "_", "_"),
    primitive("{:", "map", "fetch", "_", "1", "_"),
    primitive("}.", "behead", "drop", "_", "1", "_"),
    primitive("\".", "do", "numbers", "1", "_", "_"),
    primitive("\":", "default format", "format", "_", "1", "_"),
    primitive("?", "roll", "deal", "0", "0", "0"),
    primitive("?.", "roll", "deal fixed seed", "_", "0", "0"),
    primitive("A.", "anagram index", "anagram", "1", "0", "_"),
    primitive("C.", "cycledirect", "permute", "1", "1", "_"),
    primitive("e.", "raze in", "member in", "_", "_", "_"),
    primitive("i.", "integers", "index of", "1", "_", "_"),
    primitive("i:", "steps", "index of last", "0", "_", "_"),
    primitive("I.", "indices", "interval index", "1", "_", "_"),
    primitive("j.", "imaginary", "complex", "0", "0", "0"),
    primitive("o.", "pi times", "circle function", "0", "0", "0"),
    primitive("p.", "roots", "polynomial", "1", "1", "0"),
    primitive("p..", "poly deriv", "poly integral", "1", "0", "1"),
    primitive("q:", "prime factors", "prime exponents", "0", "0", "0"),
    primitive("r.", "angle", "polar", "0", "0", "0"),
    primitive("x:", "extend precision", "num denom", "_", "_", "_"),
];
