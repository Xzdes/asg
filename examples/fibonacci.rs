use asg_lang::asg::{ASG, Edge, Node};
use asg_lang::nodecodes::{NodeType, EdgeType};
use asg_lang::interpreter::{Interpreter, Value};
fn main() {
println!("--- ASG Example: Executing 5 + 8 ---");
let mut asg = ASG::new();

asg.add_node(Node {
    id: 1,
    node_type: NodeType::LiteralInt,
    payload: Some(5i64.to_le_bytes().to_vec()),
    edges: vec![],
    span: None,
});

asg.add_node(Node {
    id: 2,
    node_type: NodeType::LiteralInt,
    payload: Some(8i64.to_le_bytes().to_vec()),
    edges: vec![],
    span: None,
});

asg.add_node(Node {
    id: 3,
    node_type: NodeType::BinaryOperation,
    payload: None,
    edges: vec![
        Edge { edge_type: EdgeType::ApplicationArgument, target_node_id: 1, payload: None },
        Edge { edge_type: EdgeType::ApplicationArgument, target_node_id: 2, payload: None },
    ],
    span: None,
});

let mut interpreter = Interpreter::new();
// --- ИЗМЕНЕНИЕ: Вызываем `execute` ---
let result = interpreter.execute(&asg, 3);

match result {
    Ok(Value::Int(val)) => {
        println!("\nExecution successful!");
        println!("Result: {}", val);
        assert_eq!(val, 13);
    }
    // --- ИЗМЕНЕНИЕ: Добавляем обработку других возможных значений ---
    Ok(other) => {
        println!("\nExecution finished with unexpected value: {:?}", other);
    }
    Err(e) => {
        eprintln!("\nExecution failed: {}", e);
    }
}
}