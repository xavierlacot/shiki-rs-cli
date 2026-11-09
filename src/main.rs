use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use serde_json::Value;
use shiki_rs_cli::Highlighter;

/// Highlight source code as Shiki-compatible HTML.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// The file to highlight (a JSON array of snippets with --json)
    #[arg(required_unless_present_any = ["list_languages", "list_themes"])]
    file: Option<PathBuf>,

    /// The language of the code (unknown languages fall back to plain text)
    #[arg(long, default_value = "text")]
    lang: String,

    /// The theme to use
    #[arg(long, default_value = "monokai")]
    theme: String,

    /// Process a JSON array of `{ "input", "lang"?, "theme"? }` objects
    #[arg(long)]
    json: bool,

    /// List the available languages and their aliases
    #[arg(long, exclusive = true)]
    list_languages: bool,

    /// List the available themes
    #[arg(long, exclusive = true)]
    list_themes: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let highlighter = Highlighter::new()?;

    if cli.list_languages {
        for (name, aliases) in highlighter.languages() {
            if aliases.is_empty() {
                println!("{name}");
            } else {
                println!("{name} ({})", aliases.join(", "));
            }
        }
        return Ok(());
    }

    if cli.list_themes {
        for theme in shiki_rs_cli::theme_names() {
            println!("{theme}");
        }
        return Ok(());
    }

    let file = cli.file.expect("clap requires a file");
    let content =
        fs::read_to_string(&file).map_err(|e| format!("cannot read {}: {e}", file.display()))?;

    if !cli.json {
        println!(
            "{}",
            highlighter.highlight(&content, &cli.lang, &cli.theme)?
        );
        return Ok(());
    }

    let mut items: Vec<Value> = serde_json::from_str(&content)
        .map_err(|e| format!("invalid JSON in {}: {e}", file.display()))?;
    for (index, item) in items.iter_mut().enumerate() {
        let object = item
            .as_object_mut()
            .ok_or_else(|| format!("item {index} is not an object"))?;
        let field = |name: &str, default: &str| -> Result<String, String> {
            match object.get(name) {
                None | Some(Value::Null) => Ok(default.to_string()),
                Some(Value::String(s)) => Ok(s.clone()),
                Some(_) => Err(format!("item {index}: \"{name}\" must be a string")),
            }
        };
        let input = match object.get("input") {
            Some(Value::String(s)) => s.clone(),
            _ => return Err(format!("item {index}: \"input\" must be a string").into()),
        };
        let lang = field("lang", &cli.lang)?;
        let theme = field("theme", &cli.theme)?;
        let output = highlighter.highlight(&input, &lang, &theme)?;
        object.insert("output".into(), Value::String(output));
    }
    println!("{}", serde_json::to_string_pretty(&items)?);
    Ok(())
}
