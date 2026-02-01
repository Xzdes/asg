# ASG Built-in Functions Reference

Complete reference of all built-in functions in ASG with signatures, descriptions, and examples.

---

## Arithmetic Operations

### `+` - Addition (variadic)
```lisp
(+ a b)        ; => a + b
(+ a b c ...)  ; => a + b + c + ...
```
**Examples:**
```lisp
(+ 1 2)        ; => 3
(+ 1 2 3 4)    ; => 10
(+ 1.5 2.5)    ; => 4.0
```

### `-` - Subtraction / Negation
```lisp
(- a b)        ; => a - b
(- a)          ; => -a (negation)
```
**Examples:**
```lisp
(- 10 3)       ; => 7
(- 5)          ; => -5
```

### `*` - Multiplication (variadic)
```lisp
(* a b)        ; => a * b
(* a b c ...)  ; => a * b * c * ...
```
**Examples:**
```lisp
(* 3 4)        ; => 12
(* 2 3 4)      ; => 24
```

### `/` - Division
```lisp
(/ a b)        ; => a / b (Float)
```
**Examples:**
```lisp
(/ 10 4)       ; => 2.5
(/ 7 2)        ; => 3.5
```

### `//` - Integer Division
```lisp
(// a b)       ; => floor(a / b)
```
**Examples:**
```lisp
(// 10 3)      ; => 3
(// 7 2)       ; => 3
```

### `%` - Modulo
```lisp
(% a b)        ; => a mod b
```
**Examples:**
```lisp
(% 10 3)       ; => 1
(% 7 2)        ; => 1
```

### `neg` - Negation
```lisp
(neg x)        ; => -x
```

---

## Comparison Operations

### `==` - Equality
```lisp
(== a b)       ; => true if a equals b
```

### `!=` - Inequality
```lisp
(!= a b)       ; => true if a not equals b
```

### `<` - Less Than
```lisp
(< a b)        ; => true if a < b
```

### `<=` - Less Than or Equal
```lisp
(<= a b)       ; => true if a <= b
```

### `>` - Greater Than
```lisp
(> a b)        ; => true if a > b
```

### `>=` - Greater Than or Equal
```lisp
(>= a b)       ; => true if a >= b
```

**Examples:**
```lisp
(== 1 1)       ; => true
(!= 1 2)       ; => true
(< 1 2)        ; => true
(>= 5 5)       ; => true
```

---

## Logical Operations

### `and` / `&&` - Logical AND
```lisp
(and a b)      ; => true if both a and b are true
(and a b c)    ; => true if all are true
```

### `or` / `||` - Logical OR
```lisp
(or a b)       ; => true if a or b is true
```

### `not` / `!` - Logical NOT
```lisp
(not a)        ; => true if a is false
(! a)          ; same
```

**Examples:**
```lisp
(and true true)   ; => true
(and true false)  ; => false
(or false true)   ; => true
(not false)       ; => true
```

---

## Array Operations

### `array` - Create Array
```lisp
(array e1 e2 e3 ...)
```
**Examples:**
```lisp
(array 1 2 3)           ; => [1, 2, 3]
(array "a" "b" "c")     ; => ["a", "b", "c"]
(array)                 ; => []
```

### `index` / `nth` - Get Element
```lisp
(index arr i)           ; => arr[i]
(nth arr i)             ; same
```
**Examples:**
```lisp
(let arr (array 10 20 30))
(index arr 0)           ; => 10
(index arr 2)           ; => 30
```

### `first` / `second` / `third` / `last` - Shortcuts
```lisp
(first arr)             ; => arr[0]
(second arr)            ; => arr[1]
(third arr)             ; => arr[2]
(last arr)              ; => arr[length-1]
```

### `length` - Array Length
```lisp
(length arr)            ; => number of elements
```
**Examples:**
```lisp
(length (array 1 2 3))  ; => 3
(length (array))        ; => 0
```

### `set-index` - Set Element
```lisp
(set-index arr i val)   ; => new array with arr[i] = val
```

### `map` - Transform Elements
```lisp
(map arr fn)            ; => [fn(e1), fn(e2), ...]
```
**Examples:**
```lisp
(map (array 1 2 3) (lambda (x) (* x 2)))
; => [2, 4, 6]
```

### `filter` - Filter Elements
```lisp
(filter arr pred)       ; => elements where pred(e) is true
```
**Examples:**
```lisp
(filter (array 1 2 3 4 5) (lambda (x) (> x 2)))
; => [3, 4, 5]
```

### `reduce` - Fold/Reduce
```lisp
(reduce arr init fn)    ; => fn(fn(fn(init, e1), e2), e3)...
```
**Examples:**
```lisp
(reduce (array 1 2 3 4) 0 (lambda (acc x) (+ acc x)))
; => 10

(reduce (array 1 2 3 4) 1 (lambda (acc x) (* acc x)))
; => 24
```

