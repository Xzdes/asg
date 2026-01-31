# ASG Tutorial

Welcome to the ASG programming language tutorial! This guide covers real-world examples that demonstrate ASG's capabilities.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Web Server](#web-server)
3. [CLI Tool](#cli-tool)
4. [Data Processing](#data-processing)
5. [Functional Programming](#functional-programming)

---

## Getting Started

### Installation

```bash
git clone https://github.com/Xzdes/asg.git
cd asg
cargo build --release
```

### Hello World

Create `hello.asg`:

```lisp
(print "Hello, ASG!")
```

Run it:

```bash
cargo run --bin asg -- hello.asg
```

### Basic Syntax

```lisp
; Variables
(let name "World")
(let answer 42)
(let pi 3.14159)

; Functions
(fn greet (name)
  (print (concat "Hello, " name "!")))

(greet "ASG")  ; => Hello, ASG!

; Control flow
(fn abs (x)
  (if (< x 0)
      (- 0 x)
      x))

; Lists
(let numbers (array 1 2 3 4 5))
(let doubled (map numbers (lambda (x) (* x 2))))
(print doubled)  ; => [2, 4, 6, 8, 10]
```

---

## Web Server

Build a simple HTTP server that responds to requests.

### File: [01-web-server.asg](01-web-server.asg)

```lisp
; Simple HTTP server example
; Requires: cargo build --features web

(import "std/io")

; Request handler
(fn handle-request (method path)
  (match path
    ("/" (html-response "Welcome to ASG!"))
    ("/api/hello" (json-response {"message" "Hello from ASG!"}))
    ("/api/time" (json-response {"time" (current-time)}))
    (_ (not-found-response))))

; HTML response builder
(fn html-response (content)
  (dict
    "status" 200
    "headers" (dict "Content-Type" "text/html")
    "body" (concat "<html><body><h1>" content "</h1></body></html>")))

; JSON response builder
(fn json-response (data)
  (dict
    "status" 200
    "headers" (dict "Content-Type" "application/json")
    "body" (json-encode data)))

; 404 handler
(fn not-found-response ()
  (dict
    "status" 404
    "headers" (dict "Content-Type" "text/plain")
    "body" "Not Found"))

; Start server (conceptual - actual implementation uses tiny_http)
(print "Starting server on http://localhost:8080")
; (start-server 8080 handle-request)
```

### Key Concepts:
- Pattern matching for routing
- Dict for structured data
- JSON encoding
- First-class functions

---

## CLI Tool

Create a command-line utility for text processing.

### File: [02-cli-tool.asg](02-cli-tool.asg)

```lisp
; CLI tool: Word frequency counter
; Usage: asg 02-cli-tool.asg <filename>

(import "std/io")
(import "std/string")
(import "std/list")

; Main entry point
(fn main (args)
  (if (< (length args) 1)
      (do
        (print "Usage: asg wordcount.asg <filename>")
        (exit 1))
      (let filename (get args 0))
      (process-file filename)))

; Process a file and count word frequencies
(fn process-file (filename)
  (let content (read-file filename))
  (let words (tokenize content))
  (let freq (count-frequencies words))
  (print-top-words freq 10))

; Tokenize text into words
(fn tokenize (text)
  (|> text
      (str-lower)
      (str-replace "[^a-z ]" "")
      (str-split " ")
      (filter (lambda (w) (> (str-length w) 0)))))

; Count word frequencies
(fn count-frequencies (words)
  (reduce words (dict)
    (lambda (acc word)
      (let current (dict-get acc word 0))
      (dict-set acc word (+ current 1)))))

; Print top N words
(fn print-top-words (freq n)
  (let pairs (dict-to-list freq))
  (let sorted (sort-by pairs (lambda (p) (- 0 (get p 1)))))
  (let top (take sorted n))
  (print "Top words:")
  (for-each top (lambda (p)
    (print (concat "  " (get p 0) ": " (str (get p 1)))))))

; Run main with command line arguments
(main (get-args))
```

### Key Concepts:
- Command-line argument handling
- File I/O operations
- String manipulation
- Higher-order functions (reduce, filter, map)
- Pipe operator for data transformation

---

## Data Processing

Work with structured data: parsing, transforming, and outputting.

### File: [03-data-processing.asg](03-data-processing.asg)

```lisp
; Data processing: CSV to JSON converter

(import "std/io")
(import "std/string")
(import "std/list")

; Parse CSV content
(fn parse-csv (content)
  (let lines (str-split content "\n"))
  (let header (str-split (get lines 0) ","))
  (let data-lines (rest lines))
  (map data-lines (lambda (line)
    (let values (str-split line ","))
    (zip-to-dict header values))))

; Create dict from two lists
(fn zip-to-dict (keys values)
  (reduce (zip keys values) (dict)
    (lambda (acc pair)
      (dict-set acc (get pair 0) (get pair 1)))))

; Transform data: add computed fields
(fn enrich-data (records)
  (map records (lambda (record)
    (let price (parse-float (dict-get record "price" "0")))
    (let quantity (parse-int (dict-get record "quantity" "0")))
    (dict-set record "total" (* price quantity)))))

; Filter records by predicate
(fn filter-expensive (records threshold)
  (filter records (lambda (r)
    (> (dict-get r "total" 0) threshold))))

; Aggregate data
(fn summarize (records)
  (dict
    "count" (length records)
    "total" (sum (map records (lambda (r) (dict-get r "total" 0))))
    "average" (/ (sum (map records (lambda (r) (dict-get r "total" 0))))
                 (length records))))

; Main pipeline
(fn process-sales (filename)
  (let content (read-file filename))
  (let records (parse-csv content))
  (let enriched (enrich-data records))
  (let expensive (filter-expensive enriched 100))
  (let summary (summarize expensive))

  (print "=== Sales Report ===")
  (print (concat "Total records: " (str (dict-get summary "count"))))
  (print (concat "Total value: $" (str (dict-get summary "total"))))
  (print (concat "Average: $" (str (dict-get summary "average"))))

  ; Write to JSON
  (write-file "report.json" (json-encode summary)))

; Example with inline data
(fn demo ()
  (let sample-csv "product,price,quantity
Widget,10.50,5
Gadget,25.00,3
Gizmo,15.75,8
Device,50.00,2")

  (let records (parse-csv sample-csv))
  (let enriched (enrich-data records))

  (print "Enriched data:")
  (for-each enriched (lambda (r)
    (print (json-encode r)))))

(demo)
```

### Key Concepts:
- CSV parsing
- Data transformation pipelines
- Dict operations
- Aggregation functions
- File output

---

## Functional Programming

Advanced functional programming patterns in ASG.

### File: [04-functional.asg](04-functional.asg)

```lisp
; Functional programming patterns

(import "std/functional")

; === Currying ===

; Curried add function
(let add (curry2 (lambda (x y) (+ x y))))
(let add5 (add 5))
(print (add5 10))  ; => 15

; === Composition ===

; Compose functions right-to-left
(let double (lambda (x) (* x 2)))
(let square (lambda (x) (* x x)))
(let double-then-square (compose square double))

(print (double-then-square 3))  ; => 36 (3*2 = 6, 6*6 = 36)

; === Memoization ===

; Expensive computation (simulated)
(fn fib-slow (n)
  (if (<= n 1)
      n
      (+ (fib-slow (- n 1)) (fib-slow (- n 2)))))

; Memoized version
(let fib (memoize fib-slow))

(print (fib 35))  ; Fast with memoization!

; === Lazy Sequences ===

; Infinite sequence of natural numbers
(fn naturals-from (n)
  (lazy-seq n (naturals-from (+ n 1))))

(let naturals (naturals-from 0))

; Take first 10 even numbers
(let evens (filter naturals even?))
(print (take evens 10))  ; => [0, 2, 4, 6, 8, 10, 12, 14, 16, 18]

; === Monadic Operations ===

; Maybe monad for null safety
(fn safe-div (a b)
  (if (= b 0)
      nil
      (/ a b)))

(fn calculate (x y z)
  (|> (safe-div x y)
      (maybe-chain (lambda (r1) (safe-div r1 z)))
      (maybe-default 0)))

(print (calculate 100 5 2))   ; => 10
(print (calculate 100 0 2))   ; => 0 (division by zero handled)

; === Pattern Matching with Guards ===

(fn classify-number (n)
  (match n
    ((? negative?) "negative")
    (0 "zero")
    ((? (lambda (x) (< x 10))) "small positive")
    ((? (lambda (x) (< x 100))) "medium positive")
    (_ "large positive")))

(print (classify-number -5))   ; => negative
(print (classify-number 0))    ; => zero
(print (classify-number 7))    ; => small positive
(print (classify-number 42))   ; => medium positive
(print (classify-number 999))  ; => large positive

; === Transducers ===

; Efficient composition of transformations
(fn xf-map (f)
  (lambda (rf)
    (lambda (acc item)
      (rf acc (f item)))))

(fn xf-filter (pred)
  (lambda (rf)
    (lambda (acc item)
      (if (pred item)
          (rf acc item)
          acc))))

; Compose transducers
(let xf (compose
          (xf-filter even?)
          (xf-map (lambda (x) (* x 2)))))

; Apply transducer
(let result (transduce xf conj [] (range 10)))
(print result)  ; => [0, 4, 8, 12, 16]
```

### Key Concepts:
- Currying and partial application
- Function composition
- Memoization
- Lazy evaluation
- Monadic error handling
- Pattern matching with guards
- Transducers

---

## Next Steps

1. **Explore the Standard Library**: Check out `stdlib/` for more functions
2. **Read the API Reference**: See the full list of built-in functions
3. **Join the Community**: Report issues on GitHub
4. **Contribute**: PRs welcome!

## Resources

- [GitHub Repository](https://github.com/Xzdes/asg)
- [API Documentation](../docs/api.md)
- [Language Specification](../docs/spec.md)
