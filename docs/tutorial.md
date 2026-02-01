# ASG Tutorial

Welcome to ASG - an AI-friendly programming language with S-Expression syntax.

## Installation

```bash
git clone https://github.com/Xzdes/asg.git
cd asg
cargo build --release
```

## Running ASG

### Interactive REPL

```bash
cargo run --bin asg
```

```
ASG REPL v0.1.0
Type :help for commands, :quit to exit

asg> (+ 1 2)
3
asg> (let x 10)
()
asg> (* x x)
100
```

### Execute a File

```bash
cargo run --bin asg -- examples/basics.asg
```

### Execute an Expression

```bash
cargo run --bin asg -- -e "(* 6 7)"
42
```

---

## Language Basics

### Literals

```lisp
42          ; Integer
3.14        ; Float
true        ; Boolean true
false       ; Boolean false
"hello"     ; String
()          ; Unit (empty value)
```

### Arithmetic

All operations use prefix notation (operator first):

```lisp
(+ 10 5)    ; Addition: 15
(- 20 8)    ; Subtraction: 12
(* 3 4)     ; Multiplication: 12
(/ 15 3)    ; Division: 5.0 (float result)
(// 17 5)   ; Integer division: 3
(% 17 5)    ; Modulo: 2
(neg 5)     ; Negation: -5
```

### Nested Expressions

```lisp
(+ (* 2 3) (* 4 5))  ; 2*3 + 4*5 = 26
(/ (+ 10 20) 3)      ; (10 + 20) / 3 = 10
```

---

## Variables

### Declaration

```lisp
(let x 42)           ; Declare x = 42
(let name "Alice")   ; Declare name = "Alice"
```

### Using Variables

```lisp
(let a 10)
(let b 20)
(+ a b)              ; 30
```

### Assignment

```lisp
(let counter 0)
(set counter 1)      ; Update counter to 1
```

---

## Comparison Operators

```lisp
(== 1 1)     ; Equal: true
(!= 1 2)     ; Not equal: true
(< 5 10)     ; Less than: true
(<= 5 5)     ; Less or equal: true
(> 10 5)     ; Greater than: true
(>= 10 10)   ; Greater or equal: true
```

---

## Logical Operators

```lisp
(and true true)   ; Logical AND: true
(and true false)  ; false
(or false true)   ; Logical OR: true
(or false false)  ; false
(not true)        ; Logical NOT: false
(not false)       ; true
```

---

## Conditionals

The `if` expression takes three arguments: condition, then-branch, else-branch.

```lisp
(if (> 10 5)
    "greater"
    "not greater")   ; "greater"

(if (== 1 2)
    100
    0)               ; 0
```

### Example: Absolute Value

```lisp
(let x -42)
(let abs_x (if (< x 0) (neg x) x))
abs_x   ; 42
```

---

## Arrays

### Creating Arrays

```lisp
(array 1 2 3 4 5)           ; Create array [1, 2, 3, 4, 5]
(array "a" "b" "c")         ; Create array ["a", "b", "c"]
```

### Accessing Elements

```lisp
(let numbers (array 10 20 30 40 50))
(index numbers 0)    ; First element: 10
(index numbers 2)    ; Third element: 30
(index numbers 4)    ; Fifth element: 50
```

### Example: Sum of Elements

```lisp
(let arr (array 1 2 3))
(let sum (+ (index arr 0)
            (+ (index arr 1)
               (index arr 2))))
sum   ; 6
```

### Array Length

```lisp
(let arr (array 10 20 30 40 50))
(length arr)    ; 5
```

### Modifying Arrays

```lisp
(let arr (array 1 2 3))
(set-index arr 1 99)    ; arr is now [1, 99, 3]
(index arr 1)           ; 99
```

---

## Output

### Print Function

```lisp
(print "Hello, ASG!")   ; prints: Hello, ASG!
(print 42)                  ; prints: 42
(print (array 1 2 3))       ; prints: [1, 2, 3]
```

---

## Loops

### While Loop

Use `(do ...)` to group multiple statements in the loop body:

```lisp
(let i 1)
(let sum 0)
(while (<= i 5)
  (do
    (set sum (+ sum i))
    (set i (+ i 1))))
sum   ; 1 + 2 + 3 + 4 + 5 = 15
```

### Single Statement Loop

If the body has only one statement, `do` is not needed:

```lisp
(let x 0)
(while (< x 10)
  (set x (+ x 1)))
x   ; 10
```

---

## Functions

### Defining Functions

```lisp
(fn square (x)
  (* x x))

(square 5)      ; => 25
(square 12)     ; => 144
```

### Multiple Parameters

