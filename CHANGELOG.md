# Changelog

All notable changes to ASG will be documented in this file.

## [0.7.0] - 2025-01-30

### Added
- **Interactive Input** — read user input from terminal
  - `(input)` — read string
  - `(input "prompt")` — read string with prompt
  - `(input-int)` / `(input-int "prompt")` — read integer
  - `(input-float)` / `(input-float "prompt")` — read float
  - `(clear-screen)` — clear terminal screen

- **HTML Generation** — generate static web pages
  - `(html ...)`, `(head ...)`, `(body ...)` — document structure
  - `(div ...)`, `(span ...)`, `(p ...)`, `(h1 ...)` - elements
  - `(style ...)`, `(script ...)` — embedded CSS/JS
  - `(html-button ...)`, `(html-input ...)` — form elements
  - Attribute syntax: `"@class=value"`, `"@id=myid"`

- **JSON Support**
  - `(json-encode value)` — convert ASG value to JSON string
  - `(json-decode string)` — parse JSON to ASG value

- **HTTP Server** (requires `--features web`)
  - `(http-serve port handler)` — start HTTP server
  - `(http-response status headers body)` — create response

- **Native GUI** (requires `--features gui`)
  - `(window title width height body)` — create window
  - `(vbox ...)`, `(hbox ...)` — layouts
  - `(gui-button text onclick)` — button widget
  - `(gui-label text)` — text label
  - `(text-field ...)` — input field
  - `(canvas width height ondraw)` — drawing canvas
  - `(gui-run window)` — run GUI event loop

- **New Examples**
  - `examples/mandelbrot.asg` — ASCII Mandelbrot fractal
  - `examples/calculator_tui.asg` — interactive TUI calculator
  - `examples/calculator_web.asg` — HTML calculator generator

### Build Features
- `cargo build --features web` — enable HTTP server
- `cargo build --features gui` — enable native GUI
- `cargo build --features full` — enable all features

## [0.6.0] - 2025-01-26

### Added
- **Dict/HashMap** — immutable dictionaries
  - `(dict "key" value ...)` — create dictionary
  - `(dict-get d key)` — get value by key
  - `(dict-set d key value)` — set value (returns new dict)
  - `(dict-keys d)` — get all keys
  - `(dict-values d)` — get all values
  - `(dict-merge d1 d2)` — merge two dicts

- **Pipe operator** — data pipelines
  - `(|> value fn1 fn2 ...)` — pipe value through functions
  - `(compose fn1 fn2 ...)` — create composed function

- **Destructuring** — unpack arrays and dicts
  - `(let (a b c) array)` — array destructuring
  - `(let (name age) dict)` — dict destructuring

- **List comprehensions**
  - `(list-comp expr var collection predicate)`
  - Example: `(list-comp (* x x) x (range 1 10) (> x 5))`

- **Lazy sequences** — infinite/deferred evaluation
  - `(iterate fn init)` — infinite sequence from function
  - `(repeat value)` — infinite repetition
  - `(cycle array)` — cycle through array infinitely
  - `(lazy-range start end)` — lazy range
  - `(lazy-map fn seq)` — lazy map
  - `(lazy-filter pred seq)` — lazy filter
  - `(take-lazy n seq)` — take n elements
  - `(collect seq)` — materialize lazy sequence

- **Pattern matching**
  - `(match expr (pattern1 result1) (pattern2 result2) ...)`

- **Try/catch error handling**
  - `(try body (catch e handler))`

- **Variadic arithmetic**
  - `(+ 1 2 3 4 5)` — sum any number of values
  - `(* 1 2 3 4 5)` — multiply any number of values

- **Array helper functions**
  - `(nth arr n)` — get nth element (alias for index)
  - `(first arr)` — get first element
  - `(second arr)` — get second element
  - `(third arr)` — get third element
  - `(last arr)` — get last element

### Improved
- Array output now shows `[1, 2, 3]` instead of `[Int(1), Int(2), Int(3)]`
- Better error messages for type mismatches

## [0.5.0] - 2025-01-15

### Added
- **Closures** — functions capture their environment
- **String operations**
  - `(concat s1 s2)` — concatenate strings
  - `(substring s start end)` — extract substring
  - `(str-split s delim)` — split string by delimiter
  - `(str-length s)` — string length

- **File I/O**
  - `(read-file path)` — read file contents
  - `(write-file path content)` — write to file
  - `(file-exists path)` — check if file exists

- **Math standard library**
  - `(sqrt x)` — square root
  - `(sin x)`, `(cos x)`, `(tan x)` — trigonometry
  - `(pow x y)` — power
  - `(abs x)` — absolute value
  - `(PI)` — pi constant
  - `(floor x)`, `(ceil x)` — rounding
  - `(log x)`, `(exp x)` — logarithm and exponential

## [0.4.0] - 2025-01-10

### Added
- **Recursive functions** — proper call stack with CallFrame
- **Higher-order functions**
  - `(map arr fn)` — transform each element
  - `(filter arr pred)` — keep elements matching predicate
  - `(reduce arr init fn)` — fold array to single value

### Fixed
- Function parameters now properly scoped (no variable collision)
- Lambda expressions work correctly in all contexts

## [0.3.0] - 2025-01-05

### Added
- **Integer division**: `(// a b)`
- **Print function**: `(print value)`
- **Array length**: `(length arr)`
- **Array mutation**: `(set-index arr idx value)`

### Changed
- Improved REPL with command history
- Better error messages with source locations

## [0.2.0] - 2024-12-20

### Added
- **LLVM Backend** — compile to native code via inkwell
- **Type Checker** — Hindley-Milner type inference
- **Arrays**
  - `(array 1 2 3)` — create array
  - `(index arr n)` — access element
- **While loops**: `(while cond body)`
- **Do blocks**: `(do expr1 expr2 ...)`
- **Variable mutation**: `(set var value)`

## [0.1.0] - 2024-12-01

### Added
- Initial release
- **S-Expression Parser** — lexer with logos, recursive descent parser
- **ASG (Abstract Syntax Graph)** — nodes, edges, JSON serialization
- **Interpreter** — basic expression evaluation
- **REPL** — interactive shell

#### Supported constructs
- Literals: integers, floats, booleans, strings, unit
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Logic: `and`, `or`, `not`
- Variables: `(let x value)`
- Conditionals: `(if cond then else)`
- Functions: `(fn name (params) body)`
