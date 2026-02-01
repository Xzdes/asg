# ASG Code Patterns and Idioms

Common patterns and best practices for writing ASG code.

---

## 1. Recursive Functions

### Basic Recursion
```lisp
(fn factorial (n)
  (if (<= n 1)
    1
    (* n (factorial (- n 1)))))
```

### Tail Recursion with Accumulator
```lisp
(fn factorial-tail (n acc)
  (if (<= n 1)
    acc
    (factorial-tail (- n 1) (* acc n))))

(fn factorial (n)
  (factorial-tail n 1))
```

### Fibonacci
```lisp
(fn fib (n)
  (if (<= n 1)
    n
    (+ (fib (- n 1)) (fib (- n 2)))))
```

### Fibonacci with Memoization (using closure)
```lisp
(fn make-fib ()
  (do
    (let cache (dict))
    (fn fib-inner (n)
      (if (dict-has cache n)
        (dict-get cache n)
        (do
          (let result
            (if (<= n 1)
              n
              (+ (fib-inner (- n 1)) (fib-inner (- n 2)))))
          (set cache (dict-set cache n result))
          result)))))

(let fib (make-fib))
```

---

## 2. Higher-Order Functions

### Map-Filter-Reduce Pipeline
```lisp
(|> data
    (filter is-valid)
    (map transform)
    (reduce initial accumulate))
```

### Real Example: Sum of Squares of Even Numbers
```lisp
(|> (range 1 11)
    (filter (lambda (x) (== (% x 2) 0)))
    (map (lambda (x) (* x x)))
    (reduce 0 +))
; => 220 (4 + 16 + 36 + 64 + 100)
```

### Function Composition
```lisp
(let inc (lambda (x) (+ x 1)))
(let double (lambda (x) (* x 2)))
(let inc-then-double (compose double inc))

(inc-then-double 5)  ; => 12
```

### Partial Application
```lisp
(fn add (a b) (+ a b))
(let add5 (lambda (x) (add 5 x)))
(add5 10)  ; => 15
```

---

## 3. Data Processing

### Transform List of Records
```lisp
(let users (array
  (dict "name" "Alice" "age" 30)
  (dict "name" "Bob" "age" 25)
  (dict "name" "Carol" "age" 35)))

; Get names of users over 28
(|> users
    (filter (lambda (u) (> (dict-get u "age") 28)))
    (map (lambda (u) (dict-get u "name"))))
; => ["Alice", "Carol"]
```

### Group and Aggregate
```lisp
; Sum ages by first letter of name
(let users (array
  (dict "name" "Alice" "age" 30)
  (dict "name" "Adam" "age" 25)
  (dict "name" "Bob" "age" 35)))

(reduce users (dict)
  (lambda (acc user)
    (let key (substring (dict-get user "name") 0 1))
    (let current (if (dict-has acc key) (dict-get acc key) 0))
    (dict-set acc key (+ current (dict-get user "age")))))
; => {"A": 55, "B": 35}
```

---

## 4. Closures for State

### Counter
```lisp
(fn make-counter ()
  (do
    (let count 0)
    (lambda ()
      (do
        (set count (+ count 1))
        count))))

(let counter (make-counter))
(counter)  ; => 1
(counter)  ; => 2
(counter)  ; => 3
```

### Stateful Accumulator
```lisp
(fn make-accumulator (init)
  (do
    (let total init)
    (dict
      "add" (lambda (x) (do (set total (+ total x)) total))
      "get" (lambda () total)
      "reset" (lambda () (set total init)))))

(let acc (make-accumulator 0))
((dict-get acc "add") 10)   ; => 10
((dict-get acc "add") 5)    ; => 15
((dict-get acc "get"))      ; => 15
```

---

## 5. Pattern Matching

### Simple Match
```lisp
(fn describe (x)
  (match x
    0 "zero"
    1 "one"
    2 "two"
    _ "many"))
```

### Match with Guards (using nested if)
```lisp
(fn categorize (n)
  (if (< n 0) "negative"
    (if (== n 0) "zero"
      (if (< n 10) "small"
        (if (< n 100) "medium"
          "large")))))
```

### Dispatch Table Pattern
```lisp
(let handlers (dict
  "add" (lambda (a b) (+ a b))
  "sub" (lambda (a b) (- a b))
  "mul" (lambda (a b) (* a b))
  "div" (lambda (a b) (/ a b))))

(fn calculate (op a b)
  (if (dict-has handlers op)
    ((dict-get handlers op) a b)
    (throw (concat "Unknown operation: " op))))

(calculate "add" 10 5)  ; => 15
```

---

## 6. Error Handling

