# ASG Cheatsheet

Quick reference for ASG programming language.

---

## Literals
```lisp
42              ; Int
3.14            ; Float
"hello"         ; String
true false      ; Bool
()              ; Unit (empty)
```

## Variables
```lisp
(let x 10)              ; declare
(set x 20)              ; mutate
(let (a b c) arr)       ; destructure
```

## Arithmetic
```lisp
(+ 1 2 3)       ; => 6
(- 10 3)        ; => 7
(* 2 3 4)       ; => 24
(/ 10 4)        ; => 2.5
(// 10 3)       ; => 3 (int div)
(% 10 3)        ; => 1 (mod)
```

## Comparison
```lisp
(== a b)  (!= a b)
(< a b)   (<= a b)
(> a b)   (>= a b)
```

## Logic
```lisp
(and a b)   (or a b)   (not a)
```

## Control Flow
```lisp
(if cond then else)
(if (> x 0) "positive" "non-positive")

(do expr1 expr2 ... result)

(while cond body)
(while (< i 10)
  (do (print i) (set i (+ i 1))))

(for x iterable body)
(for x (range 0 5) (print x))

(match val
  0 "zero"
  1 "one"
  _ "other")
```

## Functions
```lisp
(fn name (args) body)
(fn square (x) (* x x))

(lambda (args) body)
(let double (lambda (x) (* x 2)))
```

## Arrays
```lisp
(array 1 2 3)           ; create
(index arr 0)           ; get [0]
(first arr) (last arr)  ; shortcuts
(length arr)            ; size
(map arr fn)            ; transform
(filter arr pred)       ; filter
(reduce arr init fn)    ; fold
(range 0 10)            ; [0..9]
```

## Pipeline
```lisp
(|> data
    (filter pred)
    (map fn)
    (reduce init +))
```

## Dictionaries
```lisp
(dict "key" value ...)
(dict-get d "key")
(dict-set d "key" val)
(dict-keys d)
(dict-values d)
```

## Strings
```lisp
(concat s1 s2)
(str-length s)
(str-split s ",")
(str-join arr "-")
(str-upper s) (str-lower s)
(str x)                 ; to string
```

## Math
```lisp
(sqrt x) (pow x y) (abs x)
(sin x) (cos x) (tan x)
(floor x) (ceil x) (round x)
(min a b) (max a b)
PI  E
```

## I/O
```lisp
(print val)
(input "prompt: ")
(read-file "path")
(write-file "path" content)
```

## Error Handling
```lisp
(try
  risky-expr
  (catch e handler))

(throw "error message")
(is-error val)
```

## Lazy Sequences
```lisp
(iterate fn init)       ; infinite
(repeat val)            ; infinite
(take-lazy n seq)       ; take n
(collect seq)           ; to array
```

## Modules
```lisp
(module name
  (export sym1 sym2)
  definitions...)

(import "path")
(import "path" :as alias)
```

---

## Common Patterns

**Sum array:**
```lisp
(reduce arr 0 +)
```

**Filter and transform:**
```lisp
(|> arr (filter pred) (map fn))
```

**Factorial:**
```lisp
(fn factorial (n)
  (if (<= n 1) 1 (* n (factorial (- n 1)))))
```

**Counter closure:**
```lisp
(fn make-counter ()
  (do
    (let count 0)
    (lambda ()
      (do (set count (+ count 1)) count))))
```

**Read JSON file:**
```lisp
(json-decode (read-file "data.json"))
```

---

## REPL Commands
```
:help           ; show help
:quit           ; exit
:ast <expr>     ; show AST
:type <expr>    ; show type
:env            ; show variables
:funcs          ; show functions
```

---

## Quick Tips

1. Everything is an S-expression: `(operator arg1 arg2)`
2. Use `do` to sequence multiple expressions
3. Use `|>` for readable data pipelines
4. `let` declares, `set` mutates
5. Arrays are 0-indexed
6. Functions are first-class values
