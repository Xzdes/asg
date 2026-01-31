//! Пример программы на ASG для демонстрации работы с файлами (заглушка).
//!
//! Здесь мы показываем, как можно использовать ASG для имитации чтения и записи файлов.

use asg::asg::{Edge, Node, ASG};
use asg::interpreter::Interpreter;
use asg::nodecodes::{EdgeType, NodeType};

fn main() {
    let mut asg = ASG::new();

    // Создаём узел для имени файла
    let file_name_node = Node::new(
        1,
        NodeType::LiteralString,
        Some("example.txt".as_bytes().to_vec()),
    );

    // Создаём узел для эффекта записи в файл (заглушка)
    let mut write_effect_node = Node::new(2, NodeType::EffectHandle, None);

    // Добавляем связь между именем файла и эффектом записи
    write_effect_node.add_edge(Edge::new(EdgeType::ApplicationArgument, 1));

    // Добавляем узлы в ASG
    asg.add_node(file_name_node);
    asg.add_node(write_effect_node);

    // Запускаем интерпретатор
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&asg, 1);

    match result {
        Ok(val) => println!("Result: {:?}", val),
        Err(e) => eprintln!("Error: {}", e),
    }
}
