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
| `regex` | Regular expressions | `(import "regex")` |
| `validation` | Data validation | `(import "validation")` |
| `file` | Advanced file operations | `(import "file")` |

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

---

## regex.asg

**Regular expressions for text processing.**

```lisp
(import "regex")
```

### Basic Matching

```lisp
(match pattern text)      ; full pattern match, returns bool
(contains pattern text)   ; check if pattern exists in text
```

### Search Operations

```lisp
(find pattern text)       ; first match or nil
(find-all pattern text)   ; all matches as array
(find-pos pattern text)   ; {start, end, match} or nil
```

**Examples:**
```lisp
(find "\\d+" "abc123def")
; => "123"

(find-all "\\d+" "a1b2c3")
; => ["1", "2", "3"]
```

### Replacement

```lisp
(replace-first pattern repl text)  ; replace first match
(replace-all pattern repl text)    ; replace all matches
(replace-with pattern fn text)     ; replace with function result
```

**Examples:**
```lisp
(replace-all "\\d+" "X" "a1b2c3")
; => "aXbXcX"

(replace-with "\\d+" (lambda (x) (str (* 2 (parse-int x)))) "a1b2")
; => "a2b4"
```

### Splitting

```lisp
(split pattern text)      ; split by pattern
(split-n pattern text n)  ; split with limit
```

**Examples:**
```lisp
(split "\\s+" "hello   world")
; => ["hello", "world"]
```

### Capture Groups

```lisp
(groups pattern text)         ; array of captured groups
(named-groups pattern text)   ; dict of named groups
```

**Examples:**
```lisp
(groups "(\\d+)-(\\d+)" "123-456")
; => ["123-456", "123", "456"]

(named-groups "(?P<year>\\d{4})-(?P<month>\\d{2})" "2024-01")
; => {"year": "2024", "month": "01"}
```

### Utilities

```lisp
(escape text)             ; escape special characters
(valid? pattern)          ; check if pattern is valid regex
```

### High-Level Functions

```lisp
(extract-numbers text)    ; extract all numbers
(extract-emails text)     ; extract email addresses
(extract-urls text)       ; extract URLs
(extract-hashtags text)   ; extract #hashtags
(extract-mentions text)   ; extract @mentions
```

### Validation

```lisp
(valid-email? email)      ; validate email format
(valid-url? url)          ; validate URL format
(valid-phone? phone)      ; validate phone (international)
(valid-ipv4? ip)          ; validate IPv4 address
```

### Text Transformations

```lisp
(strip-html text)         ; remove HTML tags
(normalize-whitespace text) ; collapse whitespace
(camel-to-snake text)     ; camelCase -> snake_case
(snake-to-camel text)     ; snake_case -> camelCase
```

### Predefined Patterns

```lisp
PATTERN-EMAIL             ; email regex
PATTERN-URL               ; URL regex
PATTERN-PHONE             ; phone regex
PATTERN-IPV4              ; IPv4 regex
PATTERN-HEX-COLOR         ; #RGB or #RRGGBB
PATTERN-UUID              ; UUID regex
PATTERN-DATE-ISO          ; YYYY-MM-DD
PATTERN-TIME-24H          ; HH:MM:SS
```

### Example: Log Parser

```lisp
(import "regex")

(let log-line "[2024-01-15 14:30:00] ERROR: Connection failed")

; Extract timestamp
(let timestamp (find "\\d{4}-\\d{2}-\\d{2} \\d{2}:\\d{2}:\\d{2}" log-line))
; => "2024-01-15 14:30:00"

; Extract log level
(let level (find "\\] (\\w+):" log-line))
; => "ERROR"

; Parse structured logs
(fn parse-log-line (line)
  (let groups (groups "\\[(.+?)\\] (\\w+): (.+)" line))
  (if groups
    (dict
      "timestamp" (index groups 1)
      "level" (index groups 2)
      "message" (index groups 3))
    nil))
```

---

## validation.asg

**Data validation utilities.**

```lisp
(import "validation")
```

### Type Checks

```lisp
(string? x)       ; is x a string?
(number? x)       ; is x int or float?
(int? x)          ; is x an integer?
(float? x)        ; is x a float?
(array? x)        ; is x an array?
(dict? x)         ; is x a dictionary?
(bool? x)         ; is x a boolean?
(function? x)     ; is x a function?
```

### Emptiness Checks

```lisp
(nil? x)          ; is x nil?
(present? x)      ; is x not nil?
(empty? x)        ; is x empty (string/array/dict)?
(not-empty? x)    ; is x not empty?
```

### Number Validation

```lisp
(in-range? x min max)  ; min <= x <= max
(positive? x)          ; x > 0
(negative? x)          ; x < 0
(zero? x)              ; x == 0
(non-zero? x)          ; x != 0
(integer? x)           ; is integer (int or whole float)
(natural? x)           ; integer and > 0
(non-negative? x)      ; x >= 0
```

### String Validation

```lisp
(min-length? s min)    ; length >= min
(max-length? s max)    ; length <= max
(length-between? s min max)  ; min <= length <= max
(alpha? s)             ; only letters a-zA-Z
(alphanumeric? s)      ; letters and digits
(digits? s)            ; only digits
(identifier? s)        ; valid identifier name
(no-whitespace? s)     ; no spaces/tabs
(blank? s)             ; empty or whitespace only
```

### Format Validation

