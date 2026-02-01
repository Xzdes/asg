# ASG Language Specification v1.0

This document provides a formal specification of the ASG programming language syntax and semantics.

## 1. Lexical Grammar

### 1.1 Tokens

```bnf
<token>     ::= <integer> | <float> | <string> | <ident> | <symbol> | <lparen> | <rparen>

<integer>   ::= ["-"] <digit>+
<float>     ::= ["-"] <digit>+ "." <digit>+
<string>    ::= '"' <string-char>* '"'
<ident>     ::= <ident-start> <ident-cont>*
<symbol>    ::= "'" <ident>

<digit>     ::= "0" | "1" | ... | "9"
<ident-start> ::= <letter> | "_" | "-" | "+" | "*" | "/" | "%" | "<" | ">" | "=" | "!" | "?" | "|"
<ident-cont>  ::= <ident-start> | <digit>
<letter>    ::= "a" .. "z" | "A" .. "Z"

<lparen>    ::= "("
<rparen>    ::= ")"
```

### 1.2 Comments

```bnf
<comment>   ::= ";" <any-char>* <newline>
```

### 1.3 Whitespace

Whitespace (spaces, tabs, newlines) separates tokens and is otherwise ignored.

---

## 2. Expression Grammar

```bnf
<program>   ::= <expr>*

<expr>      ::= <atom> | <list>

<atom>      ::= <integer> | <float> | <string> | <ident>

<list>      ::= "(" <expr>* ")"
```

### 2.1 Special Values

| Literal | Type | Description |
|---------|------|-------------|
| `42` | Int | 64-bit signed integer |
| `3.14` | Float | 64-bit floating point |
| `"hello"` | String | UTF-8 string |
| `true` | Bool | Boolean true |
| `false` | Bool | Boolean false |
| `()` | Unit | Empty list / unit value |

---

## 3. Special Forms

### 3.1 Variables

```lisp
; Declaration
(let <name> <value>)

; Destructuring
(let (<name1> <name2> ...) <array-or-dict>)

; Mutation
(set <name> <value>)
```

**Examples:**
```lisp
(let x 42)
(let (a b c) (array 1 2 3))
(set x 100)
```

### 3.2 Control Flow

```lisp
; Conditional
(if <condition> <then-expr> [<else-expr>])

; Block (sequence)
(do <expr1> <expr2> ... <exprN>)

; Loops
(while <condition> <body>)
(loop <body>)                    ; infinite loop
(for <var> <iterable> <body>)

; Loop control
(break [<value>])
(continue)
(return <value>)
```

**Examples:**
```lisp
(if (> x 0) "positive" "non-positive")

(do
  (print "step 1")
  (print "step 2")
  42)

(while (< i 10)
  (do
    (print i)
    (set i (+ i 1))))

(for x (range 1 5)
  (print x))
```

### 3.3 Functions

```lisp
; Named function
(fn <name> (<param1> <param2> ...) <body>)

; Anonymous function (lambda)
(lambda (<param1> <param2> ...) <body>)
```

**Examples:**
```lisp
(fn square (x) (* x x))

(fn factorial (n)
  (if (<= n 1)
    1
    (* n (factorial (- n 1)))))

(let double (lambda (x) (* x 2)))
```

### 3.4 Pattern Matching

```lisp
(match <value>
  <pattern1> <result1>
  <pattern2> <result2>
  ...
  _ <default-result>)
```

**Patterns:**
- Literal values: `0`, `1`, `"hello"`, `true`
- Wildcard: `_` (matches anything)
- Variable binding: any identifier

**Examples:**
```lisp
(match n
  0 "zero"
  1 "one"
  _ "other")

(match (dict-get config "mode")
  "dev" (setup-dev)
  "prod" (setup-prod)
  _ (error "unknown mode"))
```

### 3.5 Error Handling

```lisp
(try <expr> (catch <var> <handler>))
(throw <message>)
(is-error <value>)
(error-message <error>)
```

