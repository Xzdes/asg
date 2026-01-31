//! Пример программы на ASG для демонстрации работы с сетью (заглушка).
//!
//! Здесь мы показываем, как можно использовать ASG для имитации сетевого сервера.

use asg_lang::asg::{Edge, Node, ASG};
use asg_lang::interpreter::Interpreter;
use asg_lang::nodecodes::{EdgeType, NodeType};

fn main() {
    let mut asg = ASG::new();

    // Создаём узел со строкой сервера
    let server_node = Node::new(
        1,
        NodeType::LiteralString,
        Some("HTTP Server running on port 8080".as_bytes().to_vec()),
    );

    // Создаём узел для эффекта (просто LiteralUnit как заглушка)
    let mut effect_node = Node::new(2, NodeType::EffectHandle, None);

    // Добавляем связь между сервером и эффектом
    effect_node.add_edge(Edge::new(EdgeType::ApplicationArgument, 1));

    // Добавляем узлы в ASG
    asg.add_node(server_node);
    asg.add_node(effect_node);

    // Запускаем интерпретатор
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&asg, 1);

    match result {
        Ok(val) => println!("Result: {:?}", val),
        Err(e) => eprintln!("Error: {}", e),
    }
}