```lisp
(email? s)        ; valid email format
(url? s)          ; valid URL (http/https)
(phone? s)        ; international phone
(uuid? s)         ; valid UUID
(ipv4? s)         ; valid IPv4 address
(hex-color? s)    ; valid hex color (#RGB or #RRGGBB)
(date-iso? s)     ; YYYY-MM-DD format
(time-24h? s)     ; HH:MM or HH:MM:SS format
```

### Password Strength

```lisp
(password-basic? s)    ; 8+ characters
(password-medium? s)   ; 8+ chars, upper, lower, digit
(password-strong? s)   ; 12+ chars, upper, lower, digit, special
```

### Array Validation

```lisp
(array-not-empty? arr)       ; non-empty array
(array-min-length? arr min)  ; length >= min
(array-max-length? arr max)  ; length <= max
(all? arr pred)              ; all elements match
(any? arr pred)              ; any element matches
(unique? arr)                ; no duplicates
```

### Dictionary Validation

```lisp
(has-key? d key)         ; dict has key
(has-keys? d keys)       ; dict has all keys
(missing-keys d keys)    ; get missing keys
```

### Validator Builder

```lisp
; Create validator from rules
(let validator (make-validator (array
  (dict "field" "email" "check" email? "message" "Invalid email")
  (dict "field" "age" "check" (lambda (x) (in-range? x 18 120)) "message" "Invalid age"))))

; Use validator
(let result (validate validator user-data))
(valid? result)          ; true or false
(get-errors result)      ; array of error dicts
(first-error result)     ; first error or nil
```

### Example

```lisp
(import "validation")

(let user (dict
  "email" "test@example.com"
  "password" "SecurePass123!"
  "age" 25))

; Individual checks
(email? (dict-get user "email"))         ; => true
(password-strong? (dict-get user "password"))  ; => true
(in-range? (dict-get user "age") 18 120)      ; => true

; Build validator
(let validate-user (make-validator (array
  (dict "field" "email" "check" email? "message" "Invalid email")
  (dict "field" "password" "check" password-medium? "message" "Weak password")
  (dict "field" "age" "check" positive? "message" "Age must be positive"))))

(let result (validate validate-user user))
(if (valid? result)
  (print "User is valid!")
  (print (concat "Errors: " (str (get-errors result)))))
```

---

## file.asg

**Advanced file operations.**

```lisp
(import "file")
```

### Reading Files

```lisp
(read path)              ; read file as string
(read-lines path)        ; read as array of lines
(read-json path)         ; read and parse JSON
(read-safe path default) ; read with fallback
(read-head path n)       ; first n lines
(read-tail path n)       ; last n lines
```

### Writing Files

```lisp
(write path content)          ; write string to file
(write-lines path lines)      ; write array of lines
(write-json path data)        ; write JSON
(write-json-pretty path data) ; write formatted JSON
(append-line path line)       ; append single line
(append-lines path lines)     ; append multiple lines
```

### File Checks

```lisp
(exists? path)       ; file/dir exists?
(file? path)         ; is regular file?
(directory? path)    ; is directory?
(empty? path)        ; file size is 0?
(readable? path)     ; can read file?
```

### Metadata

```lisp
(size path)          ; file size in bytes
(size-human path)    ; "1.5 MB", "234 KB", etc.
(extension path)     ; "txt", "json", etc.
(basename path)      ; filename without path
(dirname path)       ; directory part
(name-without-ext path)  ; filename without extension
```

### Path Operations

```lisp
(join-path (array "dir" "subdir" "file.txt"))
; => "dir/subdir/file.txt"

(normalize "a/b/../c/./d")  ; => "a/c/d"
(absolute? "/home/user")    ; => true
(relative? "./file.txt")    ; => true
```

### Directory Operations

```lisp
(list-dir path)          ; list all items
(list-files path)        ; list only files
(list-dirs path)         ; list only directories
(list-by-ext path "txt") ; list files with extension
(list-recursive path)    ; recursive file list
```

### Copy and Move

```lisp
(copy src dest)      ; copy file
(move src dest)      ; move file
(rename old new)     ; rename file
```

### Specialized Formats

```lisp
; CSV
(read-csv path ",")      ; read CSV with separator
(write-csv path data ",")

; Properties (key=value)
(read-properties path)   ; => dict
(write-properties path props)
```

### Temporary Files

```lisp
(with-temp-file "content" (lambda (path)
  (print (concat "Temp file: " path))
  ; use temp file here
  "result"))  ; returns "result", file is deleted
```

### Example: Config Manager

```lisp
(import "file")

; Load config with defaults
(fn load-config (path defaults)
  (if (exists? path)
    (let config (read-json path))
    (dict-merge defaults config)
    defaults))

; Save config
(fn save-config (path config)
  (write-json-pretty path config))

; Usage
(let defaults (dict
  "theme" "dark"
  "language" "en"
  "notifications" true))

(let config (load-config "config.json" defaults))
(print (dict-get config "theme"))

; Update and save
(let new-config (dict-set config "theme" "light"))
(save-config "config.json" new-config)
```

### Example: Log Analyzer

```lisp
(import "file")

; Count lines in log files
(fn analyze-logs (dir)
  (let log-files (list-by-ext dir "log"))
  (map log-files (lambda (f)
    (let path (join-path (array dir f)))
    (let lines (read-lines path))
    (dict
      "file" f
      "lines" (length lines)
      "size" (size-human path)))))

(analyze-logs "/var/log")
```
