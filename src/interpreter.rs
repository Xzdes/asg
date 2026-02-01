//! Простой, рекурсивный интерпретатор ASG.
//!
//! Поддерживает выполнение программ, представленных в виде ASG.

use std::collections::HashMap;
use std::fs;
use std::io::Write;

use crate::asg::{Node, NodeID, ASG};
use crate::error::{ASGError, ASGResult};
use crate::nodecodes::{EdgeType, NodeType};
use crate::ops::tensor_ops;
use crate::parser::parse;
use crate::runtime::diff_tensor::DifferentiableTensor;

/// Представление рантайм-значений в ASG.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Целое число
    Int(i64),
    /// Число с плавающей точкой
    Float(f64),
    /// Булево значение
    Bool(bool),
    /// Строка
    String(String),
    /// Unit (отсутствие значения)
    Unit,
    /// Тензор (для ML операций)
    Tensor(DifferentiableTensor),
    /// Функция/Closure (хранит ID узла тела и захваченные переменные)
    Function {
        params: Vec<String>,
        body_id: NodeID,
        /// Захваченные переменные из внешнего scope (для closures)
        captured: HashMap<String, Value>,
    },
    /// Запись (структура)
    Record(HashMap<String, Value>),
    /// Массив
    Array(Vec<Value>),
    /// Ошибка (для try/catch)
    Error(String),
    /// Словарь (ключ -> значение)
    Dict(HashMap<String, Value>),
    /// Скомпонованные функции (compose f g h) = (lambda (x) (h (g (f x))))
    ComposedFunction(Vec<Value>),
    /// Ленивая последовательность
    LazySeq(Box<LazySeqKind>),
}

/// Виды ленивых последовательностей
#[derive(Debug, Clone, PartialEq)]
pub enum LazySeqKind {
    /// Итерация: [init, f(init), f(f(init)), ...]
    Iterate {
        func: Box<Value>,
        current: Box<Value>,
    },
    /// Повторение: [val, val, val, ...]
    Repeat(Box<Value>),
    /// Цикл по массиву: [a,b,c,a,b,c,...]
    Cycle { arr: Vec<Value>, index: usize },
    /// Ленивый range
    Range { current: i64, end: i64, step: i64 },
    /// Ленивый map
    Map {
        func: Box<Value>,
        source: Box<LazySeqKind>,
    },
    /// Ленивый filter
    Filter {
        func: Box<Value>,
        source: Box<LazySeqKind>,
    },
}

impl Value {
    /// Получить целое число из значения.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// Получить float из значения.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// Получить bool из значения.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Форматировать значение для вывода (человекочитаемый формат).
    pub fn format_display(&self) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => format!("\"{}\"", s),
            Value::Unit => "()".to_string(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.format_display()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Record(fields) => {
                let items: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.format_display()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            Value::Dict(dict) => {
                let items: Vec<String> = dict
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.format_display()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            Value::Function { params, .. } => format!("<fn({})>", params.join(", ")),
            Value::ComposedFunction(fns) => format!("<composed({})>", fns.len()),
            Value::Tensor(t) => format!("<tensor {:?}>", t.data.borrow().shape()),
            Value::Error(msg) => format!("<error: {}>", msg),
            Value::LazySeq(_) => "<lazy-seq>".to_string(),
        }
    }
}

/// Фрейм вызова для рекурсии.
/// Хранит локальные переменные, параметры и memo для этого вызова.
#[derive(Debug, Clone, Default)]
struct CallFrame {
    /// Локальные переменные этого вызова
    locals: HashMap<String, Value>,
    /// Memo для этого вызова (кэш узлов тела функции)
    memo: HashMap<NodeID, Value>,
}

/// Контекст выполнения, хранит вычисленные значения для каждого узла.
pub struct Interpreter {
    /// Кэш вычисленных значений узлов
    memo: HashMap<NodeID, Value>,
    /// Глобальные переменные
    variables: HashMap<String, Value>,
    /// Функции: имя -> (параметры, body_id, опциональный ASG для импортированных функций)
    functions: HashMap<String, (Vec<String>, NodeID, Option<ASG>)>,
    /// Стек вызовов для рекурсии
    call_stack: Vec<CallFrame>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self {
            memo: HashMap::new(),
            variables: HashMap::new(),
            functions: HashMap::new(),
            call_stack: Vec::new(),
        }
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Разрешает переменную с приоритетом стека вызовов.
    /// Сначала проверяет локальные переменные в call_stack (от вершины к основанию),
    /// затем глобальные переменные.
    fn resolve_variable(&self, name: &str) -> Option<&Value> {
        // Сначала проверяем стек вызовов (от вершины к основанию)
        for frame in self.call_stack.iter().rev() {
            if let Some(val) = frame.locals.get(name) {
                return Some(val);
            }
        }
        // Fallback на глобальные переменные
        self.variables.get(name)
    }

    /// Выполняет ASG, вычисляя узлы по требованию начиная с корневого.
    pub fn execute(&mut self, asg: &ASG, root_id: NodeID) -> ASGResult<Value> {
        // Оцениваем только корневой узел, остальные по требованию
        self.ensure_evaluated(asg, root_id)
    }

