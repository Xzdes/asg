# Getting Started with ASG

Welcome to ASG! This quick guide will walk you through setting up and using ASG in your own Rust project.

---

## 🛠️ Prerequisites

- [Rust](https://www.rust-lang.org/) installed.
- Familiarity with Rust modules and basic programming concepts.

---

## 📦 Installation

Clone the ASG repository:

```bash
git clone https://github.com/yourusername/asg.git
cd asg
````

Build the project:

```bash
cargo build
```

Run tests to ensure everything is working:

```bash
cargo test
```

---

## 🚀 Using ASG in Your Project

To include ASG as a dependency, add it to your `Cargo.toml`:

```toml
[dependencies]
asg-lang = "1.0"
```

Then you can start building your ASG-based programs.

Example:

```rust
use asg_lang::asg::ASG;
use asg_lang::node_factories::{literal_int, binary_operation};
use asg_lang::interpreter::InterpreterContext;

fn main() {
    let mut asg = ASG::new();

    let id1 = 1;
    let id2 = 2;
    let id3 = 3;

    let node1 = literal_int(id1, 5);
    let node2 = literal_int(id2, 8);
    let node3 = binary_operation(id3, "+");

    asg.add_node(node1);
    asg.add_node(node2);
    asg.add_node(node3);

    let interpreter = InterpreterContext;
    interpreter.execute(&asg).unwrap();
}
```

---

## 📚 Learning More

* [Node Types](node_types.md)
* [Edge Types](edge_types.md)

---

## 🤝 Contributions

We welcome contributions! Please fork the repository and create a pull request.

---

## 📜 License

MIT License — see [LICENSE](../LICENSE).