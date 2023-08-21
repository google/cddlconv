# cddlconv

A commandline utility for converting CDDL to various formats.

## Usage

 1. Clone this repo and `cd` into it.
 2. `cargo run -- path/to/file.cddl`

## Tips

### Formatting output

The output is generally ugly, so you may need to format it. The easiest way is to pipe it into a formatter.

For example,

```sh
outfile=path/to/file.ts
cargo run -- path/to/file.cddl | prettier --stdin-filepath=$outfile > $outfile
```

## Limitations

 1. Only [`TypeScript`](https://www.typescriptlang.org/) and [`Zod`](https://zod.dev/) is supported at the moment.
