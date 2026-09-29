# cddlconv

[![crates.io](https://img.shields.io/crates/v/cddlconv?style=flat-square)](https://crates.io/crates/cddlconv)
[![license](https://img.shields.io/crates/l/cddlconv?style=flat-square)](https://github.com/google/cddlconv)
[![ci](https://img.shields.io/github/actions/workflow/status/google/cddlconv/ci.yml?label=ci&style=flat-square)](https://github.com/google/cddlconv/actions/workflows/ci.yml)

A commandline utility for converting CDDL to various formats.

## Usage

1.  Clone this repo and `cd` into it.
2.  `cargo run -- path/to/file.cddl`

The default output format is TypeScript. Zod schemas can be generated for either
major version without changing their runtime schema expressions:

```sh
cargo run -- path/to/file.cddl --format zod   # Zod v3 type annotations
cargo run -- path/to/file.cddl --format zod4  # Zod v4 type annotations
```

## Tips

### Formatting output

The output is generally ugly, so you may need to format it. The easiest way is to pipe it into a formatter.

For example,

```sh
outfile=path/to/file.ts
cargo run -- path/to/file.cddl | prettier --stdin-filepath=$outfile > $outfile
```

## Limitations

1.  Only [`TypeScript`](https://www.typescriptlang.org/) and [`Zod`](https://zod.dev/) (v3 and v4) are supported at the moment.
