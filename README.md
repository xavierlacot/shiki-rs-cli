# shiki-rs-cli

A standalone command line syntax highlighter that produces the same HTML as [Shiki](https://shiki.style/), shipped as a single static binary of about 4.5 MiB, with no runtime dependency (no Node.js, no Bun, no shared libraries).

It is written in Rust on top of [giallo](https://github.com/getzola/giallo), a TextMate highlighting engine that uses Shiki's grammars and themes. This project is not affiliated with the Shiki project.

## Install

Download the latest Linux x64 binary from the [releases page](https://github.com/xavierlacot/shiki-rs-cli/releases), or build it yourself (see [Development](#development)).

## Usage

### Highlight a file

```bash
shiki-rs-cli /path/to/source-code.xyz --lang=language --theme=theme
```

- `--lang` defaults to `text`. Unknown languages fall back to plain text.
- `--theme` defaults to `monokai`. Unknown themes are an error.
- `--list-languages` and `--list-themes` print what is available: all of Shiki's languages (with their aliases) and themes.

Errors are printed on stderr and the command exits with a non-zero status.

### JSON mode

The `--json` option processes multiple code snippets at once. The input file is a JSON array of objects:

```json
[
  {
    "input": "source code",
    "lang": "language",
    "theme": "theme"
  }
]
```

`lang` and `theme` are optional and default to the `--lang` and `--theme` options. Each object gets an `output` field containing the highlighted HTML. Any other field is preserved, which makes it easy to match results with their source:

```bash
shiki-rs-cli input.json --json > output.json
```

```json
[
  {
    "input": "function greet(): void {\n  console.log('Hello, world!');\n}\ngreet();",
    "lang": "typescript",
    "reference": "550e8400-e29b-41d4-a716-446655440004",
    "output": "<pre class=\"shiki monokai\" style=\"background-color:#1f2937;color:#F8F8F2\" tabindex=\"0\"><code>...</code></pre>"
  }
]
```

## Compatibility with Shiki

The output reproduces the markup of Shiki's `codeToHtml`: the same `<pre>`, `<code>` and `<span>` structure, the same inline styles and the same escaping. The `monokai` background (`#272822`) is replaced with `#1f2937`.

The result is visually identical to Shiki: every character gets the same color and font style. Most outputs are also byte-identical; the remaining differences are invisible:

- Adjacent tokens of the same color may be split differently. Shiki (via vscode-textmate) only merges tokens that also share a token type (string, comment...), which giallo does not expose.
- Some themes define colors as `#RRGGBBFF`, which Shiki keeps as is and giallo writes as `#RRGGBB`.

## Development

The project is managed with [Castor](https://castor.jolicode.com/). It requires Rust 1.85+ and, for the static release binary, the musl target and a musl C compiler (`apt install musl-tools`).

```bash
castor setup      # install the musl Rust target
castor build      # debug build
castor run tests/fixtures/sample.ts --lang=ts
castor test       # unit and integration tests
castor qa:fmt     # format the code
castor qa:lint    # clippy
castor qa:check   # formatting check, clippy and tests
castor release    # static release binary in dist/shiki-rs-cli
castor licenses   # third party licenses in dist/THIRD_PARTY_LICENSES.txt
castor clean
```

## License

The source code of this project is MIT licensed, see [LICENSE](LICENSE).

The release binaries also embed third party software under their own licenses: [giallo](https://github.com/getzola/giallo) (licensed under the [EUPL-1.2](https://eupl.eu/1.2/en/)), other Rust crates, and Shiki's [grammars and themes](https://github.com/shikijs/textmate-grammars-themes). Their licenses and notices are listed in the `THIRD_PARTY_LICENSES.txt` file attached to each release, which `castor licenses` generates (it requires [cargo-about](https://github.com/EmbarkStudios/cargo-about): `cargo install --locked cargo-about --features cli`).
