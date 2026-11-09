<?php

defined('CASTOR_USE_CHDIR') || define('CASTOR_USE_CHDIR', true);

use Castor\Attribute\AsRawTokens;
use Castor\Attribute\AsTask;

use function Castor\fs;
use function Castor\io;
use function Castor\run;

const RELEASE_TARGET = 'x86_64-unknown-linux-musl';
const BINARY = 'shiki-rs-cli';
const DIST_BINARY = __DIR__ . '/dist/' . BINARY;

#[AsTask(description: 'Install the Rust target needed for the static release binary')]
function setup(): void
{
    run(['rustup', 'target', 'add', RELEASE_TARGET]);
    io()->note('The static build also needs a musl C compiler (e.g. `apt install musl-tools`).');
}

#[AsTask(description: 'Build the debug binary')]
function build(): void
{
    run(['cargo', 'build']);
}

#[AsTask(description: 'Build the static, standalone release binary into dist/')]
function release(): void
{
    run(['cargo', 'build', '--release', '--locked', '--target', RELEASE_TARGET]);
    fs()->copy(__DIR__ . '/target/' . RELEASE_TARGET . '/release/' . BINARY, DIST_BINARY, true);
    fs()->chmod(DIST_BINARY, 0o755);
    io()->success(sprintf('Built %s (%.1f MiB)', 'dist/' . BINARY, filesize(DIST_BINARY) / 1024 / 1024));
}

#[AsTask(name: 'run', description: 'Run the CLI through cargo, e.g. `castor run tests/fixtures/sample.ts --lang=ts`', ignoreValidationErrors: true)]
function run_cli(#[AsRawTokens] array $args = []): void
{
    run(['cargo', 'run', '--quiet', '--', ...$args]);
}

#[AsTask(description: 'Run the unit and integration tests')]
function test(): void
{
    run(['cargo', 'test']);
}

#[AsTask(namespace: 'qa', description: 'Format the code')]
function fmt(): void
{
    run(['cargo', 'fmt']);
}

#[AsTask(namespace: 'qa', description: 'Lint the code with clippy')]
function lint(): void
{
    run(['cargo', 'clippy', '--all-targets', '--', '-D', 'warnings']);
}

#[AsTask(namespace: 'qa', description: 'Check formatting, lint and run the tests (what CI should run)')]
function check(): void
{
    run(['cargo', 'fmt', '--check']);
    lint();
    test();
}

#[AsTask(description: 'Remove build artifacts')]
function clean(): void
{
    run(['cargo', 'clean']);
    fs()->remove(__DIR__ . '/dist');
}
