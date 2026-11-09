//! Syntax highlighting producing the same HTML markup as Shiki's `codeToHtml`,
//! using the giallo TextMate engine and its built-in grammars and themes.

use giallo::{
    Color, FontStyle, HighlightOptions, HighlightedCode, PLAIN_GRAMMAR_NAME, Registry, Style,
    ThemeVariant,
};

pub use giallo::Error;

mod themes;

/// The built-in theme names, sorted.
pub fn theme_names() -> impl Iterator<Item = &'static str> {
    themes::THEMES.iter().map(|(name, _, _)| *name)
}

/// Colors replaced in the rendered output.
const COLOR_REPLACEMENTS: &[(&str, &str)] = &[("#272822", "#1f2937")];

/// The language used when the requested one is unknown.
pub const FALLBACK_LANGUAGE: &str = "text";

/// Highlights code into Shiki-compatible HTML.
pub struct Highlighter {
    registry: Registry,
}

impl Highlighter {
    /// Loads the built-in grammars and themes.
    pub fn new() -> Result<Self, Error> {
        let mut registry = Registry::builtin()?;
        registry.link_grammars();
        Ok(Self { registry })
    }

    /// Whether the language (or alias) is known.
    pub fn has_language(&self, lang: &str) -> bool {
        self.registry.contains_grammar(lang)
    }

    /// Whether the theme is known.
    pub fn has_theme(&self, theme: &str) -> bool {
        self.registry.contains_theme(theme)
    }

    /// All language names with their aliases, sorted by name.
    pub fn languages(&self) -> Vec<(String, Vec<String>)> {
        let mut langs: Vec<_> = self
            .registry
            .get_grammar_names()
            .into_iter()
            .map(|g| (g.name.to_string(), g.aliases.to_vec()))
            .collect();
        langs.sort();
        langs
    }

    /// Highlights `input` and renders it as HTML. Unknown languages fall back
    /// to plain text, unknown themes are an error.
    pub fn highlight(&self, input: &str, lang: &str, theme: &str) -> Result<String, Error> {
        let theme = theme.to_lowercase();
        // Like vscode-textmate (used by Shiki), merge same-style tokens first;
        // whitespace is merged afterwards, in `merge_whitespace`.
        let options = HighlightOptions::new(lang, ThemeVariant::Single(&theme))
            .fallback_to_plain(true)
            .merge_whitespace(false)
            .merge_same_style_tokens(true);
        let highlighted = self.registry.highlight(input, &options)?;
        Ok(render(&highlighted, &theme))
    }
}

