# Tuilade

A simple tool that generates a graphic representation of the i3 environment.

## Installation

Simply clone the repository and compile with `cargo` (or `cargo install`).

## Usage

In order to do some pre-parsing, use the following one-liner, where `<n>` is the
number of the targeted workspace:

```bash
i3-save-tree --workspace=<n> | sed "s/^\(\s*\)\/\/ \"/\1  \"/" | grep -v "^\s*//" | cargo run | dot -Tpng
```

Of course, you can change the `-Tpng` to any of the output formats supported by
`dot`. You can then pipe/save the output from `idot`/`dot` and visualized it
however you want as well.
