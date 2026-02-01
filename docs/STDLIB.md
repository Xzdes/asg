# ASG Standard Library

Complete documentation of the ASG standard library modules.

---

## Overview

| Module | Description | Import |
|--------|-------------|--------|
| `prelude` | Basic functions (auto-imported) | Automatic |
| `functional` | FP combinators & utilities | `(import "functional")` |
| `list` | List/array operations | `(import "list")` |
| `math` | Mathematical functions | `(import "math")` |
| `string` | String utilities | `(import "string")` |
| `io` | I/O helpers | `(import "io")` |
| `json` | JSON manipulation utilities | `(import "json")` |
| `http` | HTTP client functions | `(import "http")` |
| `datetime` | Date and time operations | `(import "datetime")` |
| `testing` | Unit testing framework | `(import "testing")` |

---

## prelude.asg

**Auto-imported into all programs.**

### Constants
| Name | Value | Description |
|------|-------|-------------|
| `true` | `1` | Boolean true |
| `false` | `0` | Boolean false |
| `nil` | `()` | Empty/null value |

### Basic Functions

```lisp
(id x)              ; => x (identity)
(const x y)         ; => x (constant)
(compose f g)       ; => (lambda (x) (f (g x)))
(apply f x)         ; => (f x)
(flip f)            ; => (lambda (x y) (f y x))
```

### Logical Functions

```lisp
(and-fn a b)        ; => (if a b false)
(or-fn a b)         ; => (if a true b)
(not-fn x)          ; => (if x false true)
```

### Conditional Helpers

```lisp
(or-else x default) ; => x if not nil, else default
(when cond body)    ; => body if cond true, else nil
(unless cond body)  ; => body if cond false, else nil
```

### Number Predicates

```lisp
(even? n)           ; => true if n is even
(odd? n)            ; => true if n is odd
(positive? n)       ; => true if n > 0
(negative? n)       ; => true if n < 0
(zero? n)           ; => true if n == 0
```

### Number Operations

```lisp
(inc n)             ; => n + 1
(dec n)             ; => n - 1
(double n)          ; => n * 2
(half n)            ; => n / 2
(square n)          ; => n * n
(cube n)            ; => n * n * n
(sign n)            ; => 1 if n>0, -1 if n<0, 0 if n==0
(clamp x low high)  ; => x bounded to [low, high]
```

---

## functional.asg

**Functional programming utilities.**

```lisp
(import "functional")
```

### Combinators

```lisp
(identity x)        ; => x
(constantly x)      ; => (lambda (y) x)
(compose f g)       ; => (lambda (x) (f (g x)))
(pipe f g)          ; => (lambda (x) (g (f x)))
(compose-all f g h) ; => compose(compose(f, g), h)
(pipe-all f g h)    ; => pipe(pipe(f, g), h)
(flip f)            ; => (lambda (x y) (f y x))
```

### Partial Application

```lisp
(partial f x)       ; => (lambda (...args) (f x ...args))
(partial-right f x) ; => (lambda (...args) (f ...args x))
(partial1 f x)      ; => (lambda (...) (f x ...))
```

### Memoization

```lisp
(memoize f)         ; => cached version of f (single arg)
```

**Example:**
```lisp
(let fib (memoize
  (fn f (n)
    (if (<= n 1) n
      (+ (f (- n 1)) (f (- n 2)))))))
```

### Iteration

```lisp
(iterate-n f init n)      ; apply f to init n times
(iterate-while f pred init) ; apply f while pred is true
(iterate-until f pred init) ; apply f until pred is true
(fix f init)              ; find fixed point
```

### Currying

```lisp
(curry2 f)          ; => (lambda (x) (lambda (y) (f x y)))
(uncurry2 f)        ; => (lambda (x y) ((f x) y))
(curry3 f)          ; => 3-arg currying
```

### Predicate Combinators