### `reverse` - Reverse Array
```lisp
(reverse arr)           ; => reversed array
```

### `sort` - Sort Array
```lisp
(sort arr)              ; => sorted array (ascending)
```

### `sum` / `product` - Aggregate
```lisp
(sum arr)               ; => sum of all elements
(product arr)           ; => product of all elements
```

### `contains` - Check Membership
```lisp
(contains arr val)      ; => true if val in arr
```

### `index-of` - Find Index
```lisp
(index-of arr val)      ; => index of val, or -1
```

### `take` / `drop` - Slice
```lisp
(take arr n)            ; => first n elements
(drop arr n)            ; => all except first n
```

### `slice` - Subarray
```lisp
(slice arr start end)   ; => arr[start:end]
```

### `append` - Add Element
```lisp
(append arr val)        ; => arr with val appended
```

### `array-concat` - Concatenate
```lisp
(array-concat arr1 arr2) ; => arr1 ++ arr2
```

### `range` - Create Range
```lisp
(range start end)       ; => [start, start+1, ..., end-1]
```
**Examples:**
```lisp
(range 0 5)             ; => [0, 1, 2, 3, 4]
(range 1 4)             ; => [1, 2, 3]
```

---

## Dictionary Operations

### `dict` - Create Dictionary
```lisp
(dict key1 val1 key2 val2 ...)
```
**Examples:**
```lisp
(dict "name" "Alice" "age" 30)
; => {"name": "Alice", "age": 30}
```

### `dict-get` - Get Value
```lisp
(dict-get d key)        ; => value for key
```

### `dict-set` - Set Value
```lisp
(dict-set d key val)    ; => new dict with key=val
```

### `dict-has` - Check Key
```lisp
(dict-has d key)        ; => true if key exists
```

### `dict-remove` - Remove Key
```lisp
(dict-remove d key)     ; => new dict without key
```

### `dict-keys` - Get Keys
```lisp
(dict-keys d)           ; => array of keys
```

### `dict-values` - Get Values
```lisp
(dict-values d)         ; => array of values
```

### `dict-merge` - Merge Dictionaries
```lisp
(dict-merge d1 d2)      ; => d1 with d2 overlaid
```

### `dict-size` - Count Entries
```lisp
(dict-size d)           ; => number of key-value pairs
```

---

## String Operations

### `concat` - Concatenate
```lisp
(concat s1 s2)          ; => s1 + s2
```
**Examples:**
```lisp
(concat "Hello, " "World!")  ; => "Hello, World!"
```

### `str-length` - String Length
```lisp
(str-length s)          ; => number of characters
```

### `substring` - Extract Substring
```lisp
(substring s start end) ; => s[start:end]
```

### `str-split` - Split String
```lisp
(str-split s delim)     ; => array of parts
```
**Examples:**
```lisp
(str-split "a,b,c" ",") ; => ["a", "b", "c"]
```

### `str-join` - Join Array
```lisp
(str-join arr delim)    ; => joined string
```
**Examples:**
```lisp
(str-join (array "a" "b" "c") "-")  ; => "a-b-c"
```

### `str-contains` - Check Contains
```lisp
(str-contains s sub)    ; => true if s contains sub
```

### `str-replace` - Replace
```lisp
(str-replace s old new) ; => s with old replaced by new
```

### `str-trim` - Trim Whitespace
```lisp
(str-trim s)            ; => s without leading/trailing whitespace
```

### `str-upper` / `str-lower` - Case
```lisp
(str-upper s)           ; => uppercase
(str-lower s)           ; => lowercase
```

### `to-string` / `str` - Convert to String
```lisp
(str val)               ; => string representation
(to-string val)         ; same
```

### `parse-int` / `parse-float` - Parse Numbers
```lisp
(parse-int s)           ; => integer from string
(parse-float s)         ; => float from string
```

---

## Math Functions

### Trigonometric
```lisp
(sin x)    (cos x)    (tan x)
(asin x)   (acos x)   (atan x)
```

### Exponential
```lisp
(exp x)                 ; => e^x
(ln x)                  ; => natural log
(log10 x)               ; => log base 10
(pow x y)               ; => x^y
(sqrt x)                ; => square root
```

### Rounding
```lisp
(abs x)                 ; => |x|
(floor x)               ; => floor
(ceil x)                ; => ceiling
(round x)               ; => round to nearest
```

### Min/Max
```lisp
(min a b)               ; => smaller
(max a b)               ; => larger
```

### Constants
```lisp
PI                      ; => 3.141592653589793
E                       ; => 2.718281828459045
```

---

## I/O Functions

### `print` - Output
```lisp
(print val)             ; print value to stdout
```

### `input` - Read String
```lisp
(input prompt)          ; display prompt, read line
```

### `input-int` / `input-float` - Read Numbers
```lisp
(input-int prompt)      ; read and parse integer
(input-float prompt)    ; read and parse float
```

