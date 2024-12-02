# 🎯 tfocus

> ⚠️ **WARNING**: Resource targeting should be avoided unless absolutely necessary!

## What's this? 🤔

tfocus is a **super interactive** tool for selecting and executing Terraform plan/apply on specific resources.
Think of it as an "emergency tool" - not for everyday use.

![tfocus demo](.github/tfocus_01.gif)

## Features 🌟

- 🔍 Peco-like fuzzy finder for Terraform resources
- ⚡ Lightning-fast resource selection
- 🎨 Colorful TUI (Terminal User Interface)
- 🎹 Vim-like keybindings
- 📁 Recursive file scanning

## Installation 🛠️

install from crates.io
```bash
cargo install tfocus
```

install from github
```bash
cargo install --git https://github.com/nwiizo/tfocus
```

## Usage 🎮

```bash
cd your-terraform-project
tfocus
```

1. 🔍 Launch the fuzzy-search UI
2. ⌨️ Select resources using vim-like keybindings
3. 🎯 Execute plan/apply on selected resources

## Keybindings 🎹

- `↑`/`k`: Move up
- `↓`/`j`: Move down
- `/`: Incremental search
- `Enter`: Select
- `Esc`/`Ctrl+C`: Cancel

## ⚠️ Important Warning ⚠️

Using terraform resource targeting comes with significant risks:

1.  Potential disruption of the Terraform resource graph
2. 🎲 Risk of state inconsistencies
3. 🧩 Possible oversight of critical dependencies
4. 🤖 Deviation from standard Terraform workflow

## When to Use 🎯

Only use this tool in specific circumstances:
- 🚑 Emergency troubleshooting
- 🔧 Development debugging
- 🧪 Testing environment verification
- 📊 Impact assessment of large-scale changes

For regular operations, always use full `terraform plan` and `apply`!

## Appropriate Use Cases 🎭

You might consider using tfocus when:
- 🔥 Working with large Terraform codebases where you need to verify specific changes
- 🐌 Full plan execution takes too long during development
- 🔍 Emergency inspection of specific resource states
- 💣 Staged application of changes in complex infrastructure

**Remember!** Standard `terraform plan` and `apply` are the best practices for normal operations.

## Development Status 🚧

This is an experimental tool. Use at your own risk!

## Example 📺

```bash
$ tfocus
QUERY>

▶    1 [File]     main.tf
     2 [Module]   vpc
     3 [Resource] aws_vpc.main

[↑/k]Up [↓/j]Down [Enter]Select [Esc/Ctrl+C]Cancel
```

## Contributing 🤝

Issues and PRs are welcome! 
Please help make this tool safer and more useful.

## License 📜

MIT

## Final Note 🎬

Think of this tool as a "fire exit" - 
It's there when you need it, but you hope you never have to use it! 😅

---
made with 🦀 and ❤️ by nwiizo
