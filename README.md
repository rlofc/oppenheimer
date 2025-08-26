<p align="center">
  <a href="https://github.com/rlofc/oppenheimer/actions/workflows/ci.yml">
      <img src="https://github.com/rlofc/oppenheimer/actions/workflows/ci.yml/badge.svg?branch=master" alt="Github Actions CI Build Status">
  </a>
  <a href="https://crates.io/crates/oppenheimer">
      <img src="https://img.shields.io/crates/v/oppenheimer.svg?style=flat-square" alt="crates.io">
  </a>
  <br>
</p>

# OPPENHEIMER

## Earn your reputation, one task at a time

![OPPENHEIMER](screenshot.png)

**OPPENHEIMER** is a Taskell-inspired hierarchical listboard app, perfect for terminal lovers who hope to make the world a better place.

Whether you're plotting world domination or just trying to get through Monday, this listboard app will help you stay on track - or not.

## Features

- 📋 **Hierarchical Listboards**: Because sometimes simple listboards just don't cut it.
- 🌈 **Lean & Simple**: Less bells, less whistles.
- ⚡ **Fast & Efficient**: Built for the keyboard-oriented mad scientist.
- 🛠️ **Customizable?**: Yes! just tweak the code to suit your style or theme of the day.
- 🚀 **Terminal-based**: For those who live and breathe the terminal.

## Get Started

1. **Clone the Repo**: `git clone https://github.com/rlofc/oppenheimer.git`
2. **Dive In**: Navigate to the directory and run `cargo build --release`
3. **Create Your First List**: Start organizing like a pro! using `cargo run --release my_project.md`

## Key-mapping

| Key Combination                | Action                        |
| ------------------------------ | ----------------------------- |
| `Ctrl + o`                     | Insert list to board          |
| `o`                            | Insert item to current list   |
| `Ctrl + d`                     | Delete a list                 |
| `d`                            | Delete an item                |
| `Down` or `j`                  | Move down                     |
| `Up` or `k`                    | Move up                       |
| `Right` or `l`                 | Move right                    |
| `Left` or `h`                  | Move left                     |
| `Ctrl + Left` or `Ctrl + h`    | Move item to previous list    |
| `Ctrl + Right` or `Ctrl + l`   | Move item to next list        |
| `Ctrl + Down` or `Ctrl + j`    | Deprioritize selected item    |
| `Ctrl + Up` or `Ctrl + k`      | Prioritize selected item      |
| `Shift + Left` or `Shift + h`  | Shuffle list forward          |
| `Shift + Right` or `Shift + l` | Shuffle list back             |
| `Enter`                        | Edit current item             |
| `Space`                        | Toggle current item selection |
| `Tab`                          | Open item sub-board           |
| `Esc`                          | Go back to the previous board |
| `\`                            | Search                        |
| `y`                            | Yank selected item            |
| `x`                            | Cut selected item             |
| `p`                            | Paste item                    |
| `u`                            | Undo action                   |
| `r`                            | Redo action                   |
| `?`                            | Help                          |
| `q`                            | Quit application              |

## Configuring using `config.toml`

Oppenheimer can be customized using a `config.toml` file. Here are the different entities you can configure:

### `BoardConfig`

#### Options

- **`dim_tailing_items`**: This boolean option determines whether to dim trailing items in the board. The default is `false`.

- **`path_separator`**: This setting specifies the separator string used when rendering board paths. By default, it is set to ` 〉 `.

- **`Styles`**: This section allows customization of the visual styles. 

### `Styles`

You can specify the styles for various elements in your configuration file. Each style can have a foreground (`fg`) and a background (`bg`) color.

#### Color Specification

- **Named Colors**: You can use predefined color names like `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `darkgray`, `lightred`, `lightgreen`, `lightyellow`, `lightblue`, `lightmagenta`, `lightcyan`, `lightgray`.
  
- **Hex Colors**: You can define colors using hex codes, e.g., `#FF00FF` for magenta.

- **Indexed Colors**: Oppenheimer supports indexed colors. Use an integer to specify an indexed color.

#### Style Options

- **`header`**: Style for list headers.

- **`active_header`**: Style for the active list header.

- **`item`**: Style for list items.

- **`tag`**: Style for `#tags`.

- **`tag_hashsign`**: Style for the hash sign in tags.

- **`fringe_on`**: Style when an item fringe is set to `on` (when it has a sub-board).

- **`fringe_off`**: Style when the fringe is set to `off`.

- **`selected`**: Style for selected items (usually used just to set the background).

### Defaults

When not using an existing `config.toml` file, Oppenheimer will generate a default one.

## Contribution

Feeling adventurous? Fork the repo and add your magic touch. PRs are always welcome!

## License

OPPENHEIMER is licensed under the BSD License, so feel free to use, modify, and distribute it to your heart's content.