```lisp
(pred-and p1 p2)    ; => (lambda (x) (and (p1 x) (p2 x)))
(pred-or p1 p2)     ; => (lambda (x) (or (p1 x) (p2 x)))
(pred-not p)        ; => (lambda (x) (not (p x)))
(all-preds p1 p2 ...) ; all predicates must be true
(any-preds p1 p2 ...) ; any predicate must be true
```

### Maybe/Option

```lisp
(maybe-map f x)     ; => nil if x is nil, else (f x)
(maybe-default def x) ; => def if x is nil, else x
(maybe-chain f x)   ; => nil if x is nil, else (f x)
```

### Advanced

```lisp
(Y f)               ; Y combinator (anonymous recursion)
(trampoline f)      ; trampoline for tail-call optimization
(bounce f ...args)  ; create thunk for trampoline
```

---

## list.asg

**List and array operations.**

```lisp
(import "list")
```

### Basic Operations

```lisp
(head arr)          ; => first element
(tail arr)          ; => all except first
(last arr)          ; => last element
(init arr)          ; => all except last
(empty? arr)        ; => true if length == 0
(singleton x)       ; => (array x)
```

### Transformations

```lisp
(flat-map arr f)    ; => flatten(map(arr, f))
(flatten arr)       ; => flatten nested arrays
(zip arr1 arr2)     ; => [[a1,b1], [a2,b2], ...]
(zip-with f a1 a2)  ; => [f(a1[0],b1[0]), ...]
(unzip pairs)       ; => [[firsts], [seconds]]
(interleave a1 a2)  ; => [a1[0], a2[0], a1[1], ...]
```

**Examples:**
```lisp
(zip (array 1 2 3) (array "a" "b" "c"))
; => [[1, "a"], [2, "b"], [3, "c"]]

(flatten (array (array 1 2) (array 3 4)))
; => [1, 2, 3, 4]
```

### Search

```lisp
(find-index arr pred) ; => index where pred is true, or -1
(find arr pred)     ; => first element where pred is true
(all? arr pred)     ; => true if pred is true for all
(any? arr pred)     ; => true if pred is true for any
(none? arr pred)    ; => true if pred is false for all
```

### Aggregation

```lisp
(minimum arr)       ; => smallest element
(maximum arr)       ; => largest element
(average arr)       ; => mean value
(median arr)        ; => median value
```

### Grouping

```lisp
(chunk arr n)       ; => [[first n], [next n], ...]
(unique arr)        ; => array with duplicates removed
(frequencies arr)   ; => dict of element counts
(group-by arr f)    ; => dict grouped by f(elem)
```

**Examples:**
```lisp
(chunk (range 1 10) 3)
; => [[1,2,3], [4,5,6], [7,8,9]]

(frequencies (array "a" "b" "a" "c" "a"))
; => {"a": 3, "b": 1, "c": 1}

(group-by (array 1 2 3 4 5 6) even?)
; => {true: [2,4,6], false: [1,3,5]}
```

### Sorting

```lisp
(sort-by arr key-fn) ; => sorted by key-fn(elem)
(sort-desc arr)     ; => sorted descending
```

### Utilities

```lisp
(repeat-elem x n)   ; => [x, x, x, ...] n times
(range-step s e step) ; => range with step
(enumerate arr)     ; => [[0, a], [1, b], ...]
(partition arr pred) ; => [matches, non-matches]
(scan arr init f)   ; => running fold results
```

---

## math.asg

**Mathematical functions and constants.**

```lisp
(import "math")
```

### Constants

```lisp
PI                  ; 3.141592653589793
E                   ; 2.718281828459045
TAU                 ; 6.283185307179586 (2*PI)
PHI                 ; 1.618033988749895 (golden ratio)
SQRT2               ; 1.4142135623730951
LN2                 ; 0.6931471805599453
LN10                ; 2.302585092994046
```

### Trigonometric

