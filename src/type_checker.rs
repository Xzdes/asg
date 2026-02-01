//! Модуль `type_checker`
//!
//! Полноценный механизм проверки и вывода типов для ASG.
//!
//! Реализует:
//! - Алгоритм унификации типов
//! - Вывод типов на основе Hindley-Milner
//! - Проверку корректности типов в ASG

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::asg::{Node, NodeID, ASG};
use crate::error::{ASGError, ASGResult};
use crate::nodecodes::{EdgeType, NodeType};
use crate::types::{SynType, SynTypeError};

// === Генерация уникальных переменных типа ===

static TYPE_VAR_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Генерация свежей переменной типа.
fn fresh_type_var() -> SynType {
    let id = TYPE_VAR_COUNTER.fetch_add(1, Ordering::SeqCst);
    SynType::TypeVariable(format!("t{}", id))
}

/// Сброс счётчика (для тестов).
#[allow(dead_code)]
pub fn reset_type_var_counter() {
    TYPE_VAR_COUNTER.store(0, Ordering::SeqCst);
}

// === Подстановка ===

/// Подстановка: отображение переменных типа на конкретные типы.
#[derive(Debug, Clone, Default)]
pub struct Substitution {
    mappings: HashMap<String, SynType>,
}

impl Substitution {
    /// Создать пустую подстановку.
    pub fn new() -> Self {
        Self::default()
    }

    /// Добавить подстановку.
    pub fn insert(&mut self, var: String, ty: SynType) {
        self.mappings.insert(var, ty);
    }

    /// Проверить наличие переменной.
    pub fn contains(&self, var: &str) -> bool {
        self.mappings.contains_key(var)
    }

    /// Применить подстановку к типу.
    pub fn apply(&self, ty: &SynType) -> SynType {
        match ty {
            SynType::TypeVariable(name) => {
                if let Some(resolved) = self.mappings.get(name) {
                    // Рекурсивно применяем подстановку
                    self.apply(resolved)
                } else {
                    ty.clone()
                }
            }
            SynType::Function {
                parameters,
                return_type,
            } => SynType::Function {
                parameters: parameters.iter().map(|p| self.apply(p)).collect(),
                return_type: Box::new(self.apply(return_type)),
            },
            SynType::ForAll { type_params, body } => SynType::ForAll {
                type_params: type_params.clone(),
                body: Box::new(self.apply(body)),
            },
            SynType::Record(fields) => SynType::Record(
                fields
                    .iter()
                    .map(|(n, t)| (n.clone(), self.apply(t)))
                    .collect(),
            ),
            SynType::Linear(inner) => SynType::Linear(Box::new(self.apply(inner))),
            SynType::SharedRef(inner) => SynType::SharedRef(Box::new(self.apply(inner))),
            SynType::MutableRef(inner) => SynType::MutableRef(Box::new(self.apply(inner))),
            SynType::Result { ok, err } => SynType::Result {
                ok: Box::new(self.apply(ok)),
                err: Box::new(self.apply(err)),
            },
            SynType::ErrorUnion(a, b) => {
                SynType::ErrorUnion(Box::new(self.apply(a)), Box::new(self.apply(b)))
            }
            SynType::ADT { name, variants } => SynType::ADT {
                name: name.clone(),
                variants: variants
                    .iter()
                    .map(|(n, types)| (n.clone(), types.iter().map(|t| self.apply(t)).collect()))
                    .collect(),
            },
            // Базовые типы не меняются
            _ => ty.clone(),
        }
    }

    /// Композиция подстановок: self ∘ other
    pub fn compose(&self, other: &Substitution) -> Substitution {
        let mut result = Substitution::new();

        // Применяем self к каждому отображению в other
        for (k, v) in &other.mappings {
            result.insert(k.clone(), self.apply(v));
        }

        // Добавляем отображения из self, которых нет в other
        for (k, v) in &self.mappings {
            if !result.mappings.contains_key(k) {
                result.insert(k.clone(), v.clone());
            }
        }

        result
    }
}

// === Унификация ===