/// Renders highlighted code with the markup of Shiki's `codeToHtml`.
fn render(highlighted: &HighlightedCode, theme_name: &str) -> String {
    let ThemeVariant::Single(theme) = highlighted.theme else {
        unreachable!("only single themes are requested");
    };
    let default = theme.default_style;
    let is_plain = highlighted.language == PLAIN_GRAMMAR_NAME;

    let mut html = format!(
        r#"<pre class="shiki {}" style="background-color:{};color:{}" tabindex="0"><code>"#,
        escape_attribute(theme_name),
        pre_color(default.background, theme_name, |(_, bg, _)| bg),
        pre_color(default.foreground, theme_name, |(_, _, fg)| fg),
    );

    for (i, line) in highlighted.tokens.iter().enumerate() {
        if i > 0 {
            html.push('\n');
        }
        html.push_str(r#"<span class="line">"#);
        let tokens = line.iter().map(|token| {
            let ThemeVariant::Single(style) = token.style else {
                unreachable!("only single themes are requested");
            };
            // Shiki does not color plain text tokens
            (token.text.clone(), (!is_plain).then_some(style))
        });
        for (text, style) in merge_whitespace(tokens.collect()) {
            match style {
                Some(style) => {
                    html.push_str(r#"<span style=""#);
                    html.push_str(&token_style(&style, &default));
                    html.push_str(r#"">"#);
                }
                None => html.push_str("<span>"),
            }
            escape_text(&text, &mut html);
            html.push_str("</span>");
        }
        html.push_str("</span>");
    }

    html.push_str("</code></pre>");
    html
}

/// A token to render: its text and its style (`None` renders an unstyled span).
type Token = (String, Option<Style>);

/// Port of Shiki's `mergeWhitespaceTokens`: whitespace-only tokens are prepended
/// to the next token, unless it is underlined or struck through, in which case
/// they are emitted as a separate unstyled token. Trailing whitespace is kept.
fn merge_whitespace(line: Vec<Token>) -> Vec<Token> {
    let is_decorated = |style: &Option<Style>| {
        style.is_some_and(|s| {
            s.font_style.contains(FontStyle::UNDERLINE)
                || s.font_style.contains(FontStyle::STRIKETHROUGH)
        })
    };
    let mut merged = Vec::with_capacity(line.len());
    let mut carry = String::new();
    let count = line.len();
    for (index, (text, style)) in line.into_iter().enumerate() {
        let can_merge = !is_decorated(&style);
        let is_whitespace = !text.is_empty() && text.chars().all(char::is_whitespace);
        if can_merge && is_whitespace && index + 1 < count {
            carry.push_str(&text);
        } else if carry.is_empty() {
            merged.push((text, style));
        } else if can_merge {
            merged.push((std::mem::take(&mut carry) + &text, style));
        } else {
            merged.push((std::mem::take(&mut carry), None));
            merged.push((text, style));
        }
    }
    merged
}

/// Builds the inline style of a token, in the same order as Shiki.
fn token_style(style: &Style, default: &Style) -> String {
    let mut parts = vec![format!("color:{}", color(style.foreground))];
    if style.background != default.background {
        parts.push(format!("background-color:{}", color(style.background)));
    }
    let font = style.font_style;
    if font.contains(FontStyle::ITALIC) {
        parts.push("font-style:italic".into());
    }
    if font.contains(FontStyle::BOLD) {
        parts.push("font-weight:bold".into());
    }
    match (
        font.contains(FontStyle::UNDERLINE),
        font.contains(FontStyle::STRIKETHROUGH),
    ) {
        (true, true) => parts.push("text-decoration:underline line-through".into()),
        (true, false) => parts.push("text-decoration:underline".into()),
        (false, true) => parts.push("text-decoration:line-through".into()),
        (false, false) => {}
    }
    parts.join(";")
}

/// Formats a color as hex, applying the color replacements.
fn color(c: Color) -> String {
    replace_color(c.as_hex())
}

fn replace_color(hex: String) -> String {
    COLOR_REPLACEMENTS
        .iter()
        .find(|(from, _)| from.eq_ignore_ascii_case(&hex))
        .map_or(hex, |(_, to)| (*to).to_string())
}

/// Formats a `<pre>` color with the original case used by Shiki, when known.
fn pre_color(
    c: Color,
    theme_name: &str,
    pick: impl Fn(&(&str, &'static str, &'static str)) -> &'static str,
) -> String {
    let shiki = themes::THEMES
        .iter()
        .find(|t| t.0 == theme_name)
        .map(pick)
        .filter(|s| Color::from_hex(s).is_ok_and(|parsed| parsed == c));
    let hex = shiki.map_or_else(|| c.as_hex(), str::to_string);
    replace_color(hex)
}

/// Escapes text content the way hast-util-to-html does in Shiki.
fn escape_text(text: &str, out: &mut String) {
    for c in text.chars() {
        match c {
            '&' => out.push_str("&#x26;"),
            '<' => out.push_str("&#x3C;"),
            _ => out.push(c),
        }
    }
}

fn escape_attribute(value: &str) -> String {
    value
        .replace('&', "&#x26;")
        .replace('"', "&#x22;")
        .replace('<', "&#x3C;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn highlighter() -> Highlighter {
        Highlighter::new().expect("builtin registry loads")
    }

    #[test]
    fn escapes_like_shiki() {
        let mut out = String::new();
        escape_text("a < b && c > d \"e\"", &mut out);
        assert_eq!(out, "a &#x3C; b &#x26;&#x26; c > d \"e\"");
    }

    #[test]
    fn replaces_monokai_background() {
        let html = highlighter()
            .highlight("let a = 1;", "rust", "monokai")
            .unwrap();
        assert!(html.starts_with(
            r#"<pre class="shiki monokai" style="background-color:#1f2937;color:#F8F8F2" tabindex="0"><code>"#
        ));
    }

    #[test]
    fn unknown_language_falls_back_to_plain_text() {
        let h = highlighter();
        let html = h.highlight("a <b>", "not-a-language", "monokai").unwrap();
        assert_eq!(
            html,
            h.highlight("a <b>", FALLBACK_LANGUAGE, "monokai").unwrap()
        );
        assert!(html.contains(r#"<span class="line"><span>a &#x3C;b></span></span>"#));
    }

    #[test]
    fn aliases_are_case_insensitive() {
        let h = highlighter();
        assert_eq!(
            h.highlight("const a = 1;", "TS", "monokai").unwrap(),
            h.highlight("const a = 1;", "typescript", "monokai")
                .unwrap()
        );
    }

    #[test]
    fn keeps_trailing_empty_line() {
        let html = highlighter().highlight("a\n", "text", "monokai").unwrap();
        assert!(html.ends_with("</span></span>\n<span class=\"line\"></span></code></pre>"));
    }

    #[test]
    fn all_listed_themes_exist() {
        let h = highlighter();
        for theme in theme_names() {
            assert!(h.has_theme(theme), "missing theme {theme}");
        }
    }

    #[test]
    fn keeps_shiki_case_for_pre_colors() {
        let html = highlighter().highlight("a", "text", "github-dark").unwrap();
        assert!(html.contains(r#"style="background-color:#24292e;color:#e1e4e8""#));
    }

    #[test]
    fn unknown_theme_is_an_error() {
        assert!(highlighter().highlight("a", "text", "nope").is_err());
    }
}