```lisp
(cot x)             ; cotangent
(sec x)             ; secant
(csc x)             ; cosecant
(deg-to-rad deg)    ; degrees to radians
(rad-to-deg rad)    ; radians to degrees
(sinh x)            ; hyperbolic sine
(cosh x)            ; hyperbolic cosine
(tanh x)            ; hyperbolic tangent
```

### Powers and Roots

```lisp
(square x)          ; x^2
(cube x)            ; x^3
(power x n)         ; x^n (integer n)
(cbrt x)            ; cube root
(nth-root x n)      ; n-th root
```

### Logarithms

```lisp
(log2 x)            ; log base 2
(log-base b x)      ; log base b
```

### Combinatorics

```lisp
(factorial n)       ; n!
(binomial n k)      ; n choose k
(gcd a b)           ; greatest common divisor
(lcm a b)           ; least common multiple
(prime? n)          ; is n prime?
```

### Vectors (as arrays)

```lisp
(dot v1 v2)         ; dot product
(norm v)            ; vector length
(normalize v)       ; unit vector
```

### Interpolation

```lisp
(lerp a b t)        ; linear interpolation
(clamp01 x)         ; clamp to [0, 1]
(smoothstep x)      ; smooth 0-1 transition
```

---

## string.asg

**String manipulation utilities.**

```lisp
(import "string")
```

### Predicates

```lisp
(empty? s)          ; true if length == 0
(not-empty? s)      ; true if length > 0
(blank? s)          ; true if empty or only whitespace
(starts-with? s pre) ; true if s starts with pre
(ends-with? s suf)  ; true if s ends with suf
```

### Transformations

```lisp
(repeat s n)        ; repeat s n times
(reverse-str s)     ; reverse string
(replace-all s old new) ; replace all occurrences
(remove s sub)      ; remove all occurrences of sub
(capitalize s)      ; first char uppercase
(title-case s)      ; each word capitalized
```

### Padding

```lisp
(pad-left s n ch)   ; pad left with ch to length n
(pad-right s n ch)  ; pad right with ch to length n
(center s n ch)     ; center with ch to length n
(truncate s n)      ; truncate to n chars
(truncate-ellipsis s n) ; truncate with "..."
```

### Splitting

```lisp
(lines s)           ; split by newlines
(unlines arr)       ; join with newlines
(words s)           ; split by whitespace
(unwords arr)       ; join with spaces
(chars s)           ; split into characters
(from-chars arr)    ; join characters
```

### Search

```lisp
(index-of s sub)    ; first index of sub, or -1
(last-index-of s sub) ; last index of sub, or -1
(count-occurrences s sub) ; count occurrences
```

---

## io.asg

**Input/Output helpers.**

```lisp
(import "io")
```

### Output

```lisp
(print-inline val)  ; print without newline
(print-all vals)    ; print all values
(print-sep sep vals) ; print with separator
(debug val)         ; print with "DEBUG:" prefix
(assert cond msg)   ; assert condition
```

### Input

```lisp
(input-validated prompt validate default)
; read input, validate, use default if invalid

(input-yes-no prompt)
; ask yes/no question, return bool

(input-int-range prompt min max)
; read int in range [min, max]
```

### Files

```lisp
(read-lines path)   ; read file as array of lines
(write-lines path lines) ; write array of lines
(append-line path line)  ; append single line
```

### Formatting

```lisp
(format template args)  ; simple formatting
(printf template args)  ; print formatted
```

---

## Usage Examples

### Combining Modules

```lisp
(import "functional")
(import "list")
(import "math")

; Find primes using functional composition
(let primes-under-100
  (|> (range 2 100)
      (filter prime?)
      collect))

; Memoized factorial
(let fact (memoize factorial))

; Group numbers by digit count
(let by-digits
  (group-by (range 1 1000)
    (lambda (n) (str-length (str n)))))
```

### Data Processing Pipeline