/// Проверка вхождения переменной в тип (occurs check).
/// Предотвращает бесконечные типы вроде t = List<t>.
fn occurs_check(var: &str, ty: &SynType) -> bool {
    match ty {
        SynType::TypeVariable(name) => name == var,
        SynType::Function {
            parameters,
            return_type,
        } => parameters.iter().any(|p| occurs_check(var, p)) || occurs_check(var, return_type),
        SynType::ForAll { body, .. } => occurs_check(var, body),
        SynType::Record(fields) => fields.iter().any(|(_, t)| occurs_check(var, t)),
        SynType::Linear(inner) | SynType::SharedRef(inner) | SynType::MutableRef(inner) => {
            occurs_check(var, inner)
        }
        SynType::Result { ok, err } => occurs_check(var, ok) || occurs_check(var, err),
        SynType::ErrorUnion(a, b) => occurs_check(var, a) || occurs_check(var, b),
        SynType::ADT { variants, .. } => variants
            .iter()
            .any(|(_, types)| types.iter().any(|t| occurs_check(var, t))),
        _ => false,
    }
}

/// Унификация двух типов.
/// Возвращает подстановку, которая делает типы равными.
pub fn unify(t1: &SynType, t2: &SynType) -> Result<Substitution, SynTypeError> {
    match (t1, t2) {
        // Одинаковые базовые типы
        (SynType::Int, SynType::Int)
        | (SynType::Float, SynType::Float)
        | (SynType::Bool, SynType::Bool)
        | (SynType::String, SynType::String)
        | (SynType::Unit, SynType::Unit) => Ok(Substitution::new()),

        // Переменная типа унифицируется с любым типом
        (SynType::TypeVariable(name), other) | (other, SynType::TypeVariable(name)) => {
            // Проверяем, не унифицируем ли переменную саму с собой
            if let SynType::TypeVariable(other_name) = other {
                if name == other_name {
                    return Ok(Substitution::new());
                }
            }

            // Occurs check
            if occurs_check(name, other) {
                return Err(SynTypeError::Constraint(format!(
                    "Infinite type: {} occurs in {:?}",
                    name, other
                )));
            }

            let mut subst = Substitution::new();
            subst.insert(name.clone(), other.clone());
            Ok(subst)
        }

        // Функции
        (
            SynType::Function {
                parameters: p1,
                return_type: r1,
            },
            SynType::Function {
                parameters: p2,
                return_type: r2,
            },
        ) => {
            if p1.len() != p2.len() {
                return Err(SynTypeError::Mismatch {
                    expected: t1.clone(),
                    found: t2.clone(),
                });
            }

            let mut subst = Substitution::new();

            // Унифицируем параметры
            for (param1, param2) in p1.iter().zip(p2.iter()) {
                let s = unify(&subst.apply(param1), &subst.apply(param2))?;
                subst = subst.compose(&s);
            }

            // Унифицируем возвращаемые типы
            let s = unify(&subst.apply(r1), &subst.apply(r2))?;
            Ok(subst.compose(&s))
        }

        // Record типы
        (SynType::Record(fields1), SynType::Record(fields2)) => {
            if fields1.len() != fields2.len() {
                return Err(SynTypeError::Mismatch {
                    expected: t1.clone(),
                    found: t2.clone(),
                });
            }

            let mut subst = Substitution::new();
            for ((n1, t1), (n2, t2)) in fields1.iter().zip(fields2.iter()) {
                if n1 != n2 {
                    return Err(SynTypeError::Mismatch {
                        expected: SynType::Record(fields1.clone()),
                        found: SynType::Record(fields2.clone()),
                    });
                }
                let s = unify(&subst.apply(t1), &subst.apply(t2))?;
                subst = subst.compose(&s);
            }
            Ok(subst)
        }

        // Linear, SharedRef, MutableRef
        (SynType::Linear(inner1), SynType::Linear(inner2))
        | (SynType::SharedRef(inner1), SynType::SharedRef(inner2))
        | (SynType::MutableRef(inner1), SynType::MutableRef(inner2)) => unify(inner1, inner2),

        // Result типы
        (SynType::Result { ok: ok1, err: err1 }, SynType::Result { ok: ok2, err: err2 }) => {
            let s1 = unify(ok1, ok2)?;
            let s2 = unify(&s1.apply(err1), &s1.apply(err2))?;
            Ok(s1.compose(&s2))
        }

        // Foreign типы
        (SynType::Foreign(name1), SynType::Foreign(name2)) if name1 == name2 => {
            Ok(Substitution::new())
        }

        // Несовпадение типов
        _ => Err(SynTypeError::Mismatch {
            expected: t1.clone(),
            found: t2.clone(),
        }),
    }
}