**Examples:**
```lisp
(try
  (risky-operation)
  (catch e
    (print (concat "Error: " (error-message e)))
    (default-value)))

(if (< x 0)
  (throw "x must be non-negative"))
```

### 3.6 Modules

```lisp
; Define module
(module <name>
  (export <symbol1> <symbol2> ...)
  <definitions>...)

; Import module
(import <path>)
(import <path> :as <alias>)
(import <path> :only (<symbol1> <symbol2> ...))
```

**Examples:**
```lisp
(module math
  (export square cube PI)
  (let PI 3.14159)
  (fn square (x) (* x x))
  (fn cube (x) (* x x x)))

(import "math")
(import "utils" :as u)
(import "collections" :only (sort filter))
```

---

## 4. Built-in Operators

### 4.1 Arithmetic

| Operator | Syntax | Description |
|----------|--------|-------------|
| `+` | `(+ a b ...)` | Addition (variadic) |
| `-` | `(- a b)` or `(- a)` | Subtraction or negation |
| `*` | `(* a b ...)` | Multiplication (variadic) |
| `/` | `(/ a b)` | Division (returns Float) |
| `//` | `(// a b)` | Integer division |
| `%` | `(% a b)` | Modulo |
| `neg` | `(neg a)` | Negation |

### 4.2 Comparison

| Operator | Syntax | Description |
|----------|--------|-------------|
| `==` | `(== a b)` | Equality |
| `!=` | `(!= a b)` | Inequality |
| `<` | `(< a b)` | Less than |
| `<=` | `(<= a b)` | Less than or equal |
| `>` | `(> a b)` | Greater than |
| `>=` | `(>= a b)` | Greater than or equal |

### 4.3 Logical

| Operator | Syntax | Description |
|----------|--------|-------------|
| `and` / `&&` | `(and a b)` | Logical AND |
| `or` / `\|\|` | `(or a b)` | Logical OR |
| `not` / `!` | `(not a)` | Logical NOT |

### 4.4 Pipe and Composition

| Operator | Syntax | Description |
|----------|--------|-------------|
| `\|>` | `(\|> val f1 f2 ...)` | Pipeline |
| `pipe` | `(pipe val f1 f2 ...)` | Pipeline (alias) |
| `compose` | `(compose f g)` | Function composition |

**Examples:**
```lisp
(|> data
    (filter is-valid)
    (map transform)
    (reduce 0 +))

(let inc-and-double (compose double inc))
```

---

## 5. Built-in Functions

### 5.1 Arrays

| Function | Syntax | Description |
|----------|--------|-------------|
| `array` | `(array e1 e2 ...)` | Create array |
| `index` / `nth` | `(index arr i)` | Get element at index |
| `first` | `(first arr)` | First element |
| `second` | `(second arr)` | Second element |
| `third` | `(third arr)` | Third element |
| `last` | `(last arr)` | Last element |
| `length` | `(length arr)` | Array length |
| `set-index` | `(set-index arr i val)` | Set element |
| `map` | `(map arr fn)` | Map function |
| `filter` | `(filter arr pred)` | Filter by predicate |
| `reduce` | `(reduce arr init fn)` | Fold/reduce |
| `reverse` | `(reverse arr)` | Reverse array |
| `sort` | `(sort arr)` | Sort array |
| `sum` | `(sum arr)` | Sum of elements |
| `product` | `(product arr)` | Product of elements |
| `contains` | `(contains arr val)` | Check membership |
| `index-of` | `(index-of arr val)` | Find index |
| `take` | `(take arr n)` | Take first n |
| `drop` | `(drop arr n)` | Drop first n |
| `slice` | `(slice arr start end)` | Slice array |
| `append` | `(append arr val)` | Append element |
| `array-concat` | `(array-concat a b)` | Concatenate arrays |
| `range` | `(range start end)` | Create range |

