# Tuilade

A simple tool that generates a graphic representation of the i3 environment.

## Installation

Simply clone the repository and compile with `cargo` (or `cargo install`).

## Usage

You'll need to extract the i3 tree and pipe it to Tuilade:

```bash
i3-msg -t get_tree | cargo run | dot -Tpng
```

Of course, you can change the `-Tpng` to any of the output formats supported by
`dot`. You can then pipe/save the output from `idot`/`dot` and visualized it
however you want as well.

### Display in a floating window

First, install `tuilade` to your path by running `cargo install` in `tuilade`'s
directory:

```bash
cargo install --path .
```

Then you can create a small script to simplify editing of the command in the
[Usage](#usage) section, for example in your `.config/i3` directory, possibly
running `tuilade` in `silent` mode:

```sh
#!/bin/sh

i3-msg -t get_tree | $HOME/.cargo/bin/tuilade -s | dot -Tpng | feh --class fehi3tuilade -
```

Finally, you can add a binding to show the tree (in this example, `$mod+t`) and
set the window to floating mode:

```
for_window [class="^fehi3tuilade$"] floating enable

bindsym $mod+t exec $HOME/.config/i3/tuilade.sh
```

### Available options

```
Usage: tuilade [OPTIONS]

Options:
  -s, --silent   If enabled, will hide empty sections at best
  -h, --help     Print help
  -V, --version  Print version
```

#### Silent mode

Silent mode (`-s` or `--silent`) will try to hide empty or default sections.