```lisp
(import "list")
(import "string")

(let users (array
  (dict "name" "Alice" "age" 30 "dept" "eng")
  (dict "name" "Bob" "age" 25 "dept" "sales")
  (dict "name" "Carol" "age" 35 "dept" "eng")))

; Get names of engineers sorted by age
(|> users
    (filter (lambda (u) (== (dict-get u "dept") "eng")))
    (sort-by (lambda (u) (dict-get u "age")))
    (map (lambda (u) (dict-get u "name"))))
; => ["Alice", "Carol"]

; Average age by department
(let by-dept (group-by users (lambda (u) (dict-get u "dept"))))
(map (dict-keys by-dept)
  (lambda (dept)
    (let members (dict-get by-dept dept))
    (array dept (average (map members (lambda (u) (dict-get u "age")))))))
; => [["eng", 32.5], ["sales", 25]]
```

### Mathematical Computation

```lisp
(import "math")

; Golden ratio approximation via Fibonacci ratio
(let fibs (iterate-n
  (lambda (pair)
    (array (second pair) (+ (first pair) (second pair))))
  (array 1 1)
  20))

(let last-pair (last fibs))
(/ (second last-pair) (first last-pair))
; => ~1.618 (approaches PHI)

; Prime factorization
(fn prime-factors (n)
  (if (<= n 1) (array)
    (let d (find (range 2 (+ 1 (floor (sqrt n))))
                 (lambda (x) (== (% n x) 0))))
    (if (== d nil)
      (array n)
      (append (array d) (prime-factors (// n d))))))

(prime-factors 84)  ; => [2, 2, 3, 7]
```

---

## json.asg

**JSON manipulation utilities.**

```lisp
(import "json")
```

### Encoding

```lisp
(pretty value 2)    ; formatted JSON with 2-space indent
(compact value)     ; minified JSON
```

### Parsing

```lisp
(parse-safe str)    ; parse JSON, returns nil on error
(parse-or str def)  ; parse JSON with default value
```

### Navigation

```lisp
(get-path obj (array "user" "name"))  ; nested access
(set-path obj (array "user" "age") 30) ; nested set
(has-path obj (array "user" "email"))  ; check path exists
```

### Transformations

```lisp
(update-key obj "count" inc)      ; update value with function
(rename-key obj "old" "new")      ; rename key
(pick obj (array "name" "age"))   ; select only these keys
(omit obj (array "password"))     ; exclude these keys
```

### Array Operations

```lisp
(find-by arr "id" 123)            ; find object by key value
(filter-by arr "status" "active") ; filter by key value
(pluck arr "name")                ; extract all values of key
(group-by-key arr "category")     ; group by key
(index-by arr "id")               ; create lookup dict
```

### Validation

```lisp
(has-keys obj (array "name" "email"))  ; check required keys
(missing-keys obj (array "a" "b"))     ; get missing keys
```

### Merging

```lisp
(deep-merge obj1 obj2)            ; recursive merge
```

---

## http.asg

**HTTP client functions.**

```lisp
(import "http")
```

### HTTP Methods

```lisp
(get url)                   ; GET request
(get-with-headers url hdrs) ; GET with custom headers
(post url body)             ; POST request
(post-with-headers url hdrs body)
(put url body)              ; PUT request
(delete url)                ; DELETE request
(patch url body)            ; PATCH request
```

### JSON API

```lisp
(get-json url)              ; GET and parse JSON response
(post-json url data)        ; POST JSON, parse response
```

### Response Handling

```lisp
(response-body resp)        ; get response body
(response-status resp)      ; get status code
(response-headers resp)     ; get headers
(is-ok resp)                ; true if 2xx
(is-client-error resp)      ; true if 4xx
(is-server-error resp)      ; true if 5xx
```

### URL Utilities

```lisp
(build-query (dict "q" "search" "page" 1))
; => "q=search&page=1"

(with-query "https://api.example.com" (dict "limit" 10))
; => "https://api.example.com?limit=10"
```

### Headers

```lisp
(auth-header "token123")    ; Bearer token header
(basic-auth-header "user" "pass")  ; Basic auth
(json-headers)              ; Content-Type: application/json
(merge-headers h1 h2)       ; combine headers
```

