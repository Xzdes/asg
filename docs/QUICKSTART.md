# ASG Quickstart

Get started with ASG in 5 minutes.

---

## 1. Installation

```bash
cargo install asg-lang
```

This installs:
- `asg` - interpreter and REPL
- `asg-lsp` - language server for IDE support
- `asg-pkg` - package manager

---

## 2. Hello World (REPL)

Start the REPL:
```bash
asg
```

Try these:
```lisp
asg> (print "Hello, World!")
Hello, World!

asg> (+ 2 3)
5

asg> (* 6 7)
42
```

---

## 3. Your First Program

Create `hello.asg`:
```lisp
; My first ASG program
(let name "Developer")
(print (concat "Hello, " name "!"))

; Calculate something
(let result (* 6 7))
(print (concat "6 * 7 = " (str result)))
```

Run it:
```bash
asg hello.asg
```

Output:
```
Hello, Developer!
6 * 7 = 42
```

---

## 4. Key Concepts (2 minutes)

### Everything is an S-expression
```lisp
(operator arg1 arg2 ...)
```

### Variables
```lisp
(let x 10)          ; declare
(set x 20)          ; change
(print x)           ; use
```

### Functions
```lisp
(fn greet (name)
  (print (concat "Hello, " name)))

(greet "Alice")     ; => Hello, Alice
```

### Conditionals
```lisp
(if (> x 0)
  "positive"
  "non-positive")
```

### Arrays
```lisp
(let nums (array 1 2 3 4 5))
(print (first nums))           ; => 1
(print (length nums))          ; => 5
```

### Pipeline (the fun part!)
```lisp
(|> (array 1 2 3 4 5)
    (filter (lambda (x) (> x 2)))
    (map (lambda (x) (* x 2)))
    (reduce 0 +))
; => 24
```

---

## 5. Practical Example

**Factorial function:**
```lisp
(fn factorial (n)
  (if (<= n 1)
    1
    (* n (factorial (- n 1)))))

(print (factorial 5))   ; => 120
(print (factorial 10))  ; => 3628800
```

**FizzBuzz:**
```lisp
(for i (range 1 16)
  (print
    (if (== (% i 15) 0) "FizzBuzz"
      (if (== (% i 3) 0) "Fizz"
        (if (== (% i 5) 0) "Buzz"
          i)))))
```

---

## 6. IDE Setup (Optional)

### VS Code
1. Install the ASG extension from `.vsix` file
2. Or search "ASG Language" in marketplace

### Any Editor with LSP
```bash
asg-lsp
```

---

## 7. Next Steps

| Resource | Description |
|----------|-------------|
| [CHEATSHEET.md](CHEATSHEET.md) | Quick reference |
| [tutorial.md](tutorial.md) | Full tutorial |
| [BUILTIN_FUNCTIONS.md](BUILTIN_FUNCTIONS.md) | All functions |
| [examples/](../examples/) | Code examples |
| [stdlib/](../stdlib/) | Standard library |

---

## 8. Quick Reference

```lisp
; Arithmetic
(+ 1 2 3)  (- 10 5)  (* 2 3)  (/ 10 4)

; Comparison
(== a b)  (< a b)  (> a b)

; Logic
(and a b)  (or a b)  (not a)

; Arrays
(array 1 2 3)  (map arr fn)  (filter arr pred)

; Strings
(concat s1 s2)  (str-length s)  (str x)

; I/O
(print val)  (input "prompt")  (read-file path)

; Control
(if cond then else)
(while cond body)
(for x iterable body)
```

---

## Need Help?

```bash
asg --help
```

In REPL:
```
:help
```

Happy coding!
