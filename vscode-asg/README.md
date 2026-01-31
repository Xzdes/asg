# ASG Language Support for VS Code

Full language support for [ASG](https://github.com/Xzdes/asg) — an AI-friendly programming language with S-Expression syntax.

## Features

### Syntax Highlighting
- Full S-expression syntax support
- Keywords, operators, built-in functions
- Strings, numbers, comments
- Rainbow bracket matching

### IntelliSense
- Autocomplete for keywords and built-ins
- Snippets for common patterns
- Hover documentation
- Diagnostics (errors and warnings)

### Code Execution
- Run file: `Ctrl+Shift+R` / `Cmd+Shift+R`
- Run selection: `Ctrl+Enter` / `Cmd+Enter`
- Integrated REPL

### Snippets
| Prefix | Description |
|--------|-------------|
| `fn` | Function definition |
| `lambda` | Anonymous function |
| `let` | Variable binding |
| `if` | Conditional expression |
| `match` | Pattern matching |
| `for` | For loop |
| `pipe` | Pipe operator |
| `map` | Map over array |
| `filter` | Filter array |
| `reduce` | Reduce array |

## Requirements

- **ASG CLI**: Install via `cargo install asg`
- **ASG LSP** (optional): For full IDE support via `cargo install asg-lsp`

```bash
# Install ASG interpreter
cargo install asg

# Install LSP for full IDE support (optional)
cargo install asg-lsp
```

## Extension Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `asg.lsp.enable` | `true` | Enable Language Server |
| `asg.lsp.path` | `asg-lsp` | Path to LSP executable |
| `asg.lsp.trace` | `off` | Trace LSP communication |
| `asg.format.indentSize` | `2` | Indentation size |

## Commands

| Command | Keybinding | Description |
|---------|------------|-------------|
| ASG: Run File | `Ctrl+Shift+R` | Run current file |
| ASG: Run Selection | `Ctrl+Enter` | Run selected code |
| ASG: Start REPL | - | Open ASG REPL |
| ASG: Restart LSP | - | Restart language server |

## Example

```asg
; Factorial with recursion
(fn factorial (n)
  (if (<= n 1)
      1
      (* n (factorial (- n 1)))))

(print (factorial 10))  ; => 3628800

; Functional pipeline
(|> (range 1 100)
    (filter even?)
    (map square)
    (take 10)
    sum)
```

## Release Notes

### 1.0.0
- Initial release
- Syntax highlighting
- LSP integration
- Snippets
- Run commands

## License

MIT License - see [LICENSE](LICENSE)