### 5.2 Dictionaries

| Function | Syntax | Description |
|----------|--------|-------------|
| `dict` | `(dict k1 v1 k2 v2 ...)` | Create dictionary |
| `dict-get` | `(dict-get d key)` | Get value |
| `dict-set` | `(dict-set d key val)` | Set value |
| `dict-has` | `(dict-has d key)` | Check key exists |
| `dict-remove` | `(dict-remove d key)` | Remove key |
| `dict-keys` | `(dict-keys d)` | Get all keys |
| `dict-values` | `(dict-values d)` | Get all values |
| `dict-merge` | `(dict-merge d1 d2)` | Merge dictionaries |
| `dict-size` | `(dict-size d)` | Number of entries |

### 5.3 Strings

| Function | Syntax | Description |
|----------|--------|-------------|
| `concat` | `(concat s1 s2)` | Concatenate strings |
| `str-length` | `(str-length s)` | String length |
| `substring` | `(substring s start end)` | Extract substring |
| `str-split` | `(str-split s delim)` | Split by delimiter |
| `str-join` | `(str-join arr delim)` | Join with delimiter |
| `str-contains` | `(str-contains s sub)` | Check contains |
| `str-replace` | `(str-replace s old new)` | Replace substring |
| `str-trim` | `(str-trim s)` | Trim whitespace |
| `str-upper` | `(str-upper s)` | To uppercase |
| `str-lower` | `(str-lower s)` | To lowercase |
| `to-string` / `str` | `(str val)` | Convert to string |
| `parse-int` | `(parse-int s)` | Parse integer |
| `parse-float` | `(parse-float s)` | Parse float |

### 5.4 Math

| Function | Syntax | Description |
|----------|--------|-------------|
| `sqrt` | `(sqrt x)` | Square root |
| `sin` | `(sin x)` | Sine |
| `cos` | `(cos x)` | Cosine |
| `tan` | `(tan x)` | Tangent |
| `asin` | `(asin x)` | Arc sine |
| `acos` | `(acos x)` | Arc cosine |
| `atan` | `(atan x)` | Arc tangent |
| `exp` | `(exp x)` | e^x |
| `ln` | `(ln x)` | Natural log |
| `log10` | `(log10 x)` | Log base 10 |
| `pow` | `(pow x y)` | x^y |
| `abs` | `(abs x)` | Absolute value |
| `floor` | `(floor x)` | Floor |
| `ceil` | `(ceil x)` | Ceiling |
| `round` | `(round x)` | Round |
| `min` | `(min a b)` | Minimum |
| `max` | `(max a b)` | Maximum |
| `PI` | `PI` | 3.14159... |
| `E` | `E` | 2.71828... |

### 5.5 I/O

| Function | Syntax | Description |
|----------|--------|-------------|
| `print` | `(print val)` | Print value |
| `input` | `(input prompt)` | Read string input |
| `input-int` | `(input-int prompt)` | Read integer input |
| `input-float` | `(input-float prompt)` | Read float input |
| `read-file` | `(read-file path)` | Read file contents |
| `write-file` | `(write-file path content)` | Write file |
| `append-file` | `(append-file path content)` | Append to file |
| `file-exists` | `(file-exists path)` | Check file exists |
| `clear-screen` | `(clear-screen)` | Clear terminal |

### 5.6 Lazy Sequences

| Function | Syntax | Description |
|----------|--------|-------------|
| `iterate` | `(iterate fn init)` | Infinite iteration |
| `repeat` | `(repeat val)` | Infinite repetition |
| `cycle` | `(cycle arr)` | Infinite cycle |
| `lazy-range` | `(lazy-range start end)` | Lazy range |
| `take-lazy` | `(take-lazy n seq)` | Take from lazy seq |
| `lazy-map` | `(lazy-map fn seq)` | Lazy map |
| `lazy-filter` | `(lazy-filter pred seq)` | Lazy filter |
| `collect` | `(collect seq)` | Materialize lazy seq |