// === Контекст типизации ===

/// Контекст типизации: хранит типы переменных и функций.
#[derive(Debug, Clone, Default)]
pub struct TypeContext {
    /// Типы переменных
    variables: HashMap<String, SynType>,
    /// Типы функций
    functions: HashMap<String, SynType>,
    /// Типы узлов ASG (по NodeID)
    node_types: HashMap<NodeID, SynType>,
}

impl TypeContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_var(&mut self, name: String, ty: SynType) {
        self.variables.insert(name, ty);
    }

    pub fn get_var(&self, name: &str) -> Option<&SynType> {
        self.variables.get(name)
    }

    pub fn insert_function(&mut self, name: String, ty: SynType) {
        self.functions.insert(name, ty);
    }

    pub fn get_function(&self, name: &str) -> Option<&SynType> {
        self.functions.get(name)
    }

    pub fn insert_node_type(&mut self, id: NodeID, ty: SynType) {
        self.node_types.insert(id, ty);
    }

    pub fn get_node_type(&self, id: NodeID) -> Option<&SynType> {
        self.node_types.get(&id)
    }
}

// === Type Checker ===

/// Type Checker для ASG.
pub struct TypeChecker {
    context: TypeContext,
    substitution: Substitution,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            context: TypeContext::new(),
            substitution: Substitution::new(),
        }
    }

    /// Основная функция проверки типов.
    pub fn check(&mut self, asg: &ASG) -> ASGResult<()> {
        // Вывод типов для всех узлов
        for node in &asg.nodes {
            self.infer_node_type(asg, node)?;
        }
        Ok(())
    }

    /// Получить типы всех узлов после проверки.
    pub fn get_node_types(&self) -> HashMap<NodeID, SynType> {
        let mut result = HashMap::new();
        for (id, ty) in &self.context.node_types {
            result.insert(*id, self.substitution.apply(ty));
        }
        result
    }

    /// Вывод типа для одного узла.
    fn infer_node_type(&mut self, asg: &ASG, node: &Node) -> ASGResult<SynType> {
        // Если тип уже известен, возвращаем его
        if let Some(ty) = self.context.get_node_type(node.id) {
            return Ok(self.substitution.apply(ty));
        }

        let ty = match node.node_type {
            // === Литералы ===
            NodeType::LiteralInt => SynType::Int,
            NodeType::LiteralFloat => SynType::Float,
            NodeType::LiteralBool => SynType::Bool,
            NodeType::LiteralString => SynType::String,
            NodeType::LiteralUnit => SynType::Unit,

            // === Бинарные арифметические операции ===
            NodeType::BinaryOperation
            | NodeType::Sub
            | NodeType::Mul
            | NodeType::Div
            | NodeType::Mod => {
                let (t1, t2) = self.get_binary_operand_types(asg, node)?;

                // Создаём переменную типа для результата
                let result_type = fresh_type_var();

                // Унифицируем первый операнд с результатом
                let s1 =
                    unify(&t1, &result_type).map_err(|e| ASGError::TypeError(e.to_string()))?;
                self.substitution = self.substitution.compose(&s1);

                // Унифицируем второй операнд с результатом
                let s2 = unify(
                    &self.substitution.apply(&t2),
                    &self.substitution.apply(&result_type),
                )
                .map_err(|e| ASGError::TypeError(e.to_string()))?;
                self.substitution = self.substitution.compose(&s2);

                self.substitution.apply(&result_type)
            }

            // Унарный минус
            NodeType::Neg => {
                let operand_type = self.get_unary_operand_type(asg, node)?;
                // Результат того же типа
                operand_type
            }

            // === Операции сравнения ===
            NodeType::Eq
            | NodeType::Ne
            | NodeType::Lt
            | NodeType::Le
            | NodeType::Gt
            | NodeType::Ge => {
                let (t1, t2) = self.get_binary_operand_types(asg, node)?;

                // Операнды должны быть одного типа
                let s = unify(&t1, &t2).map_err(|e| ASGError::TypeError(e.to_string()))?;
                self.substitution = self.substitution.compose(&s);

                // Результат — Bool
                SynType::Bool
            }

            // === Логические операции ===
            NodeType::And | NodeType::Or => {
                let (t1, t2) = self.get_binary_operand_types(asg, node)?;

                // Оба операнда должны быть Bool
                let s1 =
                    unify(&t1, &SynType::Bool).map_err(|e| ASGError::TypeError(e.to_string()))?;
                self.substitution = self.substitution.compose(&s1);

                let s2 = unify(&self.substitution.apply(&t2), &SynType::Bool)
                    .map_err(|e| ASGError::TypeError(e.to_string()))?;
                self.substitution = self.substitution.compose(&s2);

                SynType::Bool
            }

            NodeType::Not => {
                let operand_type = self.get_unary_operand_type(asg, node)?;

                // Операнд должен быть Bool
                let s = unify(&operand_type, &SynType::Bool)
                    .map_err(|e| ASGError::TypeError(e.to_string()))?;
                self.substitution = self.substitution.compose(&s);

                SynType::Bool
            }

            // === If выражение ===
            NodeType::If => {
                // Условие должно быть Bool
                let cond_type = self.get_edge_target_type(asg, node, EdgeType::Condition)?;
                let s = unify(&cond_type, &SynType::Bool)
                    .map_err(|e| ASGError::TypeError(e.to_string()))?;
                self.substitution = self.substitution.compose(&s);

                // Then и Else должны быть одного типа
                let then_type = self.get_edge_target_type(asg, node, EdgeType::ThenBranch)?;

                let else_type =
                    if let Ok(t) = self.get_edge_target_type(asg, node, EdgeType::ElseBranch) {
                        t
                    } else {
                        // Если нет else, результат — Unit
                        SynType::Unit
                    };

                let s = unify(&then_type, &else_type)
                    .map_err(|e| ASGError::TypeError(e.to_string()))?;
                self.substitution = self.substitution.compose(&s);

                self.substitution.apply(&then_type)
            }

            // === Функция ===
            NodeType::Function => {
                let func_name = node
                    .get_name()
                    .unwrap_or_else(|| format!("anon_{}", node.id));

                // Собираем типы параметров
                let param_types = self.get_function_parameters(asg, node)?;

                // Тип тела функции
                let body_type =
                    if let Ok(t) = self.get_edge_target_type(asg, node, EdgeType::FunctionBody) {
                        t
                    } else {
                        SynType::Unit
                    };

                let func_type = SynType::Function {
                    parameters: param_types,
                    return_type: Box::new(body_type),
                };

                // Сохраняем тип функции
                self.context.insert_function(func_name, func_type.clone());

                func_type
            }

            // === Вызов функции ===
            NodeType::Call => {
                let func_type = self.get_edge_target_type(asg, node, EdgeType::CallTarget)?;
                let arg_types = self.get_call_arguments(asg, node)?;

                match &func_type {
                    SynType::Function {
                        parameters,
                        return_type,
                    } => {
                        if parameters.len() != arg_types.len() {
                            return Err(ASGError::TypeError(format!(
                                "Expected {} arguments, got {}",
                                parameters.len(),
                                arg_types.len()
                            )));
                        }

                        // Унифицируем аргументы с параметрами
                        for (param, arg) in parameters.iter().zip(arg_types.iter()) {
                            let s = unify(
                                &self.substitution.apply(param),
                                &self.substitution.apply(arg),
                            )
                            .map_err(|e| ASGError::TypeError(e.to_string()))?;
                            self.substitution = self.substitution.compose(&s);
                        }

                        self.substitution.apply(return_type)
                    }
                    SynType::TypeVariable(_) => {
                        // Создаём тип функции и унифицируем
                        let result = fresh_type_var();
                        let inferred_func = SynType::Function {
                            parameters: arg_types,
                            return_type: Box::new(result.clone()),
                        };
                        let s = unify(&func_type, &inferred_func)
                            .map_err(|e| ASGError::TypeError(e.to_string()))?;
                        self.substitution = self.substitution.compose(&s);
                        self.substitution.apply(&result)
                    }
                    _ => {
                        return Err(ASGError::TypeError(format!(
                            "Expected function type, got {:?}",
                            func_type
                        )));
                    }
                }
            }

            // === Переменная ===
            NodeType::Variable => {
                let var_name = node.get_name().ok_or(ASGError::MissingPayload(node.id))?;

                let value_type =
                    if let Ok(t) = self.get_edge_target_type(asg, node, EdgeType::VarValue) {
                        t
                    } else {
                        fresh_type_var()
                    };

                self.context.insert_var(var_name, value_type.clone());
                value_type
            }

            NodeType::VarRef => {
                let var_name = node.get_name().ok_or(ASGError::MissingPayload(node.id))?;

                self.context
                    .get_var(&var_name)
                    .cloned()
                    .ok_or_else(|| ASGError::UnknownVariable(var_name))?
            }

            // === Присваивание ===
            NodeType::Assign => {
                let target_type = self.get_edge_target_type(asg, node, EdgeType::AssignTarget)?;
                let value_type = self.get_edge_target_type(asg, node, EdgeType::AssignValue)?;

                // Типы должны совпадать
                let s = unify(&target_type, &value_type)
                    .map_err(|e| ASGError::TypeError(e.to_string()))?;
                self.substitution = self.substitution.compose(&s);

                SynType::Unit
            }

            // === Тензоры ===
            NodeType::LiteralTensor
            | NodeType::TensorAdd
            | NodeType::TensorMul
            | NodeType::TensorMatMul
            | NodeType::TensorGrad => SynType::Foreign("Tensor".to_string()),

            // === Параметр функции ===
            NodeType::Parameter => {
                let param_name = node
                    .get_name()
                    .unwrap_or_else(|| format!("param_{}", node.id));

                // Создаём переменную типа для параметра
                let param_type = fresh_type_var();
                self.context.insert_var(param_name, param_type.clone());
                param_type
            }

            // === Return ===
            NodeType::Return => {
                if let Ok(t) = self.get_edge_target_type(asg, node, EdgeType::ReturnValue) {
                    t
                } else {
                    SynType::Unit
                }
            }

            // === Loop ===
            NodeType::Loop => {
                // Тело цикла
                if let Ok(_) = self.get_edge_target_type(asg, node, EdgeType::LoopBody) {
                    // Цикл не имеет значения
                }
                SynType::Unit
            }

            // === Массивы ===
            NodeType::Array => {
                let element_edges = node.find_edges(EdgeType::ArrayElement);
                if element_edges.is_empty() {
                    // Пустой массив — тип элемента неизвестен
                    SynType::Foreign("Array".to_string())
                } else {
                    // Все элементы должны быть одного типа
                    let first_type =
                        self.get_edge_target_type(asg, node, EdgeType::ArrayElement)?;
                    for edge in element_edges.iter().skip(1) {
                        let elem_node = asg
                            .find_node(edge.target_node_id)
                            .ok_or(ASGError::NodeNotFound(edge.target_node_id))?;
                        let elem_type = self.infer_node_type(asg, elem_node)?;
                        let s = unify(&first_type, &elem_type)
                            .map_err(|e| ASGError::TypeError(e.to_string()))?;
                        self.substitution = self.substitution.compose(&s);
                    }
                    SynType::Foreign("Array".to_string())
                }
            }

            // === Record ===
            NodeType::Record => {
                let field_edges = node.find_edges(EdgeType::RecordFieldDef);
                let mut fields = Vec::new();
                for edge in field_edges {
                    let field_node = asg
                        .find_node(edge.target_node_id)
                        .ok_or(ASGError::NodeNotFound(edge.target_node_id))?;
                    let field_name = field_node.get_name().unwrap_or_default();
                    let field_type = self.infer_node_type(asg, field_node)?;
                    fields.push((field_name, field_type));
                }
                SynType::Record(fields)
            }

            // По умолчанию — свежая переменная типа
            _ => fresh_type_var(),
        };

        self.context.insert_node_type(node.id, ty.clone());
        Ok(ty)
    }

    // === Вспомогательные методы ===

    /// Получить типы двух операндов для бинарной операции.
    fn get_binary_operand_types(
        &mut self,
        asg: &ASG,
        node: &Node,
    ) -> ASGResult<(SynType, SynType)> {
        let edges: Vec<_> = node
            .edges
            .iter()
            .filter(|e| {
                e.edge_type == EdgeType::ApplicationArgument
                    || e.edge_type == EdgeType::FirstOperand
                    || e.edge_type == EdgeType::SecondOperand
            })
            .collect();

        if edges.len() < 2 {
            return Err(ASGError::MissingEdge(
                node.id,
                EdgeType::ApplicationArgument,
            ));
        }

        let n1 = asg
            .find_node(edges[0].target_node_id)
            .ok_or(ASGError::NodeNotFound(edges[0].target_node_id))?;
        let n2 = asg
            .find_node(edges[1].target_node_id)
            .ok_or(ASGError::NodeNotFound(edges[1].target_node_id))?;

        let t1 = self.infer_node_type(asg, n1)?;
        let t2 = self.infer_node_type(asg, n2)?;

        Ok((t1, t2))
    }

    /// Получить тип единственного операнда.
    fn get_unary_operand_type(&mut self, asg: &ASG, node: &Node) -> ASGResult<SynType> {
        let edge = node.edges.first().ok_or(ASGError::MissingEdge(
            node.id,
            EdgeType::ApplicationArgument,
        ))?;
        let target = asg
            .find_node(edge.target_node_id)
            .ok_or(ASGError::NodeNotFound(edge.target_node_id))?;
        self.infer_node_type(asg, target)
    }

    /// Получить тип узла по ребру.
    fn get_edge_target_type(
        &mut self,
        asg: &ASG,
        node: &Node,
        edge_type: EdgeType,
    ) -> ASGResult<SynType> {
        let edge = node
            .find_edge(edge_type)
            .ok_or(ASGError::MissingEdge(node.id, edge_type))?;
        let target = asg
            .find_node(edge.target_node_id)
            .ok_or(ASGError::NodeNotFound(edge.target_node_id))?;
        self.infer_node_type(asg, target)
    }

    /// Получить типы параметров функции.
    fn get_function_parameters(&mut self, asg: &ASG, node: &Node) -> ASGResult<Vec<SynType>> {
        let param_edges = node.find_edges(EdgeType::FunctionParameter);
        let mut types = Vec::new();

        for edge in param_edges {
            let param_node = asg
                .find_node(edge.target_node_id)
                .ok_or(ASGError::NodeNotFound(edge.target_node_id))?;
            types.push(self.infer_node_type(asg, param_node)?);
        }

        Ok(types)
    }

    /// Получить типы аргументов вызова функции.
    fn get_call_arguments(&mut self, asg: &ASG, node: &Node) -> ASGResult<Vec<SynType>> {
        let arg_edges: Vec<_> = node
            .edges
            .iter()
            .filter(|e| {
                e.edge_type == EdgeType::CallArgument
                    || e.edge_type == EdgeType::ApplicationArgument
            })
            .collect();

        let mut types = Vec::new();
        for edge in arg_edges {
            let arg_node = asg
                .find_node(edge.target_node_id)
                .ok_or(ASGError::NodeNotFound(edge.target_node_id))?;
            types.push(self.infer_node_type(asg, arg_node)?);
        }

        Ok(types)
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

// === Публичный API ===

/// Проверка корректности типов в ASG.
pub fn check_types(asg: &ASG) -> ASGResult<()> {
    let mut checker = TypeChecker::new();
    checker.check(asg)
}

/// Вывод типов в ASG, возвращает типы всех узлов.
pub fn infer_types(asg: &ASG) -> ASGResult<HashMap<NodeID, SynType>> {
    let mut checker = TypeChecker::new();
    checker.check(asg)?;
    Ok(checker.get_node_types())
}

// === Тесты ===

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unify_same_types() {
        let s = unify(&SynType::Int, &SynType::Int).unwrap();
        assert!(s.mappings.is_empty());
    }

    #[test]
    fn test_unify_type_variable() {
        let t = SynType::TypeVariable("a".to_string());
        let s = unify(&t, &SynType::Int).unwrap();
        assert_eq!(s.apply(&t), SynType::Int);
    }

    #[test]
    fn test_unify_functions() {
        let f1 = SynType::Function {
            parameters: vec![SynType::Int],
            return_type: Box::new(SynType::Bool),
        };
        let f2 = SynType::Function {
            parameters: vec![SynType::Int],
            return_type: Box::new(SynType::Bool),
        };
        let s = unify(&f1, &f2).unwrap();
        assert!(s.mappings.is_empty());
    }

    #[test]
    fn test_unify_mismatch() {
        let result = unify(&SynType::Int, &SynType::Bool);
        assert!(result.is_err());
    }

    #[test]
    fn test_occurs_check() {
        assert!(occurs_check("a", &SynType::TypeVariable("a".to_string())));
        assert!(!occurs_check("a", &SynType::TypeVariable("b".to_string())));
        assert!(!occurs_check("a", &SynType::Int));
    }
}