### Try-Catch Pattern
```lisp
(fn safe-divide (a b)
  (try
    (if (== b 0)
      (throw "Division by zero")
      (/ a b))
    (catch e
      (do
        (print (concat "Error: " (error-message e)))
        0))))
```

### Result Pattern (Manual)
```lisp
(fn parse-config (path)
  (if (not (file-exists path))
    (dict "ok" false "error" "File not found")
    (try
      (dict "ok" true "value" (json-decode (read-file path)))
      (catch e
        (dict "ok" false "error" (error-message e))))))

(let result (parse-config "config.json"))
(if (dict-get result "ok")
  (use-config (dict-get result "value"))
  (print (dict-get result "error")))
```

### Validation Chain
```lisp
(fn validate-user (user)
  (do
    (if (not (dict-has user "name"))
      (throw "Missing name"))
    (if (< (str-length (dict-get user "name")) 2)
      (throw "Name too short"))
    (if (not (dict-has user "age"))
      (throw "Missing age"))
    (if (< (dict-get user "age") 0)
      (throw "Invalid age"))
    true))
```

---

## 7. Lazy Evaluation

### Infinite Sequences
```lisp
; Natural numbers
(let naturals (iterate (lambda (x) (+ x 1)) 0))
(collect (take-lazy 10 naturals))
; => [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]

; Fibonacci sequence
(let fibs
  (iterate
    (lambda (pair)
      (array (second pair) (+ (first pair) (second pair))))
    (array 0 1)))

(collect (take-lazy 10 (lazy-map first fibs)))
; => [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
```

### Lazy Pipeline
```lisp
(|> (lazy-range 1 1000000)
    (lazy-filter (lambda (x) (== (% x 2) 0)))
    (lazy-map (lambda (x) (* x x)))
    (take-lazy 10)
    collect)
; Only computes first 10 even squares
```

---

## 8. Module Patterns

### Utility Module
```lisp
(module utils
  (export clamp lerp deg-to-rad)

  (fn clamp (x min max)
    (if (< x min) min
      (if (> x max) max x)))

  (fn lerp (a b t)
    (+ a (* (- b a) t)))

  (let PI 3.14159)
  (fn deg-to-rad (deg)
    (/ (* deg PI) 180)))
```

### Factory Module
```lisp
(module shapes
  (export make-circle make-rect area)

  (fn make-circle (r)
    (dict "type" "circle" "radius" r))

  (fn make-rect (w h)
    (dict "type" "rect" "width" w "height" h))

  (fn area (shape)
    (match (dict-get shape "type")
      "circle" (* PI (* (dict-get shape "radius")
                        (dict-get shape "radius")))
      "rect" (* (dict-get shape "width")
                (dict-get shape "height"))
      _ 0)))
```

---

## 9. Web Patterns

### Simple HTTP Handler
```lisp
(fn handler (req)
  (let path (dict-get req "path"))
  (match path
    "/" (http-response 200 (dict) "Welcome!")
    "/api/data" (http-response 200
                  (dict "Content-Type" "application/json")
                  (json-encode (dict "status" "ok")))
    _ (http-response 404 (dict) "Not Found")))

(http-serve 8080 handler)
```

### JSON API
```lisp
(fn api-handler (req)
  (try
    (do
      (let body (json-decode (dict-get req "body")))
      (let result (process-request body))
      (http-response 200
        (dict "Content-Type" "application/json")
        (json-encode result)))
    (catch e
      (http-response 500
        (dict "Content-Type" "application/json")
        (json-encode (dict "error" (error-message e)))))))
```

---

## 10. Anti-Patterns to Avoid

### Wrong: Using `=` for comparison
```lisp
; WRONG
(if (= x 0) ...)

; CORRECT
(if (== x 0) ...)
```

### Wrong: Forgetting `do` for multiple expressions
```lisp
; WRONG - only last expression runs
(if condition
  (print "a")
  (print "b")
  result)

; CORRECT
(if condition
  (do
    (print "a")
    (print "b")
    result)
  other)
```

### Wrong: Mutating during iteration
```lisp
; WRONG - modifying array while iterating
(for x arr
  (set arr (append arr x)))

; CORRECT - create new array
(let result (reduce arr (array)
  (lambda (acc x) (append acc x))))
```

### Wrong: Deep nesting instead of pipeline
```lisp
; WRONG - hard to read
(reduce (map (filter arr pred) fn) init acc)

; CORRECT - use pipeline
(|> arr (filter pred) (map fn) (reduce init acc))
```

---

## Summary

| Pattern | Use Case |
|---------|----------|
| Recursion | Tree traversal, mathematical functions |
| Pipeline | Data transformation chains |
| Closure | Encapsulated state, factories |
| Match | Dispatch on values |
| Try-Catch | Error recovery |
| Lazy | Large/infinite data, optimization |
| Module | Code organization, reuse |