    /// Вычисляет значение для одного узла и сохраняет его в кэш.
    fn eval_node(&mut self, asg: &ASG, node: &Node) -> ASGResult<()> {
        if self.memo.contains_key(&node.id) {
            return Ok(());
        }

        let result_value = match node.node_type {
            // === Литералы ===
            NodeType::LiteralInt => {
                let payload = node
                    .payload
                    .as_ref()
                    .ok_or(ASGError::MissingPayload(node.id))?;
                let bytes: [u8; 8] = payload
                    .clone()
                    .try_into()
                    .map_err(|_| ASGError::InvalidPayload(node.id))?;
                Value::Int(i64::from_le_bytes(bytes))
            }

            NodeType::LiteralFloat => {
                let payload = node
                    .payload
                    .as_ref()
                    .ok_or(ASGError::MissingPayload(node.id))?;
                let bytes: [u8; 8] = payload
                    .clone()
                    .try_into()
                    .map_err(|_| ASGError::InvalidPayload(node.id))?;
                Value::Float(f64::from_le_bytes(bytes))
            }

            NodeType::LiteralBool => {
                let payload = node
                    .payload
                    .as_ref()
                    .ok_or(ASGError::MissingPayload(node.id))?;
                let val = payload.first().map(|&b| b != 0).unwrap_or(false);
                Value::Bool(val)
            }

            NodeType::LiteralString => {
                let payload = node
                    .payload
                    .as_ref()
                    .ok_or(ASGError::MissingPayload(node.id))?;
                let s = String::from_utf8(payload.clone())
                    .map_err(|_| ASGError::InvalidPayload(node.id))?;
                Value::String(s)
            }

            NodeType::LiteralUnit => Value::Unit,

            NodeType::LiteralTensor => {
                let payload = node
                    .payload
                    .as_ref()
                    .ok_or(ASGError::MissingPayload(node.id))?;
                let bytes: [u8; 4] = payload
                    .clone()
                    .try_into()
                    .map_err(|_| ASGError::InvalidPayload(node.id))?;
                let val = f32::from_le_bytes(bytes);
                let tensor = DifferentiableTensor::new(ndarray::arr0(val).into_dyn(), true);
                Value::Tensor(tensor)
            }

            // === Арифметические операции ===
            NodeType::BinaryOperation => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
                    (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
                    (Value::Int(a), Value::Float(b)) => Value::Float((a as f64) + b),
                    (Value::Float(a), Value::Int(b)) => Value::Float(a + (b as f64)),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected two numbers for BinaryOperation".to_string(),
                        ))
                    }
                }
            }

            NodeType::Sub => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
                    (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                    (Value::Int(a), Value::Float(b)) => Value::Float((a as f64) - b),
                    (Value::Float(a), Value::Int(b)) => Value::Float(a - (b as f64)),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected two numbers for Sub".to_string(),
                        ))
                    }
                }
            }

            NodeType::Mul => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
                    (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
                    (Value::Int(a), Value::Float(b)) => Value::Float((a as f64) * b),
                    (Value::Float(a), Value::Int(b)) => Value::Float(a * (b as f64)),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected two numbers for Mul".to_string(),
                        ))
                    }
                }
            }

            NodeType::Div => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::Int(a), Value::Int(b)) => {
                        if b == 0 {
                            return Err(ASGError::InvalidOperation("Division by zero".to_string()));
                        }
                        // True division returns float
                        Value::Float(a as f64 / b as f64)
                    }
                    (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
                    (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 / b),
                    (Value::Float(a), Value::Int(b)) => Value::Float(a / b as f64),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected two numbers for Div".to_string(),
                        ))
                    }
                }
            }

            NodeType::Mod => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::Int(a), Value::Int(b)) => {
                        if b == 0 {
                            return Err(ASGError::InvalidOperation("Modulo by zero".to_string()));
                        }
                        Value::Int(a % b)
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected two integers for Mod".to_string(),
                        ))
                    }
                }
            }

            NodeType::IntDiv => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::Int(a), Value::Int(b)) => {
                        if b == 0 {
                            return Err(ASGError::InvalidOperation("Division by zero".to_string()));
                        }
                        Value::Int(a / b)
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        if b == 0.0 {
                            return Err(ASGError::InvalidOperation("Division by zero".to_string()));
                        }
                        Value::Int((a / b).floor() as i64)
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected two numbers for IntDiv".to_string(),
                        ))
                    }
                }
            }

            NodeType::Neg => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Int(a) => Value::Int(-a),
                    Value::Float(a) => Value::Float(-a),
                    _ => return Err(ASGError::TypeError("Expected number for Neg".to_string())),
                }
            }

            // === Операции сравнения ===
            NodeType::Eq => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                let result = match (val1, val2) {
                    (Value::Int(a), Value::Int(b)) => a == b,
                    (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
                    (Value::Bool(a), Value::Bool(b)) => a == b,
                    (Value::String(a), Value::String(b)) => a == b,
                    _ => false,
                };
                Value::Bool(result)
            }

            NodeType::Ne => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                let result = match (val1, val2) {
                    (Value::Int(a), Value::Int(b)) => a != b,
                    (Value::Float(a), Value::Float(b)) => (a - b).abs() >= f64::EPSILON,
                    (Value::Bool(a), Value::Bool(b)) => a != b,
                    (Value::String(a), Value::String(b)) => a != b,
                    _ => true,
                };
                Value::Bool(result)
            }

            NodeType::Lt => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::Int(a), Value::Int(b)) => Value::Bool(a < b),
                    (Value::Float(a), Value::Float(b)) => Value::Bool(a < b),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected two numbers for Lt".to_string(),
                        ))
                    }
                }
            }

            NodeType::Le => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::Int(a), Value::Int(b)) => Value::Bool(a <= b),
                    (Value::Float(a), Value::Float(b)) => Value::Bool(a <= b),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected two numbers for Le".to_string(),
                        ))
                    }
                }
            }

            NodeType::Gt => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::Int(a), Value::Int(b)) => Value::Bool(a > b),
                    (Value::Float(a), Value::Float(b)) => Value::Bool(a > b),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected two numbers for Gt".to_string(),
                        ))
                    }
                }
            }

            NodeType::Ge => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::Int(a), Value::Int(b)) => Value::Bool(a >= b),
                    (Value::Float(a), Value::Float(b)) => Value::Bool(a >= b),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected two numbers for Ge".to_string(),
                        ))
                    }
                }
            }

            // === Логические операции ===
            NodeType::And => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::Bool(a), Value::Bool(b)) => Value::Bool(a && b),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected two booleans for And".to_string(),
                        ))
                    }
                }
            }

            NodeType::Or => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::Bool(a), Value::Bool(b)) => Value::Bool(a || b),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected two booleans for Or".to_string(),
                        ))
                    }
                }
            }

            NodeType::Not => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Bool(a) => Value::Bool(!a),
                    _ => return Err(ASGError::TypeError("Expected boolean for Not".to_string())),
                }
            }

            // === If выражение ===
            NodeType::If => {
                let cond_edge = node
                    .find_edge(EdgeType::Condition)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::Condition))?;

                let cond_val = self.ensure_evaluated(asg, cond_edge.target_node_id)?;
                let cond = cond_val
                    .as_bool()
                    .ok_or(ASGError::TypeError("Condition must be boolean".to_string()))?;

                if cond {
                    let then_edge = node
                        .find_edge(EdgeType::ThenBranch)
                        .ok_or(ASGError::MissingEdge(node.id, EdgeType::ThenBranch))?;
                    self.ensure_evaluated(asg, then_edge.target_node_id)?
                } else if let Some(else_edge) = node.find_edge(EdgeType::ElseBranch) {
                    self.ensure_evaluated(asg, else_edge.target_node_id)?
                } else {
                    Value::Unit
                }
            }

            // === Block ===
            NodeType::Block => {
                let stmt_edges: Vec<_> = node
                    .find_edges(EdgeType::BlockStatement)
                    .into_iter()
                    .map(|e| e.target_node_id)
                    .collect();
                let mut result = Value::Unit;
                for target_id in stmt_edges {
                    result = self.ensure_evaluated(asg, target_id)?;
                }
                result
            }

            // === Loop (while) ===
            NodeType::Loop => {
                let body_edge = node
                    .find_edge(EdgeType::LoopBody)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::LoopBody))?;

                let body_node = asg
                    .find_node(body_edge.target_node_id)
                    .ok_or(ASGError::NodeNotFound(body_edge.target_node_id))?
                    .clone();

                // Если есть условие - это while loop
                if let Some(cond_edge) = node.find_edge(EdgeType::Condition) {
                    let mut result = Value::Unit;
                    loop {
                        // Очищаем весь кеш для пересчёта (переменные могли измениться)
                        self.memo.clear();

                        let cond_val = self.ensure_evaluated(asg, cond_edge.target_node_id)?;
                        let cond = cond_val.as_bool().ok_or(ASGError::TypeError(
                            "Loop condition must be boolean".to_string(),
                        ))?;

                        if !cond {
                            break;
                        }

                        // Выполняем тело
                        self.memo.clear();
                        result = self.ensure_evaluated(asg, body_edge.target_node_id)?;
                    }
                    result
                } else {
                    // Бесконечный цикл без условия
                    loop {
                        self.memo.clear();
                        self.eval_node(asg, &body_node)?;
                    }
                }
            }

            // === Переменные ===
            NodeType::Variable => {
                let var_name = node.get_name().ok_or(ASGError::MissingPayload(node.id))?;

                let value = if let Some(val_edge) = node.find_edge(EdgeType::VarValue) {
                    self.ensure_evaluated(asg, val_edge.target_node_id)?
                } else {
                    Value::Unit
                };

                self.variables.insert(var_name, value.clone());
                value
            }

            NodeType::LetDestructure => {
                // Декодируем имена из payload
                let payload = node
                    .payload
                    .as_ref()
                    .ok_or(ASGError::MissingPayload(node.id))?;

                // Первые 4 байта - количество имён
                let count =
                    u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]) as usize;
                let mut names = Vec::with_capacity(count);
                let mut pos = 4;

                for _ in 0..count {
                    let end = payload[pos..]
                        .iter()
                        .position(|&b| b == 0)
                        .map(|p| pos + p)
                        .unwrap_or(payload.len());
                    let name = String::from_utf8_lossy(&payload[pos..end]).to_string();
                    names.push(name);
                    pos = end + 1;
                }

                // Вычисляем значение
                let val_edge = node
                    .find_edge(EdgeType::VarValue)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::VarValue))?;
                let value = self.ensure_evaluated(asg, val_edge.target_node_id)?;

                // Деструктуризация
                match &value {
                    Value::Array(arr) => {
                        for (i, name) in names.iter().enumerate() {
                            let val = arr.get(i).cloned().unwrap_or(Value::Unit);
                            self.variables.insert(name.clone(), val);
                        }
                    }
                    Value::Record(rec) => {
                        for name in &names {
                            let val = rec.get(name).cloned().unwrap_or(Value::Unit);
                            self.variables.insert(name.clone(), val);
                        }
                    }
                    Value::Dict(dict) => {
                        for name in &names {
                            let val = dict.get(name).cloned().unwrap_or(Value::Unit);
                            self.variables.insert(name.clone(), val);
                        }
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Destructuring requires Array, Record, or Dict".to_string(),
                        ));
                    }
                }

                value
            }

            NodeType::VarRef => {
                let var_name = node.get_name().ok_or(ASGError::MissingPayload(node.id))?;
                // Сначала ищем в переменных
                if let Some(val) = self.resolve_variable(&var_name) {
                    val.clone()
                } else if let Some((params, body_id, _)) = self.functions.get(&var_name) {
                    // Если не нашли в переменных, ищем в функциях
                    Value::Function {
                        params: params.clone(),
                        body_id: *body_id,
                        captured: HashMap::new(),
                    }
                } else {
                    return Err(ASGError::UnknownVariable(var_name));
                }
            }

            NodeType::Assign => {
                let target_edge = node
                    .find_edge(EdgeType::AssignTarget)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::AssignTarget))?;
                let value_edge = node
                    .find_edge(EdgeType::AssignValue)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::AssignValue))?;

                let target_node = asg
                    .find_node(target_edge.target_node_id)
                    .ok_or(ASGError::NodeNotFound(target_edge.target_node_id))?;
                let var_name = target_node
                    .get_name()
                    .ok_or(ASGError::MissingPayload(target_node.id))?;

                let value = self.ensure_evaluated(asg, value_edge.target_node_id)?;
                self.variables.insert(var_name, value);
                Value::Unit
            }

            // === Функции ===
            NodeType::Function => {
                let func_name = node.get_name().unwrap_or_else(|| format!("fn_{}", node.id));

                // Собираем имена параметров
                let param_edges = node.find_edges(EdgeType::FunctionParameter);
                let mut params = Vec::new();
                for edge in param_edges {
                    let param_node = asg
                        .find_node(edge.target_node_id)
                        .ok_or(ASGError::NodeNotFound(edge.target_node_id))?;
                    let param_name = param_node
                        .get_name()
                        .unwrap_or_else(|| format!("param_{}", param_node.id));
                    params.push(param_name);
                }

                // Получаем ID тела
                let body_id = node
                    .find_edge(EdgeType::FunctionBody)
                    .map(|e| e.target_node_id)
                    .unwrap_or(0);

                self.functions
                    .insert(func_name.clone(), (params.clone(), body_id, None));

                Value::Function {
                    params,
                    body_id,
                    captured: HashMap::new(),
                }
            }

            NodeType::Parameter => {
                let param_name = node
                    .get_name()
                    .unwrap_or_else(|| format!("param_{}", node.id));
                // Параметр получает значение из стека вызовов или глобальных переменных
                self.resolve_variable(&param_name)
                    .cloned()
                    .unwrap_or(Value::Unit)
            }

            NodeType::Lambda => {
                // Lambda — анонимная функция с захватом переменных (closure)
                let param_edges = node.find_edges(EdgeType::FunctionParameter);
                let mut params = Vec::new();
                for edge in param_edges {
                    let param_node = asg
                        .find_node(edge.target_node_id)
                        .ok_or(ASGError::NodeNotFound(edge.target_node_id))?;
                    let param_name = param_node
                        .get_name()
                        .unwrap_or_else(|| format!("param_{}", param_node.id));
                    params.push(param_name);
                }

                let body_id = node
                    .find_edge(EdgeType::FunctionBody)
                    .map(|e| e.target_node_id)
                    .unwrap_or(0);

                // Захватываем текущий scope для closure
                let mut captured = HashMap::new();
                // Сначала глобальные переменные
                for (name, val) in &self.variables {
                    captured.insert(name.clone(), val.clone());
                }
                // Затем переменные из стека вызовов (перезаписывают глобальные)
                for frame in &self.call_stack {
                    for (name, val) in &frame.locals {
                        captured.insert(name.clone(), val.clone());
                    }
                }

                Value::Function {
                    params,
                    body_id,
                    captured,
                }
            }

            NodeType::Call => {
                // Получаем функцию
                let call_target = node
                    .find_edge(EdgeType::CallTarget)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::CallTarget))?;
                let target_node = asg
                    .find_node(call_target.target_node_id)
                    .ok_or(ASGError::NodeNotFound(call_target.target_node_id))?;
                let func_name = target_node.get_name().unwrap_or_default();

                // Собираем аргументы сначала
                let arg_edges: Vec<_> = node
                    .edges
                    .iter()
                    .filter(|e| {
                        e.edge_type == EdgeType::CallArgument
                            || e.edge_type == EdgeType::ApplicationArgument
                    })
                    .collect();

                let arg_ids: Vec<_> = arg_edges.iter().map(|e| e.target_node_id).collect();
                let mut arg_values = Vec::with_capacity(arg_ids.len());
                for arg_id in &arg_ids {
                    let arg_val = self.ensure_evaluated(asg, *arg_id)?;
                    arg_values.push(arg_val);
                }

                // Пробуем найти именованную функцию
                if let Some((params, body_id, opt_asg)) = self.functions.get(&func_name).cloned() {
                    // Именованная функция (возможно из импортированного модуля)
                    let mut frame = CallFrame::default();
                    for (i, arg_val) in arg_values.into_iter().enumerate() {
                        if i < params.len() {
                            frame.locals.insert(params[i].clone(), arg_val);
                        }
                    }

                    let saved_memo = std::mem::take(&mut self.memo);
                    frame.memo = saved_memo;
                    self.call_stack.push(frame);

                    let result = if body_id != 0 {
                        if let Some(ref imported_asg) = opt_asg {
                            self.ensure_evaluated(imported_asg, body_id)?
                        } else {
                            self.ensure_evaluated(asg, body_id)?
                        }
                    } else {
                        Value::Unit
                    };

                    if let Some(popped_frame) = self.call_stack.pop() {
                        self.memo = popped_frame.memo;
                    }
                    result
                } else {
                    // Попробуем вычислить target как значение
                    let fn_val = self.ensure_evaluated(asg, call_target.target_node_id)?;
                    match fn_val {
                        Value::Function {
                            params,
                            body_id,
                            captured,
                        } => {
                            let mut frame = CallFrame::default();
                            for (name, val) in &captured {
                                frame.locals.insert(name.clone(), val.clone());
                            }
                            for (i, arg_val) in arg_values.into_iter().enumerate() {
                                if i < params.len() {
                                    frame.locals.insert(params[i].clone(), arg_val);
                                }
                            }

                            let saved_memo = std::mem::take(&mut self.memo);
                            frame.memo = saved_memo;
                            self.call_stack.push(frame);

                            let result = self.ensure_evaluated(asg, body_id)?;

                            if let Some(popped_frame) = self.call_stack.pop() {
                                self.memo = popped_frame.memo;
                            }
                            result
                        }
                        Value::ComposedFunction(_) => {
                            // Для ComposedFunction используем первый аргумент
                            let arg = arg_values.into_iter().next().unwrap_or(Value::Unit);
                            self.call_function_value(asg, fn_val, arg)?
                        }
                        _ => return Err(ASGError::UnknownFunction(func_name)),
                    }
                }
            }

            NodeType::Return => {
                if let Some(edge) = node.find_edge(EdgeType::ReturnValue) {
                    self.ensure_evaluated(asg, edge.target_node_id)?
                } else {
                    Value::Unit
                }
            }

            // === Тензорные операции ===
            NodeType::TensorAdd => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::Tensor(a), Value::Tensor(b)) => Value::Tensor(tensor_ops::add(&a, &b)),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected two Tensors for TensorAdd".to_string(),
                        ))
                    }
                }
            }

            NodeType::TensorMul => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::Tensor(a), Value::Tensor(b)) => {
                        // Поэлементное умножение
                        let lhs = a.data.borrow();
                        let rhs = b.data.borrow();
                        let result_data = &*lhs * &*rhs;
                        Value::Tensor(DifferentiableTensor::new(result_data, false))
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected two Tensors for TensorMul".to_string(),
                        ))
                    }
                }
            }

            // === Массивы ===
            NodeType::Array => {
                let element_ids: Vec<_> = node
                    .find_edges(EdgeType::ArrayElement)
                    .into_iter()
                    .map(|e| e.target_node_id)
                    .collect();
                let mut elements = Vec::new();
                for elem_id in element_ids {
                    let elem_val = self.ensure_evaluated(asg, elem_id)?;
                    elements.push(elem_val);
                }
                Value::Array(elements)
            }

            NodeType::ArrayIndex => {
                let array_edge = node.edges.first().ok_or(ASGError::MissingEdge(
                    node.id,
                    EdgeType::ApplicationArgument,
                ))?;
                let index_edge = node
                    .find_edge(EdgeType::ArrayIndexExpr)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::ArrayIndexExpr))?;

                let array_id = array_edge.target_node_id;
                let index_id = index_edge.target_node_id;
                let array_val = self.ensure_evaluated(asg, array_id)?;
                let index_val = self.ensure_evaluated(asg, index_id)?;

                match (&array_val, &index_val) {
                    (Value::Array(arr), Value::Int(idx)) => {
                        let idx = *idx as usize;
                        arr.get(idx)
                            .cloned()
                            .ok_or(ASGError::InvalidOperation(format!(
                                "Array index {} out of bounds",
                                idx
                            )))?
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected array and integer index".to_string(),
                        ))
                    }
                }
            }

            NodeType::ArrayLength => {
                let array_edge = node.edges.first().ok_or(ASGError::MissingEdge(
                    node.id,
                    EdgeType::ApplicationArgument,
                ))?;

                let array_val = self.ensure_evaluated(asg, array_edge.target_node_id)?;

                match &array_val {
                    Value::Array(arr) => Value::Int(arr.len() as i64),
                    _ => return Err(ASGError::TypeError("Expected array for length".to_string())),
                }
            }

            NodeType::ArrayLast => {
                let array_edge = node.edges.first().ok_or(ASGError::MissingEdge(
                    node.id,
                    EdgeType::ApplicationArgument,
                ))?;

                let array_val = self.ensure_evaluated(asg, array_edge.target_node_id)?;

                match array_val {
                    Value::Array(arr) => arr.last().cloned().unwrap_or(Value::Unit),
                    _ => return Err(ASGError::TypeError("Expected array for last".to_string())),
                }
            }

            NodeType::ArraySetIndex => {
                let array_edge = node.edges.first().ok_or(ASGError::MissingEdge(
                    node.id,
                    EdgeType::ApplicationArgument,
                ))?;
                let index_edge = node
                    .find_edge(EdgeType::ArrayIndexExpr)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::ArrayIndexExpr))?;
                let value_edge = node
                    .find_edge(EdgeType::AssignValue)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::AssignValue))?;

                // Получаем имя переменной массива
                let array_node = asg
                    .find_node(array_edge.target_node_id)
                    .ok_or(ASGError::NodeNotFound(array_edge.target_node_id))?;
                let var_name = array_node
                    .get_name()
                    .ok_or(ASGError::MissingPayload(array_node.id))?;

                let index_val = self.ensure_evaluated(asg, index_edge.target_node_id)?;
                let new_value = self.ensure_evaluated(asg, value_edge.target_node_id)?;

                let idx = match &index_val {
                    Value::Int(i) => *i as usize,
                    _ => return Err(ASGError::TypeError("Index must be integer".to_string())),
                };

                // Мутируем массив в переменной
                if let Some(Value::Array(ref mut arr)) = self.variables.get_mut(&var_name) {
                    if idx < arr.len() {
                        arr[idx] = new_value;
                    } else {
                        return Err(ASGError::InvalidOperation(format!(
                            "Array index {} out of bounds",
                            idx
                        )));
                    }
                } else {
                    return Err(ASGError::TypeError(
                        "Expected array variable for set-index".to_string(),
                    ));
                }
                Value::Unit
            }

            // === Higher-order array functions ===
            NodeType::ArrayMap => {
                let array_edge = node
                    .find_edge(EdgeType::SourceArray)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::SourceArray))?;
                let fn_edge = node
                    .find_edge(EdgeType::MapFunction)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::MapFunction))?;

                let array_val = self.ensure_evaluated(asg, array_edge.target_node_id)?;
                let fn_val = self.ensure_evaluated(asg, fn_edge.target_node_id)?;

                let arr = match &array_val {
                    Value::Array(a) => a.clone(),
                    _ => return Err(ASGError::TypeError("Expected array for map".to_string())),
                };

                let (params, body_id, captured) = match &fn_val {
                    Value::Function {
                        params,
                        body_id,
                        captured,
                    } => (params.clone(), *body_id, captured.clone()),
                    _ => return Err(ASGError::TypeError("Expected function for map".to_string())),
                };

                let mut result = Vec::with_capacity(arr.len());
                for elem in arr {
                    // Создаём frame для вызова функции
                    let saved_memo = std::mem::take(&mut self.memo);
                    let mut frame = CallFrame::default();
                    // Добавляем captured переменные (closure)
                    for (name, val) in &captured {
                        frame.locals.insert(name.clone(), val.clone());
                    }
                    if !params.is_empty() {
                        frame.locals.insert(params[0].clone(), elem);
                    }
                    frame.memo = saved_memo;
                    self.call_stack.push(frame);

                    let mapped = self.ensure_evaluated(asg, body_id)?;
                    result.push(mapped);

                    if let Some(popped_frame) = self.call_stack.pop() {
                        self.memo = popped_frame.memo;
                    }
                }
                Value::Array(result)
            }

            NodeType::ArrayFilter => {
                let array_edge = node
                    .find_edge(EdgeType::SourceArray)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::SourceArray))?;
                let pred_edge = node
                    .find_edge(EdgeType::FilterPredicate)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::FilterPredicate))?;

                let array_val = self.ensure_evaluated(asg, array_edge.target_node_id)?;
                let pred_val = self.ensure_evaluated(asg, pred_edge.target_node_id)?;

                let arr = match &array_val {
                    Value::Array(a) => a.clone(),
                    _ => return Err(ASGError::TypeError("Expected array for filter".to_string())),
                };

                let (params, body_id, captured) = match &pred_val {
                    Value::Function {
                        params,
                        body_id,
                        captured,
                    } => (params.clone(), *body_id, captured.clone()),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected function for filter".to_string(),
                        ))
                    }
                };

                let mut result = Vec::new();
                for elem in arr {
                    // Создаём frame для вызова предиката
                    let saved_memo = std::mem::take(&mut self.memo);
                    let mut frame = CallFrame::default();
                    // Добавляем captured переменные (closure)
                    for (name, val) in &captured {
                        frame.locals.insert(name.clone(), val.clone());
                    }
                    if !params.is_empty() {
                        frame.locals.insert(params[0].clone(), elem.clone());
                    }
                    frame.memo = saved_memo;
                    self.call_stack.push(frame);

                    let pred_result = self.ensure_evaluated(asg, body_id)?;

                    if let Some(popped_frame) = self.call_stack.pop() {
                        self.memo = popped_frame.memo;
                    }

                    if let Value::Bool(true) = pred_result {
                        result.push(elem);
                    }
                }
                Value::Array(result)
            }

            NodeType::ArrayReduce => {
                let array_edge = node
                    .find_edge(EdgeType::SourceArray)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::SourceArray))?;
                let init_edge = node
                    .find_edge(EdgeType::ReduceInit)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::ReduceInit))?;
                let fn_edge = node
                    .find_edge(EdgeType::ReduceFunction)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::ReduceFunction))?;

                let array_val = self.ensure_evaluated(asg, array_edge.target_node_id)?;
                let init_val = self.ensure_evaluated(asg, init_edge.target_node_id)?;
                let fn_val = self.ensure_evaluated(asg, fn_edge.target_node_id)?;

                let arr = match &array_val {
                    Value::Array(a) => a.clone(),
                    _ => return Err(ASGError::TypeError("Expected array for reduce".to_string())),
                };

                let (params, body_id, captured) = match &fn_val {
                    Value::Function {
                        params,
                        body_id,
                        captured,
                    } => (params.clone(), *body_id, captured.clone()),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected function for reduce".to_string(),
                        ))
                    }
                };

                let mut acc = init_val;
                for elem in arr {
                    // Создаём frame для вызова функции с acc и elem
                    let saved_memo = std::mem::take(&mut self.memo);
                    let mut frame = CallFrame::default();
                    // Добавляем captured переменные (closure)
                    for (name, val) in &captured {
                        frame.locals.insert(name.clone(), val.clone());
                    }
                    if params.len() >= 1 {
                        frame.locals.insert(params[0].clone(), acc);
                    }
                    if params.len() >= 2 {
                        frame.locals.insert(params[1].clone(), elem);
                    }
                    frame.memo = saved_memo;
                    self.call_stack.push(frame);

                    acc = self.ensure_evaluated(asg, body_id)?;

                    if let Some(popped_frame) = self.call_stack.pop() {
                        self.memo = popped_frame.memo;
                    }
                }
                acc
            }

            NodeType::ListComprehension => {
                // (list-comp expr var iter [condition])
                let var_name = node.get_name().ok_or(ASGError::MissingPayload(node.id))?;

                let expr_edge = node
                    .find_edge(EdgeType::MapFunction)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::MapFunction))?;
                let iter_edge = node
                    .find_edge(EdgeType::LoopInit)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::LoopInit))?;
                let cond_edge = node.find_edge(EdgeType::Condition);

                let iter_val = self.ensure_evaluated(asg, iter_edge.target_node_id)?;

                let arr = match &iter_val {
                    Value::Array(a) => a.clone(),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected array for list comprehension".to_string(),
                        ))
                    }
                };

                let mut result = Vec::new();
                for elem in arr {
                    // Создаём frame для итерации
                    let saved_memo = std::mem::take(&mut self.memo);
                    let mut frame = CallFrame::default();
                    frame.locals.insert(var_name.clone(), elem);
                    frame.memo = saved_memo;
                    self.call_stack.push(frame);

                    // Проверяем условие (если есть)
                    let include = if let Some(cond) = &cond_edge {
                        let cond_val = self.ensure_evaluated(asg, cond.target_node_id)?;
                        match cond_val {
                            Value::Bool(b) => b,
                            _ => true,
                        }
                    } else {
                        true
                    };

                    if include {
                        let expr_val = self.ensure_evaluated(asg, expr_edge.target_node_id)?;
                        result.push(expr_val);
                    }

                    if let Some(popped_frame) = self.call_stack.pop() {
                        self.memo = popped_frame.memo;
                    }
                }
                Value::Array(result)
            }

            // === Lazy Sequences ===
            NodeType::Iterate => {
                // (iterate fn init) -> lazy [init, fn(init), fn(fn(init)), ...]
                let (fn_val, init_val) = self.get_binary_operands(asg, node)?;
                Value::LazySeq(Box::new(LazySeqKind::Iterate {
                    func: Box::new(fn_val),
                    current: Box::new(init_val),
                }))
            }

            NodeType::Repeat => {
                // (repeat val) -> lazy [val, val, val, ...]
                let val = self.get_single_operand(asg, node)?;
                Value::LazySeq(Box::new(LazySeqKind::Repeat(Box::new(val))))
            }

            NodeType::Cycle => {
                // (cycle arr) -> lazy [a,b,c,a,b,c,...]
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Array(arr) => {
                        Value::LazySeq(Box::new(LazySeqKind::Cycle { arr, index: 0 }))
                    }
                    _ => return Err(ASGError::TypeError("Expected array for cycle".to_string())),
                }
            }

            NodeType::LazyRange => {
                // (lazy-range start end [step])
                let start_val = self.get_first_operand(asg, node)?;
                let end_val = self.get_second_operand(asg, node)?;
                let step_edge = node.find_edge(EdgeType::LoopStep);

                let start = match start_val {
                    Value::Int(n) => n,
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected integer for lazy-range start".to_string(),
                        ))
                    }
                };
                let end = match end_val {
                    Value::Int(n) => n,
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected integer for lazy-range end".to_string(),
                        ))
                    }
                };
                let step = if let Some(step_e) = step_edge {
                    let step_val = self.ensure_evaluated(asg, step_e.target_node_id)?;
                    match step_val {
                        Value::Int(n) => n,
                        _ => {
                            return Err(ASGError::TypeError(
                                "Expected integer for lazy-range step".to_string(),
                            ))
                        }
                    }
                } else {
                    if start <= end {
                        1
                    } else {
                        -1
                    }
                };

                Value::LazySeq(Box::new(LazySeqKind::Range {
                    current: start,
                    end,
                    step,
                }))
            }

            NodeType::TakeLazy => {
                // (take-lazy n seq) -> take n elements from lazy seq
                let (n_val, seq_val) = self.get_binary_operands(asg, node)?;
                let n = match n_val {
                    Value::Int(n) => n as usize,
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected integer for take-lazy count".to_string(),
                        ))
                    }
                };

                match seq_val {
                    Value::LazySeq(kind) => {
                        let result = self.take_from_lazy(asg, *kind, n)?;
                        Value::Array(result)
                    }
                    Value::Array(arr) => {
                        // Поддержка take для обычных массивов тоже
                        Value::Array(arr.into_iter().take(n).collect())
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected lazy sequence or array for take-lazy".to_string(),
                        ))
                    }
                }
            }

            NodeType::LazyMap => {
                // (lazy-map fn seq) -> lazy mapped seq
                let (fn_val, seq_val) = self.get_binary_operands(asg, node)?;
                match seq_val {
                    Value::LazySeq(kind) => Value::LazySeq(Box::new(LazySeqKind::Map {
                        func: Box::new(fn_val),
                        source: kind,
                    })),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected lazy sequence for lazy-map".to_string(),
                        ))
                    }
                }
            }

            NodeType::LazyFilter => {
                // (lazy-filter fn seq) -> lazy filtered seq
                let (fn_val, seq_val) = self.get_binary_operands(asg, node)?;
                match seq_val {
                    Value::LazySeq(kind) => Value::LazySeq(Box::new(LazySeqKind::Filter {
                        func: Box::new(fn_val),
                        source: kind,
                    })),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected lazy sequence for lazy-filter".to_string(),
                        ))
                    }
                }
            }

            NodeType::Collect => {
                // (collect seq) -> materialize lazy seq (limited to 10000 elements)
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::LazySeq(kind) => {
                        let result = self.take_from_lazy(asg, *kind, 10000)?;
                        Value::Array(result)
                    }
                    Value::Array(arr) => Value::Array(arr),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected lazy sequence for collect".to_string(),
                        ))
                    }
                }
            }

            // === Range and iterators ===
            NodeType::Range => {
                let start_val = self.get_first_operand(asg, node)?;
                let end_val = self.get_second_operand(asg, node)?;

                let start = match start_val {
                    Value::Int(n) => n,
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected integer for range start".to_string(),
                        ))
                    }
                };
                let end = match end_val {
                    Value::Int(n) => n,
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected integer for range end".to_string(),
                        ))
                    }
                };

                let step = if let Some(step_edge) = node.find_edge(EdgeType::LoopStep) {
                    let step_val = self.ensure_evaluated(asg, step_edge.target_node_id)?;
                    match step_val {
                        Value::Int(n) => n,
                        _ => {
                            return Err(ASGError::TypeError(
                                "Expected integer for range step".to_string(),
                            ))
                        }
                    }
                } else {
                    1
                };

                if step == 0 {
                    return Err(ASGError::InvalidOperation(
                        "Range step cannot be zero".to_string(),
                    ));
                }

                let mut result = Vec::new();
                let mut i = start;
                if step > 0 {
                    while i < end {
                        result.push(Value::Int(i));
                        i += step;
                    }
                } else {
                    while i > end {
                        result.push(Value::Int(i));
                        i += step;
                    }
                }
                Value::Array(result)
            }

            NodeType::For => {
                let var_edge = node
                    .find_edge(EdgeType::LoopInit)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::LoopInit))?;
                let iterable_edge = node
                    .find_edge(EdgeType::Condition)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::Condition))?;
                let body_edge = node
                    .find_edge(EdgeType::LoopBody)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::LoopBody))?;

                let var_node = asg
                    .find_node(var_edge.target_node_id)
                    .ok_or(ASGError::NodeNotFound(var_edge.target_node_id))?;
                let var_name = var_node.get_name().unwrap_or_default();

                let iterable_val = self.ensure_evaluated(asg, iterable_edge.target_node_id)?;
                let items = match iterable_val {
                    Value::Array(arr) => arr,
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected array for for loop".to_string(),
                        ))
                    }
                };

                let mut last_result = Value::Unit;
                for item in items {
                    let saved_memo = std::mem::take(&mut self.memo);
                    let mut frame = CallFrame::default();
                    frame.locals.insert(var_name.clone(), item);
                    frame.memo = saved_memo;
                    self.call_stack.push(frame);

                    last_result = self.ensure_evaluated(asg, body_edge.target_node_id)?;

                    if let Some(popped_frame) = self.call_stack.pop() {
                        self.memo = popped_frame.memo;
                    }
                }
                last_result
            }

            NodeType::ArrayReverse => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Array(mut arr) => {
                        arr.reverse();
                        Value::Array(arr)
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected array for reverse".to_string(),
                        ))
                    }
                }
            }

            NodeType::ArraySort => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Array(mut arr) => {
                        arr.sort_by(|a, b| match (a, b) {
                            (Value::Int(x), Value::Int(y)) => x.cmp(y),
                            (Value::Float(x), Value::Float(y)) => {
                                x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                            }
                            (Value::String(x), Value::String(y)) => x.cmp(y),
                            _ => std::cmp::Ordering::Equal,
                        });
                        Value::Array(arr)
                    }
                    _ => return Err(ASGError::TypeError("Expected array for sort".to_string())),
                }
            }

            NodeType::ArraySum => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Array(arr) => {
                        let mut int_sum = 0i64;
                        let mut float_sum = 0.0f64;
                        let mut has_float = false;
                        for item in arr {
                            match item {
                                Value::Int(n) => int_sum += n,
                                Value::Float(f) => {
                                    float_sum += f;
                                    has_float = true;
                                }
                                _ => {
                                    return Err(ASGError::TypeError(
                                        "Expected numbers in array for sum".to_string(),
                                    ))
                                }
                            }
                        }
                        if has_float {
                            Value::Float(int_sum as f64 + float_sum)
                        } else {
                            Value::Int(int_sum)
                        }
                    }
                    _ => return Err(ASGError::TypeError("Expected array for sum".to_string())),
                }
            }

            NodeType::ArrayProduct => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Array(arr) => {
                        if arr.is_empty() {
                            return Ok(());
                        }
                        let mut int_prod = 1i64;
                        let mut float_prod = 1.0f64;
                        let mut has_float = false;
                        for item in arr {
                            match item {
                                Value::Int(n) => int_prod *= n,
                                Value::Float(f) => {
                                    float_prod *= f;
                                    has_float = true;
                                }
                                _ => {
                                    return Err(ASGError::TypeError(
                                        "Expected numbers in array for product".to_string(),
                                    ))
                                }
                            }
                        }
                        if has_float {
                            Value::Float(int_prod as f64 * float_prod)
                        } else {
                            Value::Int(int_prod)
                        }
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected array for product".to_string(),
                        ))
                    }
                }
            }

            NodeType::ArrayContains => {
                let (arr_val, elem_val) = self.get_binary_operands(asg, node)?;
                match arr_val {
                    Value::Array(arr) => {
                        let found = arr.iter().any(|item| self.values_equal(item, &elem_val));
                        Value::Bool(found)
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected array for contains".to_string(),
                        ))
                    }
                }
            }

            NodeType::ArrayIndexOf => {
                let (arr_val, elem_val) = self.get_binary_operands(asg, node)?;
                match arr_val {
                    Value::Array(arr) => {
                        let idx = arr
                            .iter()
                            .position(|item| self.values_equal(item, &elem_val));
                        match idx {
                            Some(i) => Value::Int(i as i64),
                            None => Value::Int(-1),
                        }
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected array for index-of".to_string(),
                        ))
                    }
                }
            }

            NodeType::ArrayTake => {
                let (arr_val, n_val) = self.get_binary_operands(asg, node)?;
                match (arr_val, n_val) {
                    (Value::Array(arr), Value::Int(n)) => {
                        let n = n.max(0) as usize;
                        Value::Array(arr.into_iter().take(n).collect())
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected (array, int) for take".to_string(),
                        ))
                    }
                }
            }

            NodeType::ArrayDrop => {
                let (arr_val, n_val) = self.get_binary_operands(asg, node)?;
                match (arr_val, n_val) {
                    (Value::Array(arr), Value::Int(n)) => {
                        let n = n.max(0) as usize;
                        Value::Array(arr.into_iter().skip(n).collect())
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected (array, int) for drop".to_string(),
                        ))
                    }
                }
            }

            NodeType::ArrayAppend => {
                let (arr_val, elem_val) = self.get_binary_operands(asg, node)?;
                match arr_val {
                    Value::Array(mut arr) => {
                        arr.push(elem_val);
                        Value::Array(arr)
                    }
                    _ => return Err(ASGError::TypeError("Expected array for append".to_string())),
                }
            }

            NodeType::ArrayConcat => {
                let (arr1_val, arr2_val) = self.get_binary_operands(asg, node)?;
                match (arr1_val, arr2_val) {
                    (Value::Array(mut arr1), Value::Array(arr2)) => {
                        arr1.extend(arr2);
                        Value::Array(arr1)
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected two arrays for array-concat".to_string(),
                        ))
                    }
                }
            }

            NodeType::ArraySlice => {
                let edges: Vec<_> = node.edges.iter().collect();
                if edges.len() != 3 {
                    return Err(ASGError::InvalidOperation(
                        "slice requires 3 arguments".to_string(),
                    ));
                }
                let arr_val = self.ensure_evaluated(asg, edges[0].target_node_id)?;
                let start_val = self.ensure_evaluated(asg, edges[1].target_node_id)?;
                let end_val = self.ensure_evaluated(asg, edges[2].target_node_id)?;

                match (arr_val, start_val, end_val) {
                    (Value::Array(arr), Value::Int(start), Value::Int(end)) => {
                        let start = start.max(0) as usize;
                        let end = (end as usize).min(arr.len());
                        if start >= end {
                            Value::Array(vec![])
                        } else {
                            Value::Array(arr[start..end].to_vec())
                        }
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected (array, int, int) for slice".to_string(),
                        ))
                    }
                }
            }

            // === Dict operations ===
            NodeType::Dict => {
                let mut dict = HashMap::new();
                let edges: Vec<_> = node.edges.iter().collect();
                let mut i = 0;
                while i + 1 < edges.len() {
                    let key_val = self.ensure_evaluated(asg, edges[i].target_node_id)?;
                    let val = self.ensure_evaluated(asg, edges[i + 1].target_node_id)?;
                    let key = match key_val {
                        Value::String(s) => s,
                        Value::Int(n) => n.to_string(),
                        _ => {
                            return Err(ASGError::TypeError(
                                "Dict keys must be strings or ints".to_string(),
                            ))
                        }
                    };
                    dict.insert(key, val);
                    i += 2;
                }
                Value::Dict(dict)
            }

            NodeType::DictGet => {
                let (dict_val, key_val) = self.get_binary_operands(asg, node)?;
                match (dict_val, key_val) {
                    (Value::Dict(dict), Value::String(key)) => {
                        dict.get(&key).cloned().unwrap_or(Value::Unit)
                    }
                    (Value::Dict(dict), Value::Int(n)) => {
                        dict.get(&n.to_string()).cloned().unwrap_or(Value::Unit)
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected (dict, key) for dict-get".to_string(),
                        ))
                    }
                }
            }

            NodeType::DictSet => {
                let edges: Vec<_> = node.edges.iter().collect();
                if edges.len() < 3 {
                    return Err(ASGError::MissingEdge(
                        node.id,
                        EdgeType::ApplicationArgument,
                    ));
                }
                let dict_val = self.ensure_evaluated(asg, edges[0].target_node_id)?;
                let key_val = self.ensure_evaluated(asg, edges[1].target_node_id)?;
                let new_val = self.ensure_evaluated(asg, edges[2].target_node_id)?;

                match (dict_val, key_val) {
                    (Value::Dict(mut dict), Value::String(key)) => {
                        dict.insert(key, new_val);
                        Value::Dict(dict)
                    }
                    (Value::Dict(mut dict), Value::Int(n)) => {
                        dict.insert(n.to_string(), new_val);
                        Value::Dict(dict)
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected (dict, key, value) for dict-set".to_string(),
                        ))
                    }
                }
            }

            NodeType::DictHas => {
                let (dict_val, key_val) = self.get_binary_operands(asg, node)?;
                match (dict_val, key_val) {
                    (Value::Dict(dict), Value::String(key)) => Value::Bool(dict.contains_key(&key)),
                    (Value::Dict(dict), Value::Int(n)) => {
                        Value::Bool(dict.contains_key(&n.to_string()))
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected (dict, key) for dict-has".to_string(),
                        ))
                    }
                }
            }

            NodeType::DictRemove => {
                let (dict_val, key_val) = self.get_binary_operands(asg, node)?;
                match (dict_val, key_val) {
                    (Value::Dict(mut dict), Value::String(key)) => {
                        dict.remove(&key);
                        Value::Dict(dict)
                    }
                    (Value::Dict(mut dict), Value::Int(n)) => {
                        dict.remove(&n.to_string());
                        Value::Dict(dict)
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected (dict, key) for dict-remove".to_string(),
                        ))
                    }
                }
            }

            NodeType::DictKeys => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Dict(dict) => {
                        Value::Array(dict.keys().map(|k| Value::String(k.clone())).collect())
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected dict for dict-keys".to_string(),
                        ))
                    }
                }
            }

            NodeType::DictValues => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Dict(dict) => Value::Array(dict.values().cloned().collect()),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected dict for dict-values".to_string(),
                        ))
                    }
                }
            }

            NodeType::DictMerge => {
                let (dict1_val, dict2_val) = self.get_binary_operands(asg, node)?;
                match (dict1_val, dict2_val) {
                    (Value::Dict(mut d1), Value::Dict(d2)) => {
                        for (k, v) in d2 {
                            d1.insert(k, v);
                        }
                        Value::Dict(d1)
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected two dicts for dict-merge".to_string(),
                        ))
                    }
                }
            }

            NodeType::DictSize => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Dict(dict) => Value::Int(dict.len() as i64),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected dict for dict-size".to_string(),
                        ))
                    }
                }
            }

            // === Pipe and Compose ===
            NodeType::Pipe => {
                // (|> value fn1 fn2 ...)
                let edges: Vec<_> = node.edges.iter().collect();
                if edges.is_empty() {
                    return Err(ASGError::InvalidOperation(
                        "Pipe requires at least one argument".to_string(),
                    ));
                }

                let mut current = self.ensure_evaluated(asg, edges[0].target_node_id)?;

                for edge in &edges[1..] {
                    let fn_val = self.ensure_evaluated(asg, edge.target_node_id)?;
                    current = self.call_function_value(asg, fn_val, current)?;
                }
                current
            }

            NodeType::Compose => {
                // (compose fn1 fn2 ...) - создаём композицию функций
                let edges: Vec<_> = node.edges.iter().collect();
                if edges.is_empty() {
                    return Err(ASGError::InvalidOperation(
                        "Compose requires at least one function".to_string(),
                    ));
                }

                let mut fns = Vec::new();
                for edge in &edges {
                    let fn_val = self.ensure_evaluated(asg, edge.target_node_id)?;
                    match &fn_val {
                        Value::Function { .. } | Value::ComposedFunction(_) => fns.push(fn_val),
                        _ => {
                            return Err(ASGError::TypeError(
                                "Compose expects functions".to_string(),
                            ))
                        }
                    }
                }
                Value::ComposedFunction(fns)
            }

            // === Строковые операции ===
            NodeType::StringConcat => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::String(s1), Value::String(s2)) => {
                        Value::String(format!("{}{}", s1, s2))
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected strings for concat".to_string(),
                        ))
                    }
                }
            }

            NodeType::StringLength => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::String(s) => Value::Int(s.chars().count() as i64),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected string for str-length".to_string(),
                        ))
                    }
                }
            }

            NodeType::StringSubstring => {
                let edges: Vec<_> = node.edges.iter().collect();
                if edges.len() < 3 {
                    return Err(ASGError::MissingEdge(
                        node.id,
                        EdgeType::ApplicationArgument,
                    ));
                }
                let str_val = self.ensure_evaluated(asg, edges[0].target_node_id)?;
                let start_val = self.ensure_evaluated(asg, edges[1].target_node_id)?;
                let end_val = self.ensure_evaluated(asg, edges[2].target_node_id)?;

                match (str_val, start_val, end_val) {
                    (Value::String(s), Value::Int(start), Value::Int(end)) => {
                        let chars: Vec<char> = s.chars().collect();
                        let start = start.max(0) as usize;
                        let end = (end as usize).min(chars.len());
                        if start >= end {
                            Value::String(String::new())
                        } else {
                            Value::String(chars[start..end].iter().collect())
                        }
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected (string, int, int) for substring".to_string(),
                        ))
                    }
                }
            }

            NodeType::StringSplit => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::String(s), Value::String(delim)) => {
                        let parts: Vec<Value> = s
                            .split(&delim)
                            .map(|p| Value::String(p.to_string()))
                            .collect();
                        Value::Array(parts)
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected strings for str-split".to_string(),
                        ))
                    }
                }
            }

            NodeType::StringJoin => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::Array(arr), Value::String(delim)) => {
                        let strings: Result<Vec<String>, _> = arr
                            .into_iter()
                            .map(|v| match v {
                                Value::String(s) => Ok(s),
                                _ => Err(ASGError::TypeError(
                                    "Array elements must be strings for str-join".to_string(),
                                )),
                            })
                            .collect();
                        Value::String(strings?.join(&delim))
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected (array, string) for str-join".to_string(),
                        ))
                    }
                }
            }

            NodeType::StringContains => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::String(s), Value::String(substr)) => Value::Bool(s.contains(&substr)),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected strings for str-contains".to_string(),
                        ))
                    }
                }
            }

            NodeType::StringReplace => {
                let edges: Vec<_> = node.edges.iter().collect();
                if edges.len() < 3 {
                    return Err(ASGError::MissingEdge(
                        node.id,
                        EdgeType::ApplicationArgument,
                    ));
                }
                let str_val = self.ensure_evaluated(asg, edges[0].target_node_id)?;
                let from_val = self.ensure_evaluated(asg, edges[1].target_node_id)?;
                let to_val = self.ensure_evaluated(asg, edges[2].target_node_id)?;

                match (str_val, from_val, to_val) {
                    (Value::String(s), Value::String(from), Value::String(to)) => {
                        Value::String(s.replace(&from, &to))
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected strings for str-replace".to_string(),
                        ))
                    }
                }
            }

            NodeType::ToString => {
                let val = self.get_single_operand(asg, node)?;
                // Для строк возвращаем как есть (без кавычек), для остальных format_display
                let s = match val {
                    Value::String(s) => s,
                    other => other.format_display(),
                };
                Value::String(s)
            }

            NodeType::ParseInt => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::String(s) => match s.trim().parse::<i64>() {
                        Ok(n) => Value::Int(n),
                        Err(_) => {
                            return Err(ASGError::InvalidOperation(format!(
                                "Cannot parse '{}' as int",
                                s
                            )))
                        }
                    },
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected string for parse-int".to_string(),
                        ))
                    }
                }
            }

            NodeType::ParseFloat => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::String(s) => match s.trim().parse::<f64>() {
                        Ok(f) => Value::Float(f),
                        Err(_) => {
                            return Err(ASGError::InvalidOperation(format!(
                                "Cannot parse '{}' as float",
                                s
                            )))
                        }
                    },
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected string for parse-float".to_string(),
                        ))
                    }
                }
            }

            NodeType::StringTrim => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::String(s) => Value::String(s.trim().to_string()),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected string for str-trim".to_string(),
                        ))
                    }
                }
            }

            NodeType::StringUpper => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::String(s) => Value::String(s.to_uppercase()),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected string for str-upper".to_string(),
                        ))
                    }
                }
            }

            NodeType::StringLower => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::String(s) => Value::String(s.to_lowercase()),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected string for str-lower".to_string(),
                        ))
                    }
                }
            }

            // === Math functions ===
            NodeType::MathSqrt => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Float(f) => Value::Float(f.sqrt()),
                    Value::Int(n) => Value::Float((n as f64).sqrt()),
                    _ => return Err(ASGError::TypeError("Expected number for sqrt".to_string())),
                }
            }

            NodeType::MathSin => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Float(f) => Value::Float(f.sin()),
                    Value::Int(n) => Value::Float((n as f64).sin()),
                    _ => return Err(ASGError::TypeError("Expected number for sin".to_string())),
                }
            }

            NodeType::MathCos => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Float(f) => Value::Float(f.cos()),
                    Value::Int(n) => Value::Float((n as f64).cos()),
                    _ => return Err(ASGError::TypeError("Expected number for cos".to_string())),
                }
            }

            NodeType::MathTan => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Float(f) => Value::Float(f.tan()),
                    Value::Int(n) => Value::Float((n as f64).tan()),
                    _ => return Err(ASGError::TypeError("Expected number for tan".to_string())),
                }
            }

            NodeType::MathAsin => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Float(f) => Value::Float(f.asin()),
                    Value::Int(n) => Value::Float((n as f64).asin()),
                    _ => return Err(ASGError::TypeError("Expected number for asin".to_string())),
                }
            }

            NodeType::MathAcos => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Float(f) => Value::Float(f.acos()),
                    Value::Int(n) => Value::Float((n as f64).acos()),
                    _ => return Err(ASGError::TypeError("Expected number for acos".to_string())),
                }
            }

            NodeType::MathAtan => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Float(f) => Value::Float(f.atan()),
                    Value::Int(n) => Value::Float((n as f64).atan()),
                    _ => return Err(ASGError::TypeError("Expected number for atan".to_string())),
                }
            }

            NodeType::MathExp => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Float(f) => Value::Float(f.exp()),
                    Value::Int(n) => Value::Float((n as f64).exp()),
                    _ => return Err(ASGError::TypeError("Expected number for exp".to_string())),
                }
            }

            NodeType::MathLn => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Float(f) => Value::Float(f.ln()),
                    Value::Int(n) => Value::Float((n as f64).ln()),
                    _ => return Err(ASGError::TypeError("Expected number for ln".to_string())),
                }
            }

            NodeType::MathLog10 => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Float(f) => Value::Float(f.log10()),
                    Value::Int(n) => Value::Float((n as f64).log10()),
                    _ => return Err(ASGError::TypeError("Expected number for log10".to_string())),
                }
            }

            NodeType::MathPow => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::Float(base), Value::Float(exp)) => Value::Float(base.powf(exp)),
                    (Value::Float(base), Value::Int(exp)) => Value::Float(base.powi(exp as i32)),
                    (Value::Int(base), Value::Float(exp)) => Value::Float((base as f64).powf(exp)),
                    (Value::Int(base), Value::Int(exp)) => {
                        if exp >= 0 {
                            Value::Int(base.pow(exp as u32))
                        } else {
                            Value::Float((base as f64).powi(exp as i32))
                        }
                    }
                    _ => return Err(ASGError::TypeError("Expected numbers for pow".to_string())),
                }
            }

            NodeType::MathAbs => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Float(f) => Value::Float(f.abs()),
                    Value::Int(n) => Value::Int(n.abs()),
                    _ => return Err(ASGError::TypeError("Expected number for abs".to_string())),
                }
            }

            NodeType::MathFloor => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Float(f) => Value::Float(f.floor()),
                    Value::Int(n) => Value::Int(n),
                    _ => return Err(ASGError::TypeError("Expected number for floor".to_string())),
                }
            }

            NodeType::MathCeil => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Float(f) => Value::Float(f.ceil()),
                    Value::Int(n) => Value::Int(n),
                    _ => return Err(ASGError::TypeError("Expected number for ceil".to_string())),
                }
            }

            NodeType::MathRound => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Float(f) => Value::Float(f.round()),
                    Value::Int(n) => Value::Int(n),
                    _ => return Err(ASGError::TypeError("Expected number for round".to_string())),
                }
            }

            NodeType::MathMin => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::Int(a), Value::Int(b)) => Value::Int(a.min(b)),
                    (Value::Float(a), Value::Float(b)) => Value::Float(a.min(b)),
                    (Value::Int(a), Value::Float(b)) => Value::Float((a as f64).min(b)),
                    (Value::Float(a), Value::Int(b)) => Value::Float(a.min(b as f64)),
                    _ => return Err(ASGError::TypeError("Expected numbers for min".to_string())),
                }
            }

            NodeType::MathMax => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::Int(a), Value::Int(b)) => Value::Int(a.max(b)),
                    (Value::Float(a), Value::Float(b)) => Value::Float(a.max(b)),
                    (Value::Int(a), Value::Float(b)) => Value::Float((a as f64).max(b)),
                    (Value::Float(a), Value::Int(b)) => Value::Float(a.max(b as f64)),
                    _ => return Err(ASGError::TypeError("Expected numbers for max".to_string())),
                }
            }

            NodeType::MathPi => Value::Float(std::f64::consts::PI),
            NodeType::MathE => Value::Float(std::f64::consts::E),

            // === I/O ===
            NodeType::Print => {
                let arg_edge = node.edges.first().ok_or(ASGError::MissingEdge(
                    node.id,
                    EdgeType::ApplicationArgument,
                ))?;

                let value = self.ensure_evaluated(asg, arg_edge.target_node_id)?;

                // Вывод значения - для строк без кавычек, для остальных format_display
                match &value {
                    Value::String(s) => println!("{}", s),
                    other => println!("{}", other.format_display()),
                }
                Value::Unit
            }

            NodeType::Input => {
                // Показать prompt если есть
                if let Some(edge) = node.edges.first() {
                    let prompt = self.ensure_evaluated(asg, edge.target_node_id)?;
                    if let Value::String(s) = prompt {
                        print!("{}", s);
                        use std::io::Write;
                        std::io::stdout().flush().ok();
                    }
                }

                // Читаем строку
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .map_err(|e| ASGError::InvalidOperation(format!("Input error: {}", e)))?;
                Value::String(input.trim_end().to_string())
            }

            NodeType::InputInt => {
                // Показать prompt если есть
                if let Some(edge) = node.edges.first() {
                    let prompt = self.ensure_evaluated(asg, edge.target_node_id)?;
                    if let Value::String(s) = prompt {
                        print!("{}", s);
                        use std::io::Write;
                        std::io::stdout().flush().ok();
                    }
                }

                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .map_err(|e| ASGError::InvalidOperation(format!("Input error: {}", e)))?;
                let n: i64 = input.trim().parse().map_err(|_| {
                    ASGError::TypeError(format!("Cannot parse '{}' as integer", input.trim()))
                })?;
                Value::Int(n)
            }

            NodeType::InputFloat => {
                // Показать prompt если есть
                if let Some(edge) = node.edges.first() {
                    let prompt = self.ensure_evaluated(asg, edge.target_node_id)?;
                    if let Value::String(s) = prompt {
                        print!("{}", s);
                        use std::io::Write;
                        std::io::stdout().flush().ok();
                    }
                }

                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .map_err(|e| ASGError::InvalidOperation(format!("Input error: {}", e)))?;
                let f: f64 = input.trim().parse().map_err(|_| {
                    ASGError::TypeError(format!("Cannot parse '{}' as float", input.trim()))
                })?;
                Value::Float(f)
            }

            NodeType::ClearScreen => {
                // ANSI escape для очистки экрана
                print!("\x1B[2J\x1B[1;1H");
                use std::io::Write;
                std::io::stdout().flush().ok();
                Value::Unit
            }

            NodeType::ReadFile => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::String(path) => match fs::read_to_string(&path) {
                        Ok(content) => Value::String(content),
                        Err(e) => {
                            return Err(ASGError::InvalidOperation(format!(
                                "Cannot read file '{}': {}",
                                path, e
                            )))
                        }
                    },
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected string path for read-file".to_string(),
                        ))
                    }
                }
            }

            NodeType::WriteFile => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::String(path), Value::String(content)) => {
                        match fs::write(&path, &content) {
                            Ok(_) => Value::Unit,
                            Err(e) => {
                                return Err(ASGError::InvalidOperation(format!(
                                    "Cannot write file '{}': {}",
                                    path, e
                                )))
                            }
                        }
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected (path, content) strings for write-file".to_string(),
                        ))
                    }
                }
            }

            NodeType::AppendFile => {
                let (val1, val2) = self.get_binary_operands(asg, node)?;
                match (val1, val2) {
                    (Value::String(path), Value::String(content)) => {
                        match fs::OpenOptions::new().create(true).append(true).open(&path) {
                            Ok(mut file) => match file.write_all(content.as_bytes()) {
                                Ok(_) => Value::Unit,
                                Err(e) => {
                                    return Err(ASGError::InvalidOperation(format!(
                                        "Cannot append to file '{}': {}",
                                        path, e
                                    )))
                                }
                            },
                            Err(e) => {
                                return Err(ASGError::InvalidOperation(format!(
                                    "Cannot open file '{}': {}",
                                    path, e
                                )))
                            }
                        }
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected (path, content) strings for append-file".to_string(),
                        ))
                    }
                }
            }

            NodeType::FileExists => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::String(path) => Value::Bool(std::path::Path::new(&path).exists()),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected string path for file-exists".to_string(),
                        ))
                    }
                }
            }

            // === Error Handling ===
            NodeType::TryCatch => {
                let try_edge = node
                    .find_edge(EdgeType::TryBody)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::TryBody))?;
                let var_edge = node
                    .find_edge(EdgeType::CatchVariable)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::CatchVariable))?;
                let handler_edge = node
                    .find_edge(EdgeType::CatchHandler)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::CatchHandler))?;

                // Get error variable name
                let var_node = asg
                    .find_node(var_edge.target_node_id)
                    .ok_or(ASGError::NodeNotFound(var_edge.target_node_id))?;
                let error_var_name = var_node.get_name().unwrap_or_default();

                // Try to evaluate the try body
                match self.ensure_evaluated(asg, try_edge.target_node_id) {
                    Ok(Value::Error(msg)) => {
                        // Error was thrown, execute handler
                        let saved_memo = std::mem::take(&mut self.memo);
                        let mut frame = CallFrame::default();
                        frame.locals.insert(error_var_name, Value::Error(msg));
                        frame.memo = saved_memo;
                        self.call_stack.push(frame);

                        let result = self.ensure_evaluated(asg, handler_edge.target_node_id)?;

                        if let Some(popped_frame) = self.call_stack.pop() {
                            self.memo = popped_frame.memo;
                        }
                        result
                    }
                    Ok(val) => val, // No error, return value
                    Err(e) => {
                        // Runtime error, convert to Value::Error and execute handler
                        let saved_memo = std::mem::take(&mut self.memo);
                        let mut frame = CallFrame::default();
                        frame
                            .locals
                            .insert(error_var_name, Value::Error(e.to_string()));
                        frame.memo = saved_memo;
                        self.call_stack.push(frame);

                        let result = self.ensure_evaluated(asg, handler_edge.target_node_id)?;

                        if let Some(popped_frame) = self.call_stack.pop() {
                            self.memo = popped_frame.memo;
                        }
                        result
                    }
                }
            }

            NodeType::Throw => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::String(msg) => Value::Error(msg),
                    _ => Value::Error(format!("{:?}", val)),
                }
            }

            NodeType::IsError => {
                let val = self.get_single_operand(asg, node)?;
                Value::Bool(matches!(val, Value::Error(_)))
            }

            NodeType::ErrorMessage => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::Error(msg) => Value::String(msg),
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected error for error-message".to_string(),
                        ))
                    }
                }
            }

            // === Record ===
            NodeType::Record => {
                let field_data: Vec<_> = node
                    .find_edges(EdgeType::RecordFieldDef)
                    .into_iter()
                    .map(|e| {
                        let field_node = asg.find_node(e.target_node_id);
                        (e.target_node_id, field_node.and_then(|n| n.get_name()))
                    })
                    .collect();
                let mut fields = HashMap::new();
                for (field_id, field_name_opt) in field_data {
                    let field_name = field_name_opt.unwrap_or_default();
                    let field_val = self.ensure_evaluated(asg, field_id)?;
                    fields.insert(field_name, field_val);
                }
                Value::Record(fields)
            }

            NodeType::RecordField => {
                let field_name = node.get_name().ok_or(ASGError::MissingPayload(node.id))?;
                let record_edge = node
                    .find_edge(EdgeType::RecordFieldAccess)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::RecordFieldAccess))?;

                let record_val = self.ensure_evaluated(asg, record_edge.target_node_id)?;
                match record_val {
                    Value::Record(fields) => {
                        fields
                            .get(&field_name)
                            .cloned()
                            .ok_or(ASGError::InvalidOperation(format!(
                                "Field {} not found",
                                field_name
                            )))?
                    }
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected record for field access".to_string(),
                        ))
                    }
                }
            }

            // === Pattern Matching ===
            NodeType::Match => {
                let subject_edge = node
                    .find_edge(EdgeType::MatchSubject)
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::MatchSubject))?;

                let subject_val = self.ensure_evaluated(asg, subject_edge.target_node_id)?;

                // Get all match arms
                let arm_edges: Vec<_> = node.find_edges(EdgeType::ApplicationArgument);

                for arm_edge in arm_edges {
                    let arm_node = asg
                        .find_node(arm_edge.target_node_id)
                        .ok_or(ASGError::NodeNotFound(arm_edge.target_node_id))?
                        .clone();

                    if arm_node.node_type != NodeType::MatchArm {
                        continue;
                    }

                    let pattern_edge = arm_node
                        .find_edge(EdgeType::MatchPattern)
                        .ok_or(ASGError::MissingEdge(arm_node.id, EdgeType::MatchPattern))?;
                    let body_edge = arm_node
                        .find_edge(EdgeType::MatchBody)
                        .ok_or(ASGError::MissingEdge(arm_node.id, EdgeType::MatchBody))?;

                    // Check if pattern matches
                    let pattern_node = asg
                        .find_node(pattern_edge.target_node_id)
                        .ok_or(ASGError::NodeNotFound(pattern_edge.target_node_id))?
                        .clone();

                    let (matches, bindings) =
                        self.match_pattern(asg, &pattern_node, &subject_val)?;

                    if matches {
                        // Pattern matched! Evaluate body with bindings
                        let result = if !bindings.is_empty() {
                            let saved_memo = std::mem::take(&mut self.memo);
                            let mut frame = CallFrame::default();
                            for (name, val) in bindings {
                                frame.locals.insert(name, val);
                            }
                            frame.memo = saved_memo;
                            self.call_stack.push(frame);

                            let res = self.ensure_evaluated(asg, body_edge.target_node_id)?;

                            if let Some(popped_frame) = self.call_stack.pop() {
                                self.memo = popped_frame.memo;
                            }
                            res
                        } else {
                            self.ensure_evaluated(asg, body_edge.target_node_id)?
                        };
                        self.memo.insert(node.id, result);
                        return Ok(());
                    }
                }

                return Err(ASGError::InvalidOperation(
                    "No matching pattern found".to_string(),
                ));
            }

            NodeType::MatchArm => {
                // MatchArm nodes are processed by Match, should not be evaluated directly
                Value::Unit
            }

            // === Modules ===
            NodeType::Module => {
                // (module name body...)
                // Просто выполняем все body выражения
                let content_edges: Vec<_> = node.find_edges(EdgeType::ModuleContent);
                let mut last_value = Value::Unit;
                for edge in content_edges {
                    last_value = self.ensure_evaluated(asg, edge.target_node_id)?;
                }
                last_value
            }

            NodeType::Import => {
                // (import "path/to/file.asg") или (import "path" as alias)
                let payload_str = node.get_name().unwrap_or_default();

                // Разбираем payload: path|alias или просто path
                let parts: Vec<&str> = payload_str.split('|').collect();
                let path = parts[0];
                let _alias = parts.get(1).copied(); // alias пока не используем

                // Читаем и парсим файл
                let source = match fs::read_to_string(path) {
                    Ok(content) => content,
                    Err(e) => {
                        return Err(ASGError::InvalidOperation(format!(
                            "Cannot import '{}': {}",
                            path, e
                        )));
                    }
                };

                // Парсим файл
                let (imported_asg, root_ids) = match parse(&source) {
                    Ok((asg, ids)) => (asg, ids),
                    Err(e) => {
                        return Err(ASGError::InvalidOperation(format!(
                            "Parse error in '{}': {:?}",
                            path, e
                        )));
                    }
                };

                // Запоминаем какие функции были до импорта
                let functions_before: std::collections::HashSet<String> =
                    self.functions.keys().cloned().collect();

                // Выполняем все top-level выражения
                // Сохраняем текущее состояние memo
                let saved_memo = std::mem::take(&mut self.memo);

                for root_id in &root_ids {
                    self.ensure_evaluated(&imported_asg, *root_id)?;
                }

                // Обновляем импортированные функции, добавляя ASG
                let new_functions: Vec<String> = self
                    .functions
                    .keys()
                    .filter(|k| !functions_before.contains(*k))
                    .cloned()
                    .collect();

                for name in new_functions {
                    if let Some((params, body_id, _)) = self.functions.remove(&name) {
                        self.functions
                            .insert(name, (params, body_id, Some(imported_asg.clone())));
                    }
                }

                // Восстанавливаем memo (импортированные определения остаются в self.functions и self.variables)
                self.memo = saved_memo;

                Value::Unit
            }

            NodeType::Export => {
                // Export пока просто возвращает Unit (для полноценной модульной системы нужен отдельный механизм)
                Value::Unit
            }

            // === HTML Generation ===
            NodeType::HtmlElement => {
                let raw_tag = node.get_name().unwrap_or_else(|| "div".to_string());
                // Преобразуем html-* теги в обычные HTML теги
                let tag_name = if raw_tag.starts_with("html-") {
                    raw_tag[5..].to_string()
                } else {
                    raw_tag
                };
                let mut html = format!("<{}", tag_name);

                // Собираем children
                let children: Vec<_> = node.find_edges(EdgeType::ApplicationArgument);
                let mut content = String::new();

                for child_edge in children {
                    let child_val = self.ensure_evaluated(asg, child_edge.target_node_id)?;
                    match child_val {
                        Value::String(s) => {
                            // Проверяем, атрибут ли это (начинается с @)
                            if s.starts_with("@") {
                                // @attr=value формат
                                if let Some((attr, val)) = s[1..].split_once('=') {
                                    html.push_str(&format!(" {}=\"{}\"", attr, val));
                                }
                            } else if s.starts_with('<') {
                                // Вложенный HTML
                                content.push_str(&s);
                            } else {
                                // Текстовый контент
                                content.push_str(&s);
                            }
                        }
                        _ => content.push_str(&child_val.format_display()),
                    }
                }

                // Самозакрывающиеся теги
                let self_closing = matches!(
                    tag_name.as_str(),
                    "br" | "hr" | "img" | "input" | "meta" | "link"
                );
                if self_closing {
                    html.push_str(" />");
                } else {
                    html.push('>');
                    html.push_str(&content);
                    html.push_str(&format!("</{}>", tag_name));
                }

                Value::String(html)
            }

            NodeType::HtmlAttr => {
                // Не должно напрямую вызываться
                Value::Unit
            }

            // === JSON ===
            NodeType::JsonEncode => {
                let val = self.get_single_operand(asg, node)?;
                let json = self.value_to_json(&val);
                Value::String(json)
            }

            NodeType::JsonDecode => {
                let val = self.get_single_operand(asg, node)?;
                match val {
                    Value::String(s) => match serde_json::from_str::<serde_json::Value>(&s) {
                        Ok(json) => self.json_to_value(json),
                        Err(e) => {
                            return Err(ASGError::InvalidOperation(format!(
                                "JSON parse error: {}",
                                e
                            )))
                        }
                    },
                    _ => {
                        return Err(ASGError::TypeError(
                            "Expected string for json-decode".to_string(),
                        ))
                    }
                }
            }

            // === HTTP Server (requires 'web' feature) ===
            #[cfg(feature = "web")]
            NodeType::HttpServe => {
                let (port_val, handler_val) = self.get_binary_operands(asg, node)?;
                match port_val {
                    Value::Int(port) => {
                        #[allow(unused_imports)]
                        use std::io::Read;
                        use tiny_http::{Method, Response, Server};

                        let server = Server::http(format!("0.0.0.0:{}", port)).map_err(|e| {
                            ASGError::InvalidOperation(format!("Cannot start HTTP server: {}", e))
                        })?;

                        println!("╔════════════════════════════════════════╗");
                        println!("║   ASG HTTP Server                  ║");
                        println!("║   http://localhost:{}                 ║", port);
                        println!("║   Press Ctrl+C to stop                 ║");
                        println!("╚════════════════════════════════════════╝");

                        for mut request in server.incoming_requests() {
                            let method = match request.method() {
                                Method::Get => "GET",
                                Method::Post => "POST",
                                Method::Put => "PUT",
                                Method::Delete => "DELETE",
                                _ => "OTHER",
                            };
                            let path = request.url().to_string();

                            // Read body for POST requests
                            let mut body_content = String::new();
                            request.as_reader().read_to_string(&mut body_content).ok();

                            // Create request dict for handler
                            let mut req_dict = HashMap::new();
                            req_dict
                                .insert("method".to_string(), Value::String(method.to_string()));
                            req_dict.insert("path".to_string(), Value::String(path.clone()));
                            req_dict.insert("body".to_string(), Value::String(body_content));

                            // Parse query params
                            let mut params = HashMap::new();
                            if let Some(query) = path.split('?').nth(1) {
                                for pair in query.split('&') {
                                    if let Some((k, v)) = pair.split_once('=') {
                                        params.insert(k.to_string(), Value::String(v.to_string()));
                                    }
                                }
                            }
                            req_dict.insert("params".to_string(), Value::Dict(params));

                            let req_value = Value::Dict(req_dict);

                            // Call handler with request
                            let response_val =
                                self.call_function_value(asg, handler_val.clone(), req_value)?;

                            // Process response
                            let (status_code, content_type, body): (u32, String, String) =
                                match response_val {
                                    Value::String(s) => (200, "text/html".to_string(), s),
                                    Value::Dict(d) => {
                                        let status = match d.get("status") {
                                            Some(Value::Int(s)) => *s as u32,
                                            _ => 200,
                                        };
                                        let ctype = match d.get("content-type") {
                                            Some(Value::String(s)) => s.clone(),
                                            _ => "text/html".to_string(),
                                        };
                                        let body = match d.get("body") {
                                            Some(Value::String(s)) => s.clone(),
                                            Some(v) => v.format_display(),
                                            _ => "".to_string(),
                                        };
                                        (status, ctype, body)
                                    }
                                    other => {
                                        (200, "text/plain".to_string(), other.format_display())
                                    }
                                };

                            println!(
                                "[{}] {} {} -> {}",
                                method,
                                path.split('?').next().unwrap_or(&path),
                                status_code,
                                body.len()
                            );

                            let response = Response::from_string(body)
                                .with_status_code(status_code as u16)
                                .with_header(
                                    format!("Content-Type: {}", content_type)
                                        .parse::<tiny_http::Header>()
                                        .unwrap(),
                                );
                            request.respond(response).ok();
                        }
                        Value::Unit
                    }
                    _ => return Err(ASGError::TypeError("Expected integer port".to_string())),
                }
            }

            #[cfg(not(feature = "web"))]
            NodeType::HttpServe => {
                return Err(ASGError::InvalidOperation(
                    "HTTP server requires 'web' feature. Recompile with: cargo build --features web".to_string()
                ));
            }

            NodeType::HttpResponse => {
                // Создание HTTP response (используется внутри handler)
                let edges: Vec<_> = node.find_edges(EdgeType::ApplicationArgument);
                if edges.len() != 3 {
                    return Err(ASGError::InvalidOperation(
                        "http-response requires 3 arguments".to_string(),
                    ));
                }
                let status = self.ensure_evaluated(asg, edges[0].target_node_id)?;
                let _headers = self.ensure_evaluated(asg, edges[1].target_node_id)?;
                let body = self.ensure_evaluated(asg, edges[2].target_node_id)?;

                // Возвращаем как Dict
                let mut response = HashMap::new();
                response.insert("status".to_string(), status);
                response.insert("body".to_string(), body);
                Value::Dict(response)
            }

            // === Native GUI (requires 'gui' feature) ===
            #[cfg(feature = "gui")]
            NodeType::GuiRun => {
                let window_val = self.get_single_operand(asg, node)?;

                // Check if it's the special calculator command
                let is_calculator = matches!(&window_val, Value::String(s) if s == "calculator");

                if is_calculator {
                    crate::gui::run_calculator()
                        .map_err(|e| ASGError::InvalidOperation(format!("GUI error: {}", e)))?;
                } else {
                    // Convert Value to Widget tree
                    let widgets = match crate::gui::ASGGuiApp::value_to_widget(&window_val) {
                        Some(w) => vec![w],
                        None => Vec::new(),
                    };

                    let title = match &window_val {
                        Value::Dict(d) => match d.get("title") {
                            Some(Value::String(s)) => s.clone(),
                            _ => "ASG App".to_string(),
                        },
                        _ => "ASG App".to_string(),
                    };

                    crate::gui::run_gui(&title, widgets)
                        .map_err(|e| ASGError::InvalidOperation(format!("GUI error: {}", e)))?;
                }

                Value::Unit
            }

            #[cfg(not(feature = "gui"))]
            NodeType::GuiRun => {
                return Err(ASGError::InvalidOperation(
                    "Native GUI requires 'gui' feature. Recompile with: cargo build --features gui"
                        .to_string(),
                ));
            }

            NodeType::GuiWindow
            | NodeType::GuiButton
            | NodeType::GuiTextField
            | NodeType::GuiLabel
            | NodeType::GuiVBox
            | NodeType::GuiHBox
            | NodeType::GuiCanvas => {
                // GUI widgets - возвращаем описание для gui-run
                let mut widget = HashMap::new();
                widget.insert(
                    "type".to_string(),
                    Value::String(format!("{:?}", node.node_type)),
                );

                let children: Vec<_> = node
                    .find_edges(EdgeType::ApplicationArgument)
                    .into_iter()
                    .filter_map(|e| self.ensure_evaluated(asg, e.target_node_id).ok())
                    .collect();
                widget.insert("children".to_string(), Value::Array(children));

                Value::Dict(widget)
            }

            // По умолчанию — Unit
            _ => Value::Unit,
        };

        self.memo.insert(node.id, result_value);
        Ok(())
    }

    /// Вычислить узел если не в кеше, и вернуть значение.
    /// Использует stacker для автоматического расширения стека при глубокой рекурсии.
    fn ensure_evaluated(&mut self, asg: &ASG, node_id: NodeID) -> ASGResult<Value> {
        // Предотвращаем stack overflow при глубокой рекурсии
        // 256KB red zone, 8MB stack growth
        stacker::maybe_grow(256 * 1024, 8 * 1024 * 1024, || {
            if self.memo.contains_key(&node_id) {
                return Ok(self.memo.get(&node_id).unwrap().clone());
            }
            let node = asg
                .find_node(node_id)
                .ok_or(ASGError::NodeNotFound(node_id))?
                .clone();
            self.eval_node(asg, &node)?;
            Ok(self.memo.get(&node_id).unwrap().clone())
        })
    }

    /// Проверить соответствие паттерна значению.
    /// Возвращает (matches: bool, bindings: Vec<(name, value)>)
    fn match_pattern(
        &mut self,
        asg: &ASG,
        pattern_node: &Node,
        subject: &Value,
    ) -> ASGResult<(bool, Vec<(String, Value)>)> {
        match pattern_node.node_type {
            // Wildcard pattern: always matches, no bindings
            NodeType::VarRef => {
                let name = pattern_node.get_name().unwrap_or_default();
                if name == "_" {
                    // Wildcard
                    Ok((true, vec![]))
                } else {
                    // Variable binding
                    Ok((true, vec![(name, subject.clone())]))
                }
            }

            // Literal patterns
            NodeType::LiteralInt => {
                if let Some(bytes) = &pattern_node.payload {
                    if bytes.len() >= 8 {
                        let pattern_val = i64::from_le_bytes(bytes[..8].try_into().unwrap());
                        match subject {
                            Value::Int(n) => Ok((*n == pattern_val, vec![])),
                            _ => Ok((false, vec![])),
                        }
                    } else {
                        Ok((false, vec![]))
                    }
                } else {
                    Ok((false, vec![]))
                }
            }

            NodeType::LiteralFloat => {
                if let Some(bytes) = &pattern_node.payload {
                    if bytes.len() >= 8 {
                        let pattern_val = f64::from_le_bytes(bytes[..8].try_into().unwrap());
                        match subject {
                            Value::Float(f) => {
                                Ok(((*f - pattern_val).abs() < f64::EPSILON, vec![]))
                            }
                            _ => Ok((false, vec![])),
                        }
                    } else {
                        Ok((false, vec![]))
                    }
                } else {
                    Ok((false, vec![]))
                }
            }

            NodeType::LiteralBool => {
                if let Some(bytes) = &pattern_node.payload {
                    let pattern_val = bytes.first().map(|&b| b != 0).unwrap_or(false);
                    match subject {
                        Value::Bool(b) => Ok((*b == pattern_val, vec![])),
                        _ => Ok((false, vec![])),
                    }
                } else {
                    Ok((false, vec![]))
                }
            }

            NodeType::LiteralString => {
                if let Some(bytes) = &pattern_node.payload {
                    let pattern_val = String::from_utf8_lossy(bytes).to_string();
                    match subject {
                        Value::String(s) => Ok((s == &pattern_val, vec![])),
                        _ => Ok((false, vec![])),
                    }
                } else {
                    Ok((false, vec![]))
                }
            }

            NodeType::LiteralUnit => match subject {
                Value::Unit => Ok((true, vec![])),
                _ => Ok((false, vec![])),
            },

            // Array pattern matching
            NodeType::Array => match subject {
                Value::Array(arr) => {
                    let pattern_elements: Vec<_> = pattern_node
                        .find_edges(EdgeType::ArrayElement)
                        .into_iter()
                        .map(|e| e.target_node_id)
                        .collect();

                    if pattern_elements.len() != arr.len() {
                        return Ok((false, vec![]));
                    }

                    let mut all_bindings = vec![];
                    for (i, elem_id) in pattern_elements.iter().enumerate() {
                        let elem_node = asg
                            .find_node(*elem_id)
                            .ok_or(ASGError::NodeNotFound(*elem_id))?
                            .clone();
                        let (matches, bindings) = self.match_pattern(asg, &elem_node, &arr[i])?;
                        if !matches {
                            return Ok((false, vec![]));
                        }
                        all_bindings.extend(bindings);
                    }
                    Ok((true, all_bindings))
                }
                _ => Ok((false, vec![])),
            },

            // Default: evaluate pattern and compare
            _ => {
                let pattern_val = self.ensure_evaluated(asg, pattern_node.id)?;
                Ok((self.values_equal(&pattern_val, subject), vec![]))
            }
        }
    }

    /// Проверить равенство двух значений.
    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => x == y,
            (Value::Float(x), Value::Float(y)) => (x - y).abs() < f64::EPSILON,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Unit, Value::Unit) => true,
            (Value::Array(x), Value::Array(y)) => {
                x.len() == y.len() && x.iter().zip(y.iter()).all(|(a, b)| self.values_equal(a, b))
            }
            _ => false,
        }
    }

    /// Получить два операнда для бинарной операции.
    fn get_binary_operands(&mut self, asg: &ASG, node: &Node) -> ASGResult<(Value, Value)> {
        let edges: Vec<_> = node
            .edges
            .iter()
            .filter(|e| {
                e.edge_type == EdgeType::ApplicationArgument
                    || e.edge_type == EdgeType::FirstOperand
                    || e.edge_type == EdgeType::SecondOperand
            })
            .map(|e| e.target_node_id)
            .collect();

        if edges.len() < 2 {
            return Err(ASGError::MissingEdge(
                node.id,
                EdgeType::ApplicationArgument,
            ));
        }

        let val1 = self.ensure_evaluated(asg, edges[0])?;
        let val2 = self.ensure_evaluated(asg, edges[1])?;

        Ok((val1, val2))
    }

    /// Получить единственный операнд.
    fn get_single_operand(&mut self, asg: &ASG, node: &Node) -> ASGResult<Value> {
        let edge = node.edges.first().ok_or(ASGError::MissingEdge(
            node.id,
            EdgeType::ApplicationArgument,
        ))?;
        self.ensure_evaluated(asg, edge.target_node_id)
    }

    /// Получить первый операнд (FirstOperand edge).
    fn get_first_operand(&mut self, asg: &ASG, node: &Node) -> ASGResult<Value> {
        let edge = node
            .find_edge(EdgeType::FirstOperand)
            .ok_or(ASGError::MissingEdge(node.id, EdgeType::FirstOperand))?;
        self.ensure_evaluated(asg, edge.target_node_id)
    }

    /// Получить второй операнд (SecondOperand edge).
    fn get_second_operand(&mut self, asg: &ASG, node: &Node) -> ASGResult<Value> {
        let edge = node
            .find_edge(EdgeType::SecondOperand)
            .ok_or(ASGError::MissingEdge(node.id, EdgeType::SecondOperand))?;
        self.ensure_evaluated(asg, edge.target_node_id)
    }

    /// Convert Value to JSON string.
    fn value_to_json(&self, val: &Value) -> String {
        match val {
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            Value::Unit => "null".to_string(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| self.value_to_json(v)).collect();
                format!("[{}]", items.join(","))
            }
            Value::Dict(d) => {
                let items: Vec<String> = d
                    .iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, self.value_to_json(v)))
                    .collect();
                format!("{{{}}}", items.join(","))
            }
            Value::Record(fields) => {
                let items: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, self.value_to_json(v)))
                    .collect();
                format!("{{{}}}", items.join(","))
            }
            Value::Error(msg) => format!("{{\"error\":\"{}\"}}", msg),
            _ => "null".to_string(),
        }
    }

    /// Convert JSON value to ASG Value.
    fn json_to_value(&self, json: serde_json::Value) -> Value {
        match json {
            serde_json::Value::Null => Value::Unit,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Int(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Float(f)
                } else {
                    Value::Unit
                }
            }
            serde_json::Value::String(s) => Value::String(s),
            serde_json::Value::Array(arr) => {
                Value::Array(arr.into_iter().map(|v| self.json_to_value(v)).collect())
            }
            serde_json::Value::Object(map) => {
                let mut dict = HashMap::new();
                for (k, v) in map {
                    dict.insert(k, self.json_to_value(v));
                }
                Value::Dict(dict)
            }
        }
    }

    /// Вызвать функцию (Function или ComposedFunction) с одним аргументом.
    fn call_function_value(&mut self, asg: &ASG, fn_val: Value, arg: Value) -> ASGResult<Value> {
        match fn_val {
            Value::Function {
                params,
                body_id,
                captured,
            } => {
                let saved_memo = std::mem::take(&mut self.memo);
                let mut frame = CallFrame::default();
                for (name, val) in &captured {
                    frame.locals.insert(name.clone(), val.clone());
                }
                if !params.is_empty() {
                    frame.locals.insert(params[0].clone(), arg);
                }
                frame.memo = saved_memo;
                self.call_stack.push(frame);

                let result = self.ensure_evaluated(asg, body_id)?;

                if let Some(popped_frame) = self.call_stack.pop() {
                    self.memo = popped_frame.memo;
                }
                Ok(result)
            }
            Value::ComposedFunction(fns) => {
                // Применяем все функции последовательно
                let mut current = arg;
                for f in fns {
                    current = self.call_function_value(asg, f, current)?;
                }
                Ok(current)
            }
            _ => Err(ASGError::TypeError("Expected function".to_string())),
        }
    }

    /// Материализовать n элементов из lazy sequence.
    fn take_from_lazy(
        &mut self,
        asg: &ASG,
        mut kind: LazySeqKind,
        n: usize,
    ) -> ASGResult<Vec<Value>> {
        let mut result = Vec::with_capacity(n);

        for _ in 0..n {
            match self.next_lazy_element(asg, &mut kind)? {
                Some(val) => result.push(val),
                None => break,
            }
        }

        Ok(result)
    }

    /// Получить следующий элемент из lazy sequence.
    fn next_lazy_element(&mut self, asg: &ASG, kind: &mut LazySeqKind) -> ASGResult<Option<Value>> {
        match kind {
            LazySeqKind::Iterate { func, current } => {
                let val = (**current).clone();
                let next = self.call_function_value(asg, (**func).clone(), val.clone())?;
                *current = Box::new(next);
                Ok(Some(val))
            }
            LazySeqKind::Repeat(val) => Ok(Some((**val).clone())),
            LazySeqKind::Cycle { arr, index } => {
                if arr.is_empty() {
                    return Ok(None);
                }
                let val = arr[*index].clone();
                *index = (*index + 1) % arr.len();
                Ok(Some(val))
            }
            LazySeqKind::Range { current, end, step } => {
                if (*step > 0 && *current >= *end) || (*step < 0 && *current <= *end) || *step == 0
                {
                    return Ok(None);
                }
                let val = Value::Int(*current);
                *current += *step;
                Ok(Some(val))
            }
            LazySeqKind::Map { func, source } => match self.next_lazy_element(asg, source)? {
                Some(val) => {
                    let mapped = self.call_function_value(asg, (**func).clone(), val)?;
                    Ok(Some(mapped))
                }
                None => Ok(None),
            },
            LazySeqKind::Filter { func, source } => {
                // Keep getting elements until we find one that passes the filter
                loop {
                    match self.next_lazy_element(asg, source)? {
                        Some(val) => {
                            let result =
                                self.call_function_value(asg, (**func).clone(), val.clone())?;
                            match result {
                                Value::Bool(true) => return Ok(Some(val)),
                                Value::Bool(false) => continue,
                                _ => return Ok(Some(val)), // Non-bool treated as true
                            }
                        }
                        None => return Ok(None),
                    }
                }
            }
        }
    }

    // === REPL Helper Methods ===

    /// Получить все переменные для REPL команды :env
    pub fn get_variables(&self) -> &HashMap<String, Value> {
        &self.variables
    }

    /// Получить все функции для REPL команды :funcs
    /// Возвращает HashMap с именем функции и её Value представлением
    pub fn get_functions(&self) -> HashMap<String, Value> {
        self.functions
            .iter()
            .map(|(name, (params, body_id, _))| {
                (
                    name.clone(),
                    Value::Function {
                        params: params.clone(),
                        body_id: *body_id,
                        captured: HashMap::new(),
                    },
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asg::{Edge, Node, ASG};

    #[test]
    fn test_interpret_add() {
        let mut asg = ASG::new();

        // Создаём 5 + 8
        asg.add_node(Node::new(
            1,
            NodeType::LiteralInt,
            Some(5i64.to_le_bytes().to_vec()),
        ));
        asg.add_node(Node::new(
            2,
            NodeType::LiteralInt,
            Some(8i64.to_le_bytes().to_vec()),
        ));
        asg.add_node(Node::with_edges(
            3,
            NodeType::BinaryOperation,
            None,
            vec![
                Edge::new(EdgeType::ApplicationArgument, 1),
                Edge::new(EdgeType::ApplicationArgument, 2),
            ],
        ));

        let mut interpreter = Interpreter::new();
        let result = interpreter.execute(&asg, 3).unwrap();

        match result {
            Value::Int(val) => assert_eq!(val, 13),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_interpret_comparison() {
        let mut asg = ASG::new();

        // 5 < 10
        asg.add_node(Node::new(
            1,
            NodeType::LiteralInt,
            Some(5i64.to_le_bytes().to_vec()),
        ));
        asg.add_node(Node::new(
            2,
            NodeType::LiteralInt,
            Some(10i64.to_le_bytes().to_vec()),
        ));
        asg.add_node(Node::with_edges(
            3,
            NodeType::Lt,
            None,
            vec![
                Edge::new(EdgeType::ApplicationArgument, 1),
                Edge::new(EdgeType::ApplicationArgument, 2),
            ],
        ));

        let mut interpreter = Interpreter::new();
        let result = interpreter.execute(&asg, 3).unwrap();

        match result {
            Value::Bool(val) => assert!(val),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_json_encode() {
        use crate::parser::parse_expr;

        let (asg, root) = parse_expr(r#"(json-encode (dict "name" "John" "age" 30))"#).unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.execute(&asg, root).unwrap();

        match result {
            Value::String(s) => {
                assert!(s.contains("\"name\":\"John\"") || s.contains("\"age\":30"));
            }
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_json_decode() {
        use crate::parser::parse_expr;

        let (asg, root) = parse_expr(r#"(json-decode "{\"x\": 42}")"#).unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.execute(&asg, root).unwrap();

        match result {
            Value::Dict(d) => {
                assert_eq!(d.get("x"), Some(&Value::Int(42)));
            }
            _ => panic!("Expected Dict"),
        }
    }

    #[test]
    fn test_html_generation() {
        use crate::parser::parse_expr;

        let (asg, root) = parse_expr(r#"(div "Hello")"#).unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.execute(&asg, root).unwrap();

        match result {
            Value::String(s) => {
                assert!(s.contains("<div>"));
                assert!(s.contains("Hello"));
                assert!(s.contains("</div>"));
            }
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_html_with_attributes() {
        use crate::parser::parse_expr;

        let (asg, root) = parse_expr(r#"(div "@class=container" "Content")"#).unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.execute(&asg, root).unwrap();

        match result {
            Value::String(s) => {
                assert!(s.contains("class=\"container\""));
                assert!(s.contains("Content"));
            }
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_variadic_add() {
        use crate::parser::parse_expr;

        let (asg, root) = parse_expr("(+ 1 2 3 4 5)").unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.execute(&asg, root).unwrap();

        match result {
            Value::Int(val) => assert_eq!(val, 15),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_variadic_mul() {
        use crate::parser::parse_expr;

        let (asg, root) = parse_expr("(* 2 3 4)").unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.execute(&asg, root).unwrap();

        match result {
            Value::Int(val) => assert_eq!(val, 24),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_first() {
        use crate::parser::parse_expr;

        let (asg, root) = parse_expr("(first (array 1 2 3))").unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.execute(&asg, root).unwrap();
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_last() {
        use crate::parser::parse_expr;

        let (asg, root) = parse_expr("(last (array 10 20 30))").unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.execute(&asg, root).unwrap();
        assert_eq!(result, Value::Int(30));
    }

    #[test]
    fn test_clear_screen() {
        use crate::parser::parse_expr;

        let (asg, root) = parse_expr("(clear-screen)").unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.execute(&asg, root).unwrap();
        assert_eq!(result, Value::Unit);
    }

    #[test]
    fn test_dict_operations() {
        use crate::parser::parse_expr;

        // Create dict
        let (asg, root) = parse_expr(r#"(dict "a" 1 "b" 2)"#).unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.execute(&asg, root).unwrap();

        match result {
            Value::Dict(d) => {
                assert_eq!(d.get("a"), Some(&Value::Int(1)));
                assert_eq!(d.get("b"), Some(&Value::Int(2)));
            }
            _ => panic!("Expected Dict"),
        }
    }
}
