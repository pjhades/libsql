use std::path::PathBuf;
use anyhow::Result;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::{CompletionType, Config, Context, Editor};
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};

const HISTORY_FILENAME: &str = ".libsql_history";

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

pub struct CliInput {
    pub line_editor: Editor<ShellHelper, FileHistory>,
    pub history_path: PathBuf,
}

impl CliInput {
    pub fn new() -> Result<Self> {
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::Circular)
            .build();
        let mut line_editor = Editor::with_config(config)?;
        let helper = ShellHelper {
            completer: ShellCompleter::new(),
        };
        line_editor.set_helper(Some(helper));
        let mut history_path = home::home_dir().unwrap_or_default();
        history_path.push(HISTORY_FILENAME);
        line_editor.load_history(history_path.as_path())?;
        Ok(Self { line_editor, history_path })
    }
}