### 5.7 Records

| Function | Syntax | Description |
|----------|--------|-------------|
| `record` | `(record field1 val1 ...)` | Create record |
| `field` | `(field rec name)` | Get field value |

### 5.8 Tensors (ML)

| Function | Syntax | Description |
|----------|--------|-------------|
| `tensor` | `(tensor shape data)` | Create tensor |
| `tensor-add` | `(tensor-add t1 t2)` | Element-wise add |
| `tensor-mul` | `(tensor-mul t1 t2)` | Element-wise multiply |
| `tensor-matmul` | `(tensor-matmul t1 t2)` | Matrix multiply |

### 5.9 Web/HTTP

| Function | Syntax | Description |
|----------|--------|-------------|
| `http-serve` | `(http-serve port handler)` | Start HTTP server |
| `http-response` | `(http-response status headers body)` | Create response |
| `json-encode` | `(json-encode val)` | Encode to JSON |
| `json-decode` | `(json-decode str)` | Decode from JSON |

### 5.10 HTML Elements

```lisp
(html attrs children...)
(head children...)
(body attrs children...)
(div attrs children...)
(span attrs children...)
(p attrs children...)
(h1 attrs children...)
(a attrs children...)
(ul children...)
(li children...)
; ... and more
```

### 5.11 GUI (Native)

| Function | Syntax | Description |
|----------|--------|-------------|
| `window` | `(window title content)` | Create window |
| `gui-button` | `(gui-button label on-click)` | Button widget |
| `text-field` | `(text-field placeholder on-change)` | Text input |
| `gui-label` | `(gui-label text)` | Label widget |
| `vbox` | `(vbox children...)` | Vertical layout |
| `hbox` | `(hbox children...)` | Horizontal layout |
| `gui-run` | `(gui-run window)` | Run GUI |

---

## 6. Type System

ASG uses dynamic typing with the following runtime value types:

| Type | Description | Example |
|------|-------------|---------|
| `Int` | 64-bit signed integer | `42` |
| `Float` | 64-bit floating point | `3.14` |
| `Bool` | Boolean | `true`, `false` |
| `String` | UTF-8 string | `"hello"` |
| `Unit` | Empty value | `()` |
| `Array` | Dynamic array | `(array 1 2 3)` |
| `Dict` | Hash map | `(dict "a" 1)` |
| `Record` | Named fields | `(record name "x")` |
| `Function` | Closure | `(lambda (x) x)` |
| `LazySeq` | Lazy sequence | `(iterate inc 0)` |
| `Error` | Error value | `(throw "error")` |

---

## 7. Scoping Rules

1. **Lexical scoping**: Variables are resolved in the scope where they are defined
2. **Closures**: Functions capture variables from their enclosing scope
3. **Shadowing**: Inner scopes can shadow outer variables
4. **Mutability**: `set` can only modify variables declared with `let`

```lisp
(let x 10)
(let f (lambda ()
  (+ x 1)))   ; captures x
(f)           ; => 11

(let x 20)    ; shadows outer x
(+ x 1)       ; => 21
```

---

## 8. Evaluation Order

1. **Strict evaluation**: Arguments are evaluated before function application
2. **Left-to-right**: Arguments evaluated left to right
3. **Short-circuit**: `and` and `or` short-circuit
4. **Lazy sequences**: Only materialized when collected

---

## 9. Reserved Keywords

```
let set if do while loop for break continue return
fn lambda match try throw catch import module export
true false
```

---

## 10. Standard Library

The standard library is located in `stdlib/` and includes:

- `prelude.asg` - Auto-imported basic functions
- `functional.asg` - Functional programming utilities
- `list.asg` - List operations
- `math.asg` - Mathematical functions
- `string.asg` - String utilities
- `io.asg` - I/O helpers

See `STDLIB.md` for complete documentation.
