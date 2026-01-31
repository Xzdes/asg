# ASG

**ASG** (Abstract Syntax Graph) — современный язык программирования, построенный на основе абстрактного синтаксического графа.

[![Crates.io](https://img.shields.io/crates/v/asg-lang.svg)](https://crates.io/crates/asg-lang)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

## Особенности

- **ASG-based** — код представляется как граф, а не дерево
- **S-Expression синтаксис** — простой и единообразный
- **Функциональное программирование** — первоклассные функции, замыкания
- **Множественные бэкенды** — интерпретатор, LLVM, WASM
- **LSP поддержка** — полноценная IDE интеграция
- **Менеджер пакетов** — asg-pkg для управления зависимостями

## Быстрый старт

```bash
# Установка
cargo install asg-lang

# Запуск REPL
asg

# Выполнение файла
asg examples/demo.asg

# LSP сервер для IDE
cargo install asg-lsp

# Пакетный менеджер
cargo install asg-pkg
```

## Пример кода

```lisp
; Привет, мир!
(print "Hello, World!")

; Функция
(fn factorial (n)
  (if (<= n 1)
    1
    (* n (factorial (- n 1)))))

(print (factorial 5))  ; => 120

; Массивы и функциональный стиль
(let numbers (array 1 2 3 4 5))
(let doubled (map (lambda (x) (* x 2)) numbers))
(print doubled)  ; => [2, 4, 6, 8, 10]
```

## Компоненты

| Компонент | Описание |
|-----------|----------|
| `asg` | Интерпретатор и компилятор |
| `asg-lsp` | Language Server Protocol |
| `asg-pkg` | Менеджер пакетов |
| [VSCode Extension](https://marketplace.visualstudio.com/) | Расширение для VSCode |

## Сборка из исходников

```bash
git clone https://github.com/Xzdes/asg.git
cd asg
cargo build --release
```

## Документация

- [Туториал](tutorial/)
- [Примеры](examples/)
- [Стандартная библиотека](stdlib/)

## Лицензия

MIT License — см. [LICENSE](LICENSE)