### Retry Logic

```lisp
(with-retry request-fn 3)   ; retry up to 3 times on error
```

---

## datetime.asg

**Date and time operations.**

```lisp
(import "datetime")
```

### Creating Dates

```lisp
(date 2024 1 15)            ; create date
(time 14 30 0)              ; create time
(datetime 2024 1 15 14 30 0) ; create datetime
(now-timestamp)             ; current Unix timestamp
```

### Parsing

```lisp
(parse-date "2024-01-15")   ; parse ISO date
(parse-time "14:30:00")     ; parse ISO time
(parse-datetime "2024-01-15T14:30:00") ; parse ISO datetime
```

### Formatting

```lisp
(format-date dt)            ; "2024-01-15"
(format-time dt)            ; "14:30:00"
(format-datetime dt)        ; "2024-01-15T14:30:00"
(format-human dt)           ; "January 15, 2024 14:30:00"
```

### Components

```lisp
(year dt) (month dt) (day dt)
(hour dt) (minute dt) (second dt)
(month-name dt)             ; "January"
(day-of-week dt)            ; 0-6 (Sunday = 0)
(day-name dt)               ; "Monday"
```

### Checks

```lisp
(leap-year? 2024)           ; true
(days-in-month 2024 2)      ; 29
(valid-date? dt)            ; check valid
```

### Arithmetic

```lisp
(add-days dt 7)             ; add 7 days
(add-months dt 1)           ; add 1 month
(add-years dt 1)            ; add 1 year
```

### Comparison

```lisp
(compare dt1 dt2)           ; -1, 0, or 1
(before? dt1 dt2)           ; true if dt1 < dt2
(after? dt1 dt2)            ; true if dt1 > dt2
(same-day? dt1 dt2)         ; true if same date
```

---

## testing.asg

**Unit testing framework.**

```lisp
(import "testing")
```

### Test Structure

```lisp
(describe "Math operations" (lambda ()
  (it "adds numbers" (lambda ()
    (assert-eq (+ 1 2) 3)))

  (it "multiplies numbers" (lambda ()
    (assert-eq (* 3 4) 12)))))

(summary)  ; print results
```

### Basic Assertions

```lisp
(assert-eq actual expected)   ; equality
(assert-ne actual expected)   ; inequality
(assert-true value)           ; truthy
(assert-false value)          ; falsy
(assert-nil value)            ; nil check
(assert-not-nil value)        ; not nil
```

### Comparison Assertions

```lisp
(assert-gt a b)               ; a > b
(assert-lt a b)               ; a < b
(assert-gte a b)              ; a >= b
(assert-lte a b)              ; a <= b
```

### Collection Assertions

```lisp
(assert-contains arr value)   ; array contains
(assert-length arr 3)         ; array length
```

### String Assertions

```lisp
(assert-starts-with str "prefix")
```

### Error Assertions

```lisp
(assert-throws (lambda () (throw "error")))
(assert-no-throw (lambda () (+ 1 2)))
```

### Float Assertions

```lisp
(assert-close 3.14 PI 0.01)   ; within epsilon
```

### Results

```lisp
(summary)                     ; print test report
(reset)                       ; reset counters
(get-passed)                  ; passed count
(get-failed)                  ; failed count
```

### Example Test Suite

```lisp
(import "testing")

(describe "Array operations" (lambda ()
  (it "creates arrays" (lambda ()
    (let arr (array 1 2 3))
    (assert-length arr 3)
    (assert-eq (first arr) 1)))

  (it "maps arrays" (lambda ()
    (let result (map (array 1 2 3) (lambda (x) (* x 2))))
    (assert-eq result (array 2 4 6))))

  (it "filters arrays" (lambda ()
    (let result (filter (array 1 2 3 4 5) (lambda (x) (> x 2))))
    (assert-eq (length result) 3)))))

(summary)
```
