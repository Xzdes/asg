# ASG Tutorial

A quick guide to learning the ASG programming language.

## Getting Started

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/asg.git
cd asg

# Build
cargo build --release

# Run REPL
cargo run

# Run a file
cargo run -- examples/demo.asg
```

### The REPL

Start the interactive shell:

```
$ cargo run
ASG 0.7.0 - AI-friendly language
Type :help for commands, :quit to exit.

asg> (+ 1 2)
3
asg> (let x 10)
asg> (* x x)
100
```

REPL commands:
- `:help` — show help
- `:quit` — exit
- `:env` — show variables
- `:funcs` — show functions
- `:load file.asg` — load a file
- `:ast expr` — show AST
- `:type expr` — show inferred type

---

## Basic Syntax

ASG uses S-expressions (like Lisp):

```lisp
(operator arg1 arg2 ...)
```

### Literals

```lisp
42          ; Integer
3.14        ; Float
true        ; Boolean
false
"hello"     ; String
()          ; Unit (void)
```

### Arithmetic

```lisp
(+ 1 2)           ; → 3
(- 10 4)          ; → 6
(* 3 4)           ; → 12
(/ 15 3)          ; → 5
(// 17 5)         ; → 3 (integer division)
(% 17 5)          ; → 2 (modulo)

; Variadic (any number of args)
(+ 1 2 3 4 5)     ; → 15
(* 2 3 4)         ; → 24
```

### Comparison

```lisp
(== 1 1)          ; → true
(!= 1 2)          ; → true
(< 1 2)           ; → true
(> 2 1)           ; → true
(<= 5 5)          ; → true
(>= 3 3)          ; → true
```

### Logic

```lisp
(and true false)  ; → false
(or true false)   ; → true
(not true)        ; → false
```

---

## Variables

```lisp
; Declare variable
(let x 42)
(let name "Alice")

; Use variable
(print x)         ; → 42
(+ x 10)          ; → 52

; Mutate variable
(set x 100)
(print x)         ; → 100
```

---

## Conditionals

```lisp
; if-then-else
(if (> x 0)
  "positive"
  "non-positive")

; Nested
(if (== x 0)
  "zero"
  (if (> x 0)
    "positive"
    "negative"))
```

---

## Functions

### Definition

```lisp
; Named function
(fn add (a b)
  (+ a b))

; Call
(add 3 4)         ; → 7

; Lambda (anonymous)
(lambda (x) (* x x))

; Assign lambda
(let square (lambda (x) (* x x)))
(square 5)        ; → 25
```

### Recursion

```lisp
(fn factorial (n)
  (if (<= n 1)
    1
    (* n (factorial (- n 1)))))

(factorial 5)     ; → 120

(fn fib (n)
  (if (<= n 1)
    n
    (+ (fib (- n 1)) (fib (- n 2)))))

(fib 10)          ; → 55
```

### Closures

```lisp
(fn make-counter ()
  (do
    (let count 0)
    (lambda ()
      (do
        (set count (+ count 1))
        count))))

(let counter (make-counter))
(counter)         ; → 1
(counter)         ; → 2
```

---

## Arrays

```lisp
; Create
(let arr (array 1 2 3 4 5))

; Access
(index arr 0)     ; → 1
(first arr)       ; → 1
(last arr)        ; → 5
(nth arr 2)       ; → 3

; Length
(length arr)      ; → 5

; Modify (returns new array)
(append arr 6)    ; → [1, 2, 3, 4, 5, 6]
(set-index arr 0 100)  ; → [100, 2, 3, 4, 5]

; Range
(range 1 5)       ; → [1, 2, 3, 4]
(range 0 10 2)    ; → [0, 2, 4, 6, 8]
```

---

## Higher-Order Functions

```lisp
(let nums (array 1 2 3 4 5))

; Map
(map nums (lambda (x) (* x 2)))
; → [2, 4, 6, 8, 10]

; Filter
(filter nums (lambda (x) (> x 2)))
; → [3, 4, 5]

; Reduce
(reduce nums 0 (lambda (acc x) (+ acc x)))
; → 15

; Combined with pipe
(|> nums
    (filter (lambda (x) (> x 2)))
    (map (lambda (x) (* x 10))))
; → [30, 40, 50]
```

---

## Dictionaries

```lisp
; Create
(let user (dict "name" "Alice" "age" 25))

; Access
(dict-get user "name")     ; → "Alice"

; Modify (returns new dict)
(dict-set user "age" 26)

; Keys and values
(dict-keys user)           ; → ["name", "age"]
(dict-values user)         ; → ["Alice", 25]

; Merge
(dict-merge user (dict "city" "NYC"))
```

---

## Loops

```lisp
; While loop
(let i 0)
(while (< i 5)
  (do
    (print i)
    (set i (+ i 1))))

; Do block (sequence)
(do
  (print "first")
  (print "second")
  (print "third"))
```

---

## Pattern Matching

```lisp
(match value
  (0 "zero")
  (1 "one")
  (_ "other"))

(fn describe (x)
  (match x
    (0 "nothing")
    (1 "single")
    (_ (concat "many: " (str x)))))
```

---

## Error Handling

```lisp
(try
  (/ 10 0)
  (catch e
    (print (concat "Error: " e))
    0))
```

---

## List Comprehensions

```lisp
; [x^2 for x in range if x > 2]
(list-comp (* x x) x (range 1 10) (> x 2))
; → [9, 16, 25, 36, 49, 64, 81]

; Without filter
(list-comp (* x 2) x (array 1 2 3) true)
; → [2, 4, 6]
```

---

## Pipe and Compose

```lisp
; Pipe: thread value through functions
(|> 5
    (lambda (x) (* x 2))
    (lambda (x) (+ x 1)))
; → 11

; Compose: create new function
(let add-and-double
  (compose
    (lambda (x) (+ x 1))
    (lambda (x) (* x 2))))

(add-and-double 5)  ; → 12
```

---

## Interactive Input

```lisp
; Read string
(let name (input "Your name: "))

; Read number
(let age (input-int "Your age: "))
(let weight (input-float "Weight: "))

; Clear screen
(clear-screen)
```

---

## Strings

```lisp
(concat "Hello, " "World!")    ; → "Hello, World!"
(str-length "hello")           ; → 5
(substring "hello" 1 4)        ; → "ell"
(str-split "a,b,c" ",")        ; → ["a", "b", "c"]
(str x)                        ; → convert to string
(parse-int "42")               ; → 42
```

---

## File I/O

```lisp
; Read file
(let content (read-file "data.txt"))

; Write file
(write-file "output.txt" "Hello!")

; Check existence
(if (file-exists "config.json")
  (print "Config found")
  (print "No config"))
```

---

## Math Functions

```lisp
(sqrt 16)        ; → 4.0
(pow 2 10)       ; → 1024.0
(abs -5)         ; → 5
(sin 0)          ; → 0.0
(cos 0)          ; → 1.0
(floor 3.7)      ; → 3
(ceil 3.2)       ; → 4
(log 10)         ; → 2.302...
(exp 1)          ; → 2.718...
(PI)             ; → 3.14159...
```

---

## JSON

```lisp
; Encode
(let data (dict "name" "Alice" "scores" (array 95 87 92)))
(json-encode data)
; → {"name":"Alice","scores":[95,87,92]}

; Decode
(let parsed (json-decode "{\"x\": 1, \"y\": 2}"))
(dict-get parsed "x")  ; → 1
```

---

## HTML Generation

```lisp
(html
  (head
    (title "My Page")
    (style "body { font-family: sans-serif; }"))
  (body
    (h1 "Welcome")
    (p "This is ASG!")
    (div "@class=container"
      (a "@href=https://example.com" "Link"))))
```

---

## HTTP Server

Requires `--features web`:

```lisp
(fn handler (req)
  (do
    (let path (dict-get req "path"))
    (if (== path "/")
      (html (body (h1 "Hello!")))
      (json-encode (dict "error" "Not found")))))

(http-serve 8080 handler)
```

Run with:
```bash
cargo run --features web -- examples/webserver.asg
```

---

## Native GUI

Requires `--features gui`:

```lisp
(let win
  (window "My App" 400 300
    (vbox
      (gui-label "Hello, ASG!")
      (gui-button "Click me" on-click))))

(gui-run win)
```

Run with:
```bash
cargo run --features gui -- examples/calculator_gui.asg
```

---

## Example Programs

### FizzBuzz

```lisp
(let i 1)
(while (<= i 15)
  (do
    (print
      (if (== (% i 15) 0) "FizzBuzz"
        (if (== (% i 3) 0) "Fizz"
          (if (== (% i 5) 0) "Buzz"
            (str i)))))
    (set i (+ i 1))))
```

### Quick Sort

```lisp
(fn quicksort (arr)
  (if (<= (length arr) 1)
    arr
    (do
      (let pivot (first arr))
      (let rest (slice arr 1 (length arr)))
      (let less (filter rest (lambda (x) (< x pivot))))
      (let greater (filter rest (lambda (x) (>= x pivot))))
      (concat (quicksort less)
              (concat (array pivot)
                      (quicksort greater))))))

(print (quicksort (array 3 1 4 1 5 9 2 6)))
```

---

## More Examples

Check out the `examples/` directory:

- `examples/demo.asg` — overview of features
- `examples/fibonacci.asg` — Fibonacci sequences
- `examples/mandelbrot.asg` — ASCII fractal art
- `examples/data_processing.asg` — functional data transformation
- `examples/webserver.asg` — HTTP server with routing
- `examples/todo_app.asg` — interactive todo list
- `examples/calculator_gui.asg` — native GUI calculator

---

## Next Steps

1. Read the [CHANGELOG.md](CHANGELOG.md) for all features
2. Explore `examples/` directory
3. Try building your own project!

Happy coding with ASG! 🚀
