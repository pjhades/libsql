use anyhow::Result;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::{CompletionType, Config, Context, Editor};
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};
use std::path::PathBuf;

const HISTORY_FILENAME: &str = ".libsql_history";
const MAIN_PROMPT: &str = "libsql> ";
const CONT_PROMPT: &str = "   ...> ";

#[derive(Default)]
struct ShellCompleter {}

impl ShellCompleter {
    fn new() -> Self {
        Self::default()
    }

    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _: &Context,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let mut pairs: Vec<Pair> = vec![];
        let commands = vec![
            ".echo",
            ".headers",
            ".help",
            ".indexes",
            ".nullvalue",
            ".print",
            ".prompt",
            ".quit",
            ".show",
            ".tables",
        ];
        for command in commands {
            if command.starts_with(line) {
                pairs.push(Pair {
                    display: command.to_string(),
                    replacement: command.to_string(),
                })
            }
        }
        Ok((0, pairs))
    }
}

#[derive(Helper, Hinter, Validator, Highlighter)]
// XXX this doesn't need to be pub
pub struct ShellHelper {
    #[rustyline(Completer)]
    completer: ShellCompleter,
}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

pub enum Input {
    DotCommand(String),
    Sql(String),
}

pub struct CliReader {
    pub editor: Editor<ShellHelper, FileHistory>,
    pub history: PathBuf,
    main_prompt: String,
    cont_prompt: String,
}

impl CliReader {
    pub fn new() -> Result<Self> {
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::Circular)
            .build();
        let mut editor = Editor::with_config(config)?;
        let helper = ShellHelper {
            completer: ShellCompleter::new(),
        };
        editor.set_helper(Some(helper));
        let mut history = home::home_dir().unwrap_or_default();
        history.push(HISTORY_FILENAME);
        editor.load_history(history.as_path())?;
        Ok(Self {
            editor,
            history,
            main_prompt: MAIN_PROMPT.to_string(),
            cont_prompt: CONT_PROMPT.to_string(),
        })
    }

    pub fn read_input(&mut self) -> Result<Input> {
        let mut accumulated = String::new();
        loop {
            let line = self.editor.readline(if accumulated.is_empty() {
                &self.main_prompt
            } else {
                &self.cont_prompt
            })?;
            self.editor.add_history_entry(&line)?;
            if line.starts_with('.') {
                if !accumulated.is_empty() {
                    accumulated.push_str(&line);
                } else {
                    return Ok(Input::DotCommand(line));
                }
            } else if !line.starts_with('#') {
                let trimmed = line.trim_end();
                accumulated.push_str(trimmed);
                if trimmed.ends_with(';') || trimmed == "go" || trimmed == "/" {
                    break;
                }
            }
        }
        Ok(Input::Sql(accumulated))
    }
}