### `read-file` - Read File
```lisp
(read-file path)        ; => file contents as string
```

### `write-file` - Write File
```lisp
(write-file path content)  ; write content to file
```

### `append-file` - Append to File
```lisp
(append-file path content) ; append content to file
```

### `file-exists` - Check File
```lisp
(file-exists path)      ; => true if file exists
```

### `clear-screen` - Clear Terminal
```lisp
(clear-screen)          ; clear terminal
```

---

## Error Handling

### `try` / `catch` - Handle Errors
```lisp
(try expr (catch var handler))
```
**Examples:**
```lisp
(try
  (/ 1 0)
  (catch e
    (print "Error occurred")
    0))
```

### `throw` - Raise Error
```lisp
(throw message)         ; raise error with message
```

### `is-error` - Check Error
```lisp
(is-error val)          ; => true if val is error
```

### `error-message` - Get Message
```lisp
(error-message err)     ; => error message string
```

---

## Lazy Sequences

### `iterate` - Infinite Iteration
```lisp
(iterate fn init)       ; => [init, fn(init), fn(fn(init)), ...]
```
**Examples:**
```lisp
(take-lazy 5 (iterate (lambda (x) (+ x 1)) 0))
; => [0, 1, 2, 3, 4]
```

### `repeat` - Infinite Repetition
```lisp
(repeat val)            ; => [val, val, val, ...]
```

### `cycle` - Infinite Cycle
```lisp
(cycle arr)             ; => arr repeated infinitely
```

### `lazy-range` - Lazy Range
```lisp
(lazy-range start end)  ; => lazy [start..end)
```

### `take-lazy` - Take from Lazy
```lisp
(take-lazy n seq)       ; => first n elements
```

### `lazy-map` / `lazy-filter`
```lisp
(lazy-map fn seq)       ; lazy map
(lazy-filter pred seq)  ; lazy filter
```

### `collect` - Materialize
```lisp
(collect seq)           ; => array from lazy seq
```

---

## Pipe and Composition

### `|>` / `pipe` - Pipeline
```lisp
(|> val fn1 fn2 fn3)    ; => fn3(fn2(fn1(val)))
(pipe val fn1 fn2)      ; same
```
**Examples:**
```lisp
(|> (array 1 2 3 4 5)
    (filter (lambda (x) (> x 2)))
    (map (lambda (x) (* x 2)))
    (reduce 0 +))
; => 24
```

### `compose` - Function Composition
```lisp
(compose f g)           ; => (lambda (x) (f (g x)))
```

---

## Records

### `record` - Create Record
```lisp
(record field1 val1 field2 val2 ...)
```

### `field` - Get Field
```lisp
(field rec name)        ; => value of field
```

**Examples:**
```lisp
(let person (record name "Alice" age 30))
(field person name)     ; => "Alice"
(field person age)      ; => 30
```

---

## Tensors (ML)

### `tensor` - Create Tensor
```lisp
(tensor shape data)     ; create tensor with shape and data
```

### `tensor-add` / `tensor-mul` - Element-wise
```lisp
(tensor-add t1 t2)      ; element-wise addition
(tensor-mul t1 t2)      ; element-wise multiplication
```

### `tensor-matmul` - Matrix Multiply
```lisp
(tensor-matmul t1 t2)   ; matrix multiplication
```

---

## Web/HTTP

### `http-serve` - Start Server
```lisp
(http-serve port handler)
```
**Examples:**
```lisp
(fn handler (req)
  (http-response 200 (dict) "Hello!"))

(http-serve 8080 handler)
```

### `http-response` - Create Response
```lisp
(http-response status headers body)
```

### `json-encode` / `json-decode`
```lisp
(json-encode val)       ; => JSON string
(json-decode str)       ; => value from JSON
```

---

## GUI (Native)

### `window` - Create Window
```lisp
(window title content)
```

### `gui-button` - Button
```lisp
(gui-button label on-click)
```

### `text-field` - Text Input
```lisp
(text-field placeholder on-change)
```

### `gui-label` - Label
```lisp
(gui-label text)
```

### `vbox` / `hbox` - Layout
```lisp
(vbox child1 child2 ...)  ; vertical
(hbox child1 child2 ...)  ; horizontal
```

### `gui-run` - Run Application
```lisp
(gui-run window)
```

---

## HTML Elements

All HTML elements follow the pattern:
```lisp
(element attrs children...)
```

Available elements:
```
html head body div span p
h1 h2 h3 ul ol li a img
form html-input html-button
table tr td th
style script meta link title
header footer nav main section article
textarea select option label br hr
```

**Examples:**
```lisp
(html (dict)
  (head (dict)
    (title (dict) "My Page"))
  (body (dict)
    (h1 (dict) "Hello!")
    (p (dict "class" "intro") "Welcome to my page.")))
```