```lisp
(fn add (a b)
  (+ a b))

(fn greet (name)
  (print (concat "Hello, " name "!")))

(add 10 20)     ; => 30
(greet "Alice") ; prints: Hello, Alice!
```

### Recursive Functions

```lisp
(fn factorial (n)
  (if (<= n 1)
    1
    (* n (factorial (- n 1)))))

(factorial 5)   ; => 120
(factorial 10)  ; => 3628800
```

### Lambda (Anonymous Functions)

```lisp
(let double (lambda (x) (* x 2)))
(double 21)     ; => 42

; Use with higher-order functions
(map (array 1 2 3) (lambda (x) (* x x)))
; => [1, 4, 9]
```

### Higher-Order Functions

```lisp
; map - transform each element
(map (array 1 2 3) (lambda (x) (* x 2)))
; => [2, 4, 6]

; filter - keep elements matching predicate
(filter (array 1 2 3 4 5) (lambda (x) (> x 2)))
; => [3, 4, 5]

; reduce - fold array to single value
(reduce (array 1 2 3 4) 0 (lambda (acc x) (+ acc x)))
; => 10
```

### Pipeline Operator

```lisp
(|> (array 1 2 3 4 5)
    (filter (lambda (x) (> x 2)))
    (map (lambda (x) (* x 2)))
    (reduce 0 +))
; => 24 (3*2 + 4*2 + 5*2)
```

---

## Pattern Matching

Use `match` to dispatch on values:

```lisp
(fn describe (n)
  (match n
    0 "zero"
    1 "one"
    2 "two"
    _ "many"))

(describe 0)    ; => "zero"
(describe 1)    ; => "one"
(describe 42)   ; => "many"
```

---

## Error Handling

### Try/Catch

```lisp
(try
  (/ 10 0)
  (catch e
    (print "Error occurred!")
    0))
```

### Throw Errors

```lisp
(fn divide (a b)
  (if (== b 0)
    (throw "Division by zero!")
    (/ a b)))

(try
  (divide 10 0)
  (catch e
    (print (error-message e))
    0))
```

### Check for Errors

```lisp
(let result (try (/ 1 0) (catch e e)))
(if (is-error result)
  (print "Got an error")
  (print result))
```

---

## Dictionaries

### Creating Dictionaries

```lisp
(let person (dict "name" "Alice" "age" 30))
```

### Accessing Values

```lisp
(dict-get person "name")    ; => "Alice"
(dict-get person "age")     ; => 30
```

### Modifying Dictionaries

```lisp
(let updated (dict-set person "age" 31))
(dict-get updated "age")    ; => 31
```

### Dictionary Operations

```lisp
(dict-has person "name")    ; => true
(dict-keys person)          ; => ["name", "age"]
(dict-values person)        ; => ["Alice", 30]
```

---

## Tensors (Machine Learning)

ASG has built-in tensor support for ML operations:

```lisp
(tensor 2.0)              ; Create scalar tensor
(tensor-add t1 t2)        ; Element-wise addition
(tensor-mul t1 t2)        ; Element-wise multiplication
```

---

## Comments

Comments start with `;` and continue to the end of the line:

```lisp
; This is a comment
(+ 1 2)   ; This adds 1 and 2
```

---

## REPL Commands

| Command | Description |
|---------|-------------|
| `:help` | Show help |
| `:quit` or `:q` | Exit REPL |
| `:ast <expr>` | Show AST for expression |
| `:type <expr>` | Show inferred type |
| `:env` | Show defined variables |
| `:funcs` | Show defined functions |
| `:clear` | Clear screen |

---

## Complete Example

Here's a complete program that calculates factorial using a loop:

```lisp
; ASG Example: Factorial Calculator

; Calculate factorial of n
(let n 5)
(let result 1)
(let i 1)

(while (<= i n)
  (do
    (set result (* result i))
    (set i (+ i 1))))

result  ; 120 (5!)
```

---

## Next Steps

### Documentation
- [CHEATSHEET.md](CHEATSHEET.md) - Quick reference card
- [BUILTIN_FUNCTIONS.md](BUILTIN_FUNCTIONS.md) - Complete function reference
- [PATTERNS.md](PATTERNS.md) - Code patterns and idioms
- [STDLIB.md](STDLIB.md) - Standard library documentation
- [LANGUAGE_SPEC.md](LANGUAGE_SPEC.md) - Formal language specification

### Examples
- Explore the examples in `examples/` directory
- Start with `examples/basics.asg` and `examples/demo.asg`

### Advanced Features
- Try the LLVM backend: `cargo build --features llvm_backend`
- Try WASM compilation: `cargo build --features wasm_backend`
- Try formal proofs with Z3: `cargo build --features proofs`

Happy coding with ASG!
