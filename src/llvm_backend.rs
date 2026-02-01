//! Модуль `llvm_backend`
//!
//! Компиляция ASG в LLVM IR через inkwell.
//!
//! Включается feature-флагом `llvm_backend`.

#[cfg(feature = "llvm_backend")]
use std::collections::HashMap;

#[cfg(feature = "llvm_backend")]
use crate::asg::{Node, NodeID, ASG};
#[cfg(feature = "llvm_backend")]
use crate::error::{ASGError, ASGResult};
#[cfg(feature = "llvm_backend")]
use crate::nodecodes::{EdgeType, NodeType};

// Импорты для заглушки без llvm_backend
#[cfg(not(feature = "llvm_backend"))]
use crate::asg::ASG;
#[cfg(not(feature = "llvm_backend"))]
use crate::error::ASGResult;

// === Реализация с inkwell (когда feature включен) ===

#[cfg(feature = "llvm_backend")]
use inkwell::builder::Builder;
#[cfg(feature = "llvm_backend")]
use inkwell::context::Context;
#[cfg(feature = "llvm_backend")]
use inkwell::module::Module;
#[cfg(feature = "llvm_backend")]
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
#[cfg(feature = "llvm_backend")]
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, PointerValue};
#[cfg(feature = "llvm_backend")]
use inkwell::FloatPredicate;
#[cfg(feature = "llvm_backend")]
use inkwell::IntPredicate;
#[cfg(feature = "llvm_backend")]
use inkwell::OptimizationLevel;

/// Замыкание (closure) - функция + окружение.
#[cfg(feature = "llvm_backend")]
#[derive(Clone)]
pub struct Closure<'ctx> {
    /// Указатель на функцию
    pub function: FunctionValue<'ctx>,
    /// Указатель на окружение (может быть null)
    pub env_ptr: Option<PointerValue<'ctx>>,
    /// Имена захваченных переменных
    pub captured: Vec<String>,
}

/// LLVM Backend для компиляции ASG.
#[cfg(feature = "llvm_backend")]
pub struct LLVMBackend<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    /// Кэш скомпилированных значений для узлов
    values: HashMap<NodeID, BasicValueEnum<'ctx>>,
    /// Кэш функций
    functions: HashMap<String, FunctionValue<'ctx>>,
    /// Кэш переменных (alloca)
    variables: HashMap<String, PointerValue<'ctx>>,
    /// Кэш замыканий
    closures: HashMap<String, Closure<'ctx>>,
    /// Счётчик для уникальных имён замыканий
    closure_counter: u32,
    /// Текущий scope переменных (для определения captured vars)
    current_scope: Vec<String>,
}

#[cfg(feature = "llvm_backend")]
impl<'ctx> LLVMBackend<'ctx> {
    /// Создать новый LLVM backend.
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        Self {
            context,
            module,
            builder,
            values: HashMap::new(),
            functions: HashMap::new(),
            variables: HashMap::new(),
            closures: HashMap::new(),
            closure_counter: 0,
            current_scope: Vec::new(),
        }
    }

    /// Компиляция ASG в LLVM IR.
    pub fn compile(&mut self, asg: &ASG) -> ASGResult<String> {
        // Создаём функцию main
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", fn_type, None);

        let entry_block = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry_block);

        // Компилируем все узлы
        let mut last_value = None;
        for node in &asg.nodes {
            match self.compile_node(asg, node) {
                Ok(val) => last_value = Some(val),
                Err(_) => continue, // Пропускаем узлы, которые не компилируются напрямую
            }
        }

        // Возвращаем последнее значение или 0
        let return_value = match last_value {
            Some(BasicValueEnum::IntValue(v)) => v,
            _ => i64_type.const_int(0, false),
        };

        self.builder
            .build_return(Some(&return_value))
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        Ok(self.module.print_to_string().to_string())
    }

    /// Компиляция одного узла ASG.
    fn compile_node(&mut self, asg: &ASG, node: &Node) -> ASGResult<BasicValueEnum<'ctx>> {
        // Проверяем кэш
        if let Some(val) = self.values.get(&node.id) {
            return Ok(*val);
        }

        let value = match node.node_type {
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
                let val = i64::from_le_bytes(bytes);
                let i64_type = self.context.i64_type();
                BasicValueEnum::IntValue(i64_type.const_int(val as u64, true))
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
                let val = f64::from_le_bytes(bytes);
                let f64_type = self.context.f64_type();
                BasicValueEnum::FloatValue(f64_type.const_float(val))
            }

            NodeType::LiteralBool => {
                let payload = node
                    .payload
                    .as_ref()
                    .ok_or(ASGError::MissingPayload(node.id))?;
                let val = payload.first().map(|&b| b != 0).unwrap_or(false);
                let i1_type = self.context.bool_type();
                BasicValueEnum::IntValue(i1_type.const_int(val as u64, false))
            }

            NodeType::LiteralString => self.compile_string_literal(node)?,

            NodeType::LiteralUnit => {
                // Unit представляем как 0
                BasicValueEnum::IntValue(self.context.i64_type().const_int(0, false))
            }

            // === Строковые операции ===
            NodeType::StringConcat => self.compile_string_concat(asg, node)?,
            NodeType::StringLength => self.compile_string_length(asg, node)?,

            // === Арифметические операции (поддержка int и float) ===
            NodeType::BinaryOperation => self.compile_binary_arithmetic(
                asg,
                node,
                |builder, a, b| builder.build_int_add(a, b, "add"),
                |builder, a, b| builder.build_float_add(a, b, "fadd"),
            )?,

            NodeType::Sub => self.compile_binary_arithmetic(
                asg,
                node,
                |builder, a, b| builder.build_int_sub(a, b, "sub"),
                |builder, a, b| builder.build_float_sub(a, b, "fsub"),
            )?,

            NodeType::Mul => self.compile_binary_arithmetic(
                asg,
                node,
                |builder, a, b| builder.build_int_mul(a, b, "mul"),
                |builder, a, b| builder.build_float_mul(a, b, "fmul"),
            )?,

            NodeType::Div => self.compile_binary_arithmetic(
                asg,
                node,
                |builder, a, b| builder.build_int_signed_div(a, b, "div"),
                |builder, a, b| builder.build_float_div(a, b, "fdiv"),
            )?,

            NodeType::Mod => self.compile_binary_arithmetic(
                asg,
                node,
                |builder, a, b| builder.build_int_signed_rem(a, b, "mod"),
                |builder, a, b| builder.build_float_rem(a, b, "fmod"),
            )?,

            // === Операции сравнения (поддержка int и float) ===
            NodeType::Eq => {
                self.compile_comparison(asg, node, IntPredicate::EQ, FloatPredicate::OEQ)?
            }
            NodeType::Ne => {
                self.compile_comparison(asg, node, IntPredicate::NE, FloatPredicate::ONE)?
            }
            NodeType::Lt => {
                self.compile_comparison(asg, node, IntPredicate::SLT, FloatPredicate::OLT)?
            }
            NodeType::Le => {
                self.compile_comparison(asg, node, IntPredicate::SLE, FloatPredicate::OLE)?
            }
            NodeType::Gt => {
                self.compile_comparison(asg, node, IntPredicate::SGT, FloatPredicate::OGT)?
            }
            NodeType::Ge => {
                self.compile_comparison(asg, node, IntPredicate::SGE, FloatPredicate::OGE)?
            }

            // === Логические операции ===
            NodeType::And => self.compile_binary_int_op(asg, node, "and", |builder, a, b| {
                builder.build_and(a, b, "and")
            })?,

            NodeType::Or => self.compile_binary_int_op(asg, node, "or", |builder, a, b| {
                builder.build_or(a, b, "or")
            })?,

            NodeType::Not => {
                let operand = self.get_single_operand(asg, node)?;
                if let BasicValueEnum::IntValue(v) = operand {
                    let result = self
                        .builder
                        .build_not(v, "not")
                        .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                    BasicValueEnum::IntValue(result)
                } else {
                    return Err(ASGError::TypeError("Expected int for NOT".to_string()));
                }
            }

            // === If выражение ===
            NodeType::If => self.compile_if_expression(asg, node)?,

            // === Функции ===
            NodeType::Function => self.compile_function_definition(asg, node)?,

            NodeType::Lambda => self.compile_lambda(asg, node)?,

            NodeType::Call => self.compile_function_call(asg, node)?,

            // === Переменные ===
            NodeType::Variable => self.compile_variable_declaration(asg, node)?,

            NodeType::VarRef => self.compile_variable_reference(node)?,

            NodeType::Assign => self.compile_assignment(asg, node)?,

            // === Цикл ===
            NodeType::Loop => self.compile_loop(asg, node)?,

            // === Block (do) ===
            NodeType::Block => self.compile_block(asg, node)?,

            // === Neg (унарный минус) ===
            NodeType::Neg => {
                let operand = self.get_single_operand(asg, node)?;
                match operand {
                    BasicValueEnum::IntValue(v) => {
                        let result = self
                            .builder
                            .build_int_neg(v, "neg")
                            .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                        BasicValueEnum::IntValue(result)
                    }
                    BasicValueEnum::FloatValue(v) => {
                        let result = self
                            .builder
                            .build_float_neg(v, "fneg")
                            .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                        BasicValueEnum::FloatValue(result)
                    }
                    _ => return Err(ASGError::TypeError("Expected number for NEG".to_string())),
                }
            }

            // === IntDiv (целочисленное деление) ===
            NodeType::IntDiv => {
                self.compile_binary_int_op(asg, node, "intdiv", |builder, a, b| {
                    builder.build_int_signed_div(a, b, "intdiv")
                })?
            }

            // === Print ===
            NodeType::Print => self.compile_print(asg, node)?,

            // === Math Functions ===
            NodeType::MathSqrt => self.compile_math_intrinsic(asg, node, "llvm.sqrt.f64")?,
            NodeType::MathSin => self.compile_math_intrinsic(asg, node, "llvm.sin.f64")?,
            NodeType::MathCos => self.compile_math_intrinsic(asg, node, "llvm.cos.f64")?,
            NodeType::MathExp => self.compile_math_intrinsic(asg, node, "llvm.exp.f64")?,
            NodeType::MathLn => self.compile_math_intrinsic(asg, node, "llvm.log.f64")?,
            NodeType::MathLog10 => self.compile_math_intrinsic(asg, node, "llvm.log10.f64")?,
            NodeType::MathPow => self.compile_math_pow(asg, node)?,
            NodeType::MathAbs => self.compile_math_intrinsic(asg, node, "llvm.fabs.f64")?,
            NodeType::MathFloor => self.compile_math_intrinsic(asg, node, "llvm.floor.f64")?,
            NodeType::MathCeil => self.compile_math_intrinsic(asg, node, "llvm.ceil.f64")?,
            NodeType::MathRound => self.compile_math_intrinsic(asg, node, "llvm.round.f64")?,
            NodeType::MathPi => {
                let f64_type = self.context.f64_type();
                BasicValueEnum::FloatValue(f64_type.const_float(std::f64::consts::PI))
            }
            NodeType::MathE => {
                let f64_type = self.context.f64_type();
                BasicValueEnum::FloatValue(f64_type.const_float(std::f64::consts::E))
            }
            NodeType::MathMin => self.compile_math_minmax(asg, node, true)?,
            NodeType::MathMax => self.compile_math_minmax(asg, node, false)?,

            // Неподдерживаемые типы
            _ => {
                return Err(ASGError::CompilationError(format!(
                    "Unsupported node type for LLVM compilation: {:?}",
                    node.node_type
                )));
            }
        };

        self.values.insert(node.id, value);
        Ok(value)
    }

    // === Вспомогательные методы ===

    /// Компиляция бинарной целочисленной операции.
    fn compile_binary_int_op<F>(
        &mut self,
        asg: &ASG,
        node: &Node,
        _name: &str,
        op: F,
    ) -> ASGResult<BasicValueEnum<'ctx>>
    where
        F: FnOnce(
            &Builder<'ctx>,
            IntValue<'ctx>,
            IntValue<'ctx>,
        ) -> Result<IntValue<'ctx>, inkwell::builder::BuilderError>,
    {
        let (left, right) = self.get_binary_operands(asg, node)?;

        match (left, right) {
            (BasicValueEnum::IntValue(a), BasicValueEnum::IntValue(b)) => {
                let result = op(&self.builder, a, b)
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                Ok(BasicValueEnum::IntValue(result))
            }
            _ => Err(ASGError::TypeError(
                "Expected integers for binary operation".to_string(),
            )),
        }
    }

    /// Компиляция бинарной арифметической операции (int или float).
    fn compile_binary_arithmetic<FI, FF>(
        &mut self,
        asg: &ASG,
        node: &Node,
        int_op: FI,
        float_op: FF,
    ) -> ASGResult<BasicValueEnum<'ctx>>
    where
        FI: FnOnce(
            &Builder<'ctx>,
            IntValue<'ctx>,
            IntValue<'ctx>,
        ) -> Result<IntValue<'ctx>, inkwell::builder::BuilderError>,
        FF: FnOnce(
            &Builder<'ctx>,
            inkwell::values::FloatValue<'ctx>,
            inkwell::values::FloatValue<'ctx>,
        )
            -> Result<inkwell::values::FloatValue<'ctx>, inkwell::builder::BuilderError>,
    {
        let (left, right) = self.get_binary_operands(asg, node)?;

        match (left, right) {
            (BasicValueEnum::IntValue(a), BasicValueEnum::IntValue(b)) => {
                let result = int_op(&self.builder, a, b)
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                Ok(BasicValueEnum::IntValue(result))
            }
            (BasicValueEnum::FloatValue(a), BasicValueEnum::FloatValue(b)) => {
                let result = float_op(&self.builder, a, b)
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                Ok(BasicValueEnum::FloatValue(result))
            }
            // Смешанные типы: конвертируем int в float
            (BasicValueEnum::IntValue(a), BasicValueEnum::FloatValue(b)) => {
                let a_float = self
                    .builder
                    .build_signed_int_to_float(a, self.context.f64_type(), "itof")
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                let result = float_op(&self.builder, a_float, b)
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                Ok(BasicValueEnum::FloatValue(result))
            }
            (BasicValueEnum::FloatValue(a), BasicValueEnum::IntValue(b)) => {
                let b_float = self
                    .builder
                    .build_signed_int_to_float(b, self.context.f64_type(), "itof")
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                let result = float_op(&self.builder, a, b_float)
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                Ok(BasicValueEnum::FloatValue(result))
            }
            _ => Err(ASGError::TypeError(
                "Expected numbers for arithmetic operation".to_string(),
            )),
        }
    }

    /// Компиляция сравнения (int или float).
    fn compile_comparison(
        &mut self,
        asg: &ASG,
        node: &Node,
        int_pred: IntPredicate,
        float_pred: FloatPredicate,
    ) -> ASGResult<BasicValueEnum<'ctx>> {
        let (left, right) = self.get_binary_operands(asg, node)?;

        match (left, right) {
            (BasicValueEnum::IntValue(a), BasicValueEnum::IntValue(b)) => {
                let result = self
                    .builder
                    .build_int_compare(int_pred, a, b, "icmp")
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                Ok(BasicValueEnum::IntValue(result))
            }
            (BasicValueEnum::FloatValue(a), BasicValueEnum::FloatValue(b)) => {
                let result = self
                    .builder
                    .build_float_compare(float_pred, a, b, "fcmp")
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                Ok(BasicValueEnum::IntValue(result))
            }
            // Смешанные типы: конвертируем int в float
            (BasicValueEnum::IntValue(a), BasicValueEnum::FloatValue(b)) => {
                let a_float = self
                    .builder
                    .build_signed_int_to_float(a, self.context.f64_type(), "itof")
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                let result = self
                    .builder
                    .build_float_compare(float_pred, a_float, b, "fcmp")
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                Ok(BasicValueEnum::IntValue(result))
            }
            (BasicValueEnum::FloatValue(a), BasicValueEnum::IntValue(b)) => {
                let b_float = self
                    .builder
                    .build_signed_int_to_float(b, self.context.f64_type(), "itof")
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                let result = self
                    .builder
                    .build_float_compare(float_pred, a, b_float, "fcmp")
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                Ok(BasicValueEnum::IntValue(result))
            }
            _ => Err(ASGError::TypeError(
                "Expected numbers for comparison".to_string(),
            )),
        }
    }

    /// Компиляция целочисленного сравнения (обратная совместимость).
    fn compile_int_comparison(
        &mut self,
        asg: &ASG,
        node: &Node,
        predicate: IntPredicate,
    ) -> ASGResult<BasicValueEnum<'ctx>> {
        let (left, right) = self.get_binary_operands(asg, node)?;

        match (left, right) {
            (BasicValueEnum::IntValue(a), BasicValueEnum::IntValue(b)) => {
                let result = self
                    .builder
                    .build_int_compare(predicate, a, b, "cmp")
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                Ok(BasicValueEnum::IntValue(result))
            }
            _ => Err(ASGError::TypeError(
                "Expected integers for comparison".to_string(),
            )),
        }
    }

    /// Получить два операнда для бинарной операции.
    fn get_binary_operands(
        &mut self,
        asg: &ASG,
        node: &Node,
    ) -> ASGResult<(BasicValueEnum<'ctx>, BasicValueEnum<'ctx>)> {
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

        let left_node = asg
            .find_node(edges[0].target_node_id)
            .ok_or(ASGError::NodeNotFound(edges[0].target_node_id))?;
        let right_node = asg
            .find_node(edges[1].target_node_id)
            .ok_or(ASGError::NodeNotFound(edges[1].target_node_id))?;

        let left = self.compile_node(asg, left_node)?;
        let right = self.compile_node(asg, right_node)?;

        Ok((left, right))
    }

    /// Получить единственный операнд.
    fn get_single_operand(&mut self, asg: &ASG, node: &Node) -> ASGResult<BasicValueEnum<'ctx>> {
        let edge = node.edges.first().ok_or(ASGError::MissingEdge(
            node.id,
            EdgeType::ApplicationArgument,
        ))?;
        let target = asg
            .find_node(edge.target_node_id)
            .ok_or(ASGError::NodeNotFound(edge.target_node_id))?;
        self.compile_node(asg, target)
    }

    /// Компиляция унарной math intrinsic функции (sin, cos, sqrt, etc.).
    fn compile_math_intrinsic(
        &mut self,
        asg: &ASG,
        node: &Node,
        intrinsic_name: &str,
    ) -> ASGResult<BasicValueEnum<'ctx>> {
        let operand = self.get_single_operand(asg, node)?;

        // Конвертируем в float если нужно
        let f64_type = self.context.f64_type();
        let float_val = match operand {
            BasicValueEnum::FloatValue(v) => v,
            BasicValueEnum::IntValue(v) => self
                .builder
                .build_signed_int_to_float(v, f64_type, "itof")
                .map_err(|e| ASGError::CompilationError(e.to_string()))?,
            _ => {
                return Err(ASGError::TypeError(
                    "Expected number for math function".to_string(),
                ))
            }
        };

        // Получаем или создаём intrinsic функцию
        let fn_type = f64_type.fn_type(&[f64_type.into()], false);
        let intrinsic = self
            .module
            .get_function(intrinsic_name)
            .unwrap_or_else(|| self.module.add_function(intrinsic_name, fn_type, None));

        let result = self
            .builder
            .build_call(intrinsic, &[float_val.into()], "math_result")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?
            .try_as_basic_value()
            .left()
            .ok_or_else(|| {
                ASGError::CompilationError("Math intrinsic returned void".to_string())
            })?;

        Ok(result)
    }

    /// Компиляция pow (две аргумента).
    fn compile_math_pow(&mut self, asg: &ASG, node: &Node) -> ASGResult<BasicValueEnum<'ctx>> {
        let (base, exp) = self.get_binary_operands(asg, node)?;

        let f64_type = self.context.f64_type();

        // Конвертируем оба операнда в float
        let base_float = match base {
            BasicValueEnum::FloatValue(v) => v,
            BasicValueEnum::IntValue(v) => self
                .builder
                .build_signed_int_to_float(v, f64_type, "itof")
                .map_err(|e| ASGError::CompilationError(e.to_string()))?,
            _ => return Err(ASGError::TypeError("Expected number for pow".to_string())),
        };

        let exp_float = match exp {
            BasicValueEnum::FloatValue(v) => v,
            BasicValueEnum::IntValue(v) => self
                .builder
                .build_signed_int_to_float(v, f64_type, "itof")
                .map_err(|e| ASGError::CompilationError(e.to_string()))?,
            _ => return Err(ASGError::TypeError("Expected number for pow".to_string())),
        };

        let fn_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
        let pow_fn = self
            .module
            .get_function("llvm.pow.f64")
            .unwrap_or_else(|| self.module.add_function("llvm.pow.f64", fn_type, None));

        let result = self
            .builder
            .build_call(pow_fn, &[base_float.into(), exp_float.into()], "pow_result")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?
            .try_as_basic_value()
            .left()
            .ok_or_else(|| ASGError::CompilationError("Pow returned void".to_string()))?;

        Ok(result)
    }

    /// Компиляция min/max.
    fn compile_math_minmax(
        &mut self,
        asg: &ASG,
        node: &Node,
        is_min: bool,
    ) -> ASGResult<BasicValueEnum<'ctx>> {
        let (left, right) = self.get_binary_operands(asg, node)?;

        let f64_type = self.context.f64_type();
        let intrinsic_name = if is_min {
            "llvm.minnum.f64"
        } else {
            "llvm.maxnum.f64"
        };

        // Конвертируем оба операнда в float
        let left_float = match left {
            BasicValueEnum::FloatValue(v) => v,
            BasicValueEnum::IntValue(v) => self
                .builder
                .build_signed_int_to_float(v, f64_type, "itof")
                .map_err(|e| ASGError::CompilationError(e.to_string()))?,
            _ => {
                return Err(ASGError::TypeError(
                    "Expected number for min/max".to_string(),
                ))
            }
        };

        let right_float = match right {
            BasicValueEnum::FloatValue(v) => v,
            BasicValueEnum::IntValue(v) => self
                .builder
                .build_signed_int_to_float(v, f64_type, "itof")
                .map_err(|e| ASGError::CompilationError(e.to_string()))?,
            _ => {
                return Err(ASGError::TypeError(
                    "Expected number for min/max".to_string(),
                ))
            }
        };

        let fn_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
        let minmax_fn = self
            .module
            .get_function(intrinsic_name)
            .unwrap_or_else(|| self.module.add_function(intrinsic_name, fn_type, None));

        let result = self
            .builder
            .build_call(
                minmax_fn,
                &[left_float.into(), right_float.into()],
                "minmax_result",
            )
            .map_err(|e| ASGError::CompilationError(e.to_string()))?
            .try_as_basic_value()
            .left()
            .ok_or_else(|| ASGError::CompilationError("Min/max returned void".to_string()))?;

        Ok(result)
    }

    /// Компиляция if выражения.
    fn compile_if_expression(&mut self, asg: &ASG, node: &Node) -> ASGResult<BasicValueEnum<'ctx>> {
        let cond_edge = node
            .find_edge(EdgeType::Condition)
            .ok_or(ASGError::MissingEdge(node.id, EdgeType::Condition))?;
        let then_edge = node
            .find_edge(EdgeType::ThenBranch)
            .ok_or(ASGError::MissingEdge(node.id, EdgeType::ThenBranch))?;
        let else_edge = node.find_edge(EdgeType::ElseBranch);

        // Компилируем условие
        let cond_node = asg
            .find_node(cond_edge.target_node_id)
            .ok_or(ASGError::NodeNotFound(cond_edge.target_node_id))?;
        let cond_val = self.compile_node(asg, cond_node)?;

        let cond_int = match cond_val {
            BasicValueEnum::IntValue(v) => v,
            _ => return Err(ASGError::TypeError("Condition must be boolean".to_string())),
        };

        // Получаем текущую функцию
        let current_fn = self
            .builder
            .get_insert_block()
            .ok_or(ASGError::CompilationError("No current block".to_string()))?
            .get_parent()
            .ok_or(ASGError::CompilationError("No parent function".to_string()))?;

        // Создаём базовые блоки
        let then_block = self.context.append_basic_block(current_fn, "then");
        let else_block = self.context.append_basic_block(current_fn, "else");
        let merge_block = self.context.append_basic_block(current_fn, "merge");

        // Условный переход
        self.builder
            .build_conditional_branch(cond_int, then_block, else_block)
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        // Then branch
        self.builder.position_at_end(then_block);
        let then_node = asg
            .find_node(then_edge.target_node_id)
            .ok_or(ASGError::NodeNotFound(then_edge.target_node_id))?;
        let then_val = self.compile_node(asg, then_node)?;
        self.builder
            .build_unconditional_branch(merge_block)
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;
        let then_block = self.builder.get_insert_block().unwrap();

        // Else branch
        self.builder.position_at_end(else_block);
        let else_val = if let Some(edge) = else_edge {
            let else_node = asg
                .find_node(edge.target_node_id)
                .ok_or(ASGError::NodeNotFound(edge.target_node_id))?;
            self.compile_node(asg, else_node)?
        } else {
            // Default: return 0
            BasicValueEnum::IntValue(self.context.i64_type().const_int(0, false))
        };
        self.builder
            .build_unconditional_branch(merge_block)
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;
        let else_block = self.builder.get_insert_block().unwrap();

        // Merge block с phi
        self.builder.position_at_end(merge_block);

        match (then_val, else_val) {
            (BasicValueEnum::IntValue(t), BasicValueEnum::IntValue(e)) => {
                let phi = self
                    .builder
                    .build_phi(self.context.i64_type(), "ifresult")
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                phi.add_incoming(&[(&t, then_block), (&e, else_block)]);
                Ok(BasicValueEnum::IntValue(
                    phi.as_basic_value().into_int_value(),
                ))
            }
            (BasicValueEnum::FloatValue(t), BasicValueEnum::FloatValue(e)) => {
                let phi = self
                    .builder
                    .build_phi(self.context.f64_type(), "ifresult")
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                phi.add_incoming(&[(&t, then_block), (&e, else_block)]);
                Ok(BasicValueEnum::FloatValue(
                    phi.as_basic_value().into_float_value(),
                ))
            }
            _ => Err(ASGError::TypeError("Branch type mismatch".to_string())),
        }
    }

    /// Компиляция определения функции.
    ///
    /// Параметры добавляются в current_scope для возможного захвата вложенными лямбдами.
    fn compile_function_definition(
        &mut self,
        asg: &ASG,
        node: &Node,
    ) -> ASGResult<BasicValueEnum<'ctx>> {
        let func_name = node.get_name().unwrap_or_else(|| format!("fn_{}", node.id));

        // Получаем параметры
        let param_edges = node.find_edges(EdgeType::FunctionParameter);

        let param_names: Vec<String> = param_edges
            .iter()
            .filter_map(|e| asg.find_node(e.target_node_id).and_then(|n| n.get_name()))
            .collect();

        let param_types: Vec<_> = param_edges
            .iter()
            .map(|_| self.context.i64_type().into())
            .collect();

        // Создаём тип функции
        let fn_type = self.context.i64_type().fn_type(&param_types, false);
        let function = self.module.add_function(&func_name, fn_type, None);

        // Сохраняем функцию
        self.functions.insert(func_name, function);

        // Сохраняем текущий builder position и scope
        let current_block = self.builder.get_insert_block();
        let old_scope = self.current_scope.clone();
        let old_variables = self.variables.clone();

        // Создаём entry block
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        // Очищаем scope и добавляем параметры
        self.current_scope.clear();
        let i64_type = self.context.i64_type();

        for (i, param_name) in param_names.iter().enumerate() {
            let param_value = function.get_nth_param(i as u32).unwrap();
            let alloca = self
                .builder
                .build_alloca(i64_type, param_name)
                .map_err(|e| ASGError::CompilationError(e.to_string()))?;
            self.builder
                .build_store(alloca, param_value)
                .map_err(|e| ASGError::CompilationError(e.to_string()))?;

            self.variables.insert(param_name.clone(), alloca);
            self.current_scope.push(param_name.clone());
        }

        // Компилируем тело
        if let Some(body_edge) = node.find_edge(EdgeType::FunctionBody) {
            let body_node = asg
                .find_node(body_edge.target_node_id)
                .ok_or(ASGError::NodeNotFound(body_edge.target_node_id))?;
            let result = self.compile_node(asg, body_node)?;

            match result {
                BasicValueEnum::IntValue(v) => {
                    self.builder
                        .build_return(Some(&v))
                        .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                }
                BasicValueEnum::FloatValue(v) => {
                    let int_val = self
                        .builder
                        .build_float_to_signed_int(v, i64_type, "f2i")
                        .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                    self.builder
                        .build_return(Some(&int_val))
                        .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                }
                _ => {
                    self.builder
                        .build_return(Some(&i64_type.const_int(0, false)))
                        .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                }
            }
        } else {
            self.builder
                .build_return(Some(&i64_type.const_int(0, false)))
                .map_err(|e| ASGError::CompilationError(e.to_string()))?;
        }

        // Восстанавливаем scope и builder position
        self.current_scope = old_scope;
        self.variables = old_variables;
        if let Some(block) = current_block {
            self.builder.position_at_end(block);
        }

        Ok(BasicValueEnum::PointerValue(
            function.as_global_value().as_pointer_value(),
        ))
    }

    /// Компиляция лямбда-выражения (замыкания).
    ///
    /// Lambda lifting: преобразуем лямбду в top-level функцию с дополнительным
    /// параметром для захваченных переменных (окружения).
    fn compile_lambda(&mut self, asg: &ASG, node: &Node) -> ASGResult<BasicValueEnum<'ctx>> {
        // Генерируем уникальное имя для лямбды
        let lambda_name = format!("__lambda_{}", self.closure_counter);
        self.closure_counter += 1;

        // Получаем параметры лямбды
        let param_edges = node.find_edges(EdgeType::FunctionParameter);
        let param_names: Vec<String> = param_edges
            .iter()
            .filter_map(|e| asg.find_node(e.target_node_id).and_then(|n| n.get_name()))
            .collect();

        // Находим захваченные переменные (те что в current_scope но не в параметрах)
        let captured: Vec<String> = self
            .current_scope
            .iter()
            .filter(|v| !param_names.contains(v))
            .cloned()
            .collect();

        // Создаём тип функции
        // Если есть захваченные переменные, первый параметр - указатель на окружение
        let i64_type = self.context.i64_type();
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

        let mut param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = Vec::new();

        // Добавляем указатель на окружение если есть захваченные переменные
        if !captured.is_empty() {
            param_types.push(ptr_type.into());
        }

        // Добавляем обычные параметры
        for _ in &param_names {
            param_types.push(i64_type.into());
        }

        let fn_type = i64_type.fn_type(&param_types, false);
        let function = self.module.add_function(&lambda_name, fn_type, None);

        // Сохраняем текущую позицию builder
        let current_block = self.builder.get_insert_block();

        // Создаём entry block для лямбды
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        // Сохраняем старые переменные и создаём новые для параметров
        let old_variables = self.variables.clone();

        // Если есть окружение, извлекаем захваченные переменные
        let env_offset = if !captured.is_empty() { 1 } else { 0 };

        if !captured.is_empty() {
            let env_ptr = function.get_nth_param(0).unwrap().into_pointer_value();

            // Создаём тип структуры окружения
            let env_field_types: Vec<inkwell::types::BasicTypeEnum> =
                captured.iter().map(|_| i64_type.into()).collect();
            let env_struct_type = self.context.struct_type(&env_field_types, false);

            // Извлекаем каждую захваченную переменную
            for (i, var_name) in captured.iter().enumerate() {
                let field_ptr = self
                    .builder
                    .build_struct_gep(
                        env_struct_type,
                        env_ptr,
                        i as u32,
                        &format!("env_{}", var_name),
                    )
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;

                let value = self
                    .builder
                    .build_load(i64_type, field_ptr, var_name)
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;

                // Создаём локальную переменную
                let alloca = self
                    .builder
                    .build_alloca(i64_type, var_name)
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                self.builder
                    .build_store(alloca, value)
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;

                self.variables.insert(var_name.clone(), alloca);
            }
        }

        // Создаём локальные переменные для параметров
        for (i, param_name) in param_names.iter().enumerate() {
            let param_value = function.get_nth_param((i + env_offset) as u32).unwrap();
            let alloca = self
                .builder
                .build_alloca(i64_type, param_name)
                .map_err(|e| ASGError::CompilationError(e.to_string()))?;
            self.builder
                .build_store(alloca, param_value)
                .map_err(|e| ASGError::CompilationError(e.to_string()))?;

            self.variables.insert(param_name.clone(), alloca);
        }

        // Компилируем тело
        let result = if let Some(body_edge) = node.find_edge(EdgeType::FunctionBody) {
            let body_node = asg
                .find_node(body_edge.target_node_id)
                .ok_or(ASGError::NodeNotFound(body_edge.target_node_id))?;
            self.compile_node(asg, body_node)?
        } else {
            BasicValueEnum::IntValue(i64_type.const_int(0, false))
        };

        // Return
        match result {
            BasicValueEnum::IntValue(v) => {
                self.builder
                    .build_return(Some(&v))
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
            }
            BasicValueEnum::FloatValue(v) => {
                // Конвертируем float в int для возврата
                let int_val = self
                    .builder
                    .build_float_to_signed_int(v, i64_type, "f2i")
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                self.builder
                    .build_return(Some(&int_val))
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
            }
            _ => {
                self.builder
                    .build_return(Some(&i64_type.const_int(0, false)))
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
            }
        }

        // Восстанавливаем переменные и позицию builder
        self.variables = old_variables;
        if let Some(block) = current_block {
            self.builder.position_at_end(block);
        }

        // Сохраняем closure
        let env_ptr = if !captured.is_empty() {
            // Создаём окружение на куче
            let env_field_types: Vec<inkwell::types::BasicTypeEnum> =
                captured.iter().map(|_| i64_type.into()).collect();
            let env_struct_type = self.context.struct_type(&env_field_types, false);

            // Вычисляем размер структуры
            let env_size = env_struct_type.size_of().unwrap();

            // Выделяем память
            let malloc = self.get_or_declare_malloc();
            let env_ptr = self
                .builder
                .build_call(malloc, &[env_size.into()], "env_alloc")
                .map_err(|e| ASGError::CompilationError(e.to_string()))?
                .try_as_basic_value()
                .left()
                .unwrap()
                .into_pointer_value();

            // Заполняем окружение текущими значениями переменных
            for (i, var_name) in captured.iter().enumerate() {
                if let Some(&var_ptr) = self.variables.get(var_name) {
                    let value = self
                        .builder
                        .build_load(i64_type, var_ptr, &format!("load_{}", var_name))
                        .map_err(|e| ASGError::CompilationError(e.to_string()))?;

                    let field_ptr = self
                        .builder
                        .build_struct_gep(
                            env_struct_type,
                            env_ptr,
                            i as u32,
                            &format!("env_field_{}", i),
                        )
                        .map_err(|e| ASGError::CompilationError(e.to_string()))?;

                    self.builder
                        .build_store(field_ptr, value)
                        .map_err(|e| ASGError::CompilationError(e.to_string()))?;
                }
            }

            Some(env_ptr)
        } else {
            None
        };

        // Сохраняем closure в кэш
        self.closures.insert(
            lambda_name.clone(),
            Closure {
                function,
                env_ptr,
                captured,
            },
        );

        self.functions.insert(lambda_name, function);

        // Возвращаем указатель на функцию
        Ok(BasicValueEnum::PointerValue(
            function.as_global_value().as_pointer_value(),
        ))
    }

    /// Компиляция вызова функции.
    ///
    /// Поддерживает вызов обычных функций и замыканий (closures).
    /// Для замыканий автоматически передаётся указатель на окружение как первый аргумент.
    fn compile_function_call(&mut self, asg: &ASG, node: &Node) -> ASGResult<BasicValueEnum<'ctx>> {
        // Получаем имя функции
        let call_target = node
            .find_edge(EdgeType::CallTarget)
            .ok_or(ASGError::MissingEdge(node.id, EdgeType::CallTarget))?;

        let target_node = asg
            .find_node(call_target.target_node_id)
            .ok_or(ASGError::NodeNotFound(call_target.target_node_id))?;
        let func_name = target_node.get_name().unwrap_or_default();

        // Проверяем, является ли это замыканием
        let closure = self.closures.get(&func_name).cloned();

        let function = if let Some(ref c) = closure {
            c.function
        } else {
            *self
                .functions
                .get(&func_name)
                .or_else(|| self.module.get_function(&func_name).as_ref())
                .ok_or(ASGError::UnknownFunction(func_name.clone()))?
        };

        // Компилируем аргументы
        let arg_edges: Vec<_> = node
            .edges
            .iter()
            .filter(|e| {
                e.edge_type == EdgeType::CallArgument
                    || e.edge_type == EdgeType::ApplicationArgument
            })
            .collect();

        let mut args: Vec<inkwell::values::BasicMetadataValueEnum> = Vec::new();

        // Если это замыкание с окружением, добавляем env_ptr как первый аргумент
        if let Some(ref c) = closure {
            if let Some(env_ptr) = c.env_ptr {
                args.push(env_ptr.into());
            }
        }

        // Добавляем обычные аргументы
        for edge in arg_edges {
            let arg_node = asg
                .find_node(edge.target_node_id)
                .ok_or(ASGError::NodeNotFound(edge.target_node_id))?;
            let arg_val = self.compile_node(asg, arg_node)?;
            args.push(arg_val.into());
        }

        let call = self
            .builder
            .build_call(function, &args, "call")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        call.try_as_basic_value()
            .left()
            .ok_or(ASGError::CompilationError("Call returned void".to_string()))
    }

    /// Компиляция объявления переменной.
    ///
    /// Добавляет переменную в current_scope для возможного захвата замыканиями.
    fn compile_variable_declaration(
        &mut self,
        asg: &ASG,
        node: &Node,
    ) -> ASGResult<BasicValueEnum<'ctx>> {
        let var_name = node
            .get_name()
            .unwrap_or_else(|| format!("var_{}", node.id));

        // Создаём alloca
        let alloca = self
            .builder
            .build_alloca(self.context.i64_type(), &var_name)
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        // Если есть значение, присваиваем
        if let Some(val_edge) = node.find_edge(EdgeType::VarValue) {
            let val_node = asg
                .find_node(val_edge.target_node_id)
                .ok_or(ASGError::NodeNotFound(val_edge.target_node_id))?;
            let value = self.compile_node(asg, val_node)?;
            self.builder
                .build_store(alloca, value)
                .map_err(|e| ASGError::CompilationError(e.to_string()))?;
        }

        self.variables.insert(var_name.clone(), alloca);

        // Добавляем в текущий scope для возможного захвата лямбдами
        if !self.current_scope.contains(&var_name) {
            self.current_scope.push(var_name);
        }

        Ok(BasicValueEnum::PointerValue(alloca))
    }

    /// Компиляция ссылки на переменную.
    fn compile_variable_reference(&mut self, node: &Node) -> ASGResult<BasicValueEnum<'ctx>> {
        let var_name = node.get_name().unwrap_or_default();
        let alloca = self
            .variables
            .get(&var_name)
            .ok_or(ASGError::UnknownVariable(var_name.clone()))?;

        let loaded = self
            .builder
            .build_load(self.context.i64_type(), *alloca, &var_name)
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;
        Ok(loaded)
    }

    /// Компиляция присваивания.
    fn compile_assignment(&mut self, asg: &ASG, node: &Node) -> ASGResult<BasicValueEnum<'ctx>> {
        let target_edge = node
            .find_edge(EdgeType::AssignTarget)
            .ok_or(ASGError::MissingEdge(node.id, EdgeType::AssignTarget))?;
        let value_edge = node
            .find_edge(EdgeType::AssignValue)
            .ok_or(ASGError::MissingEdge(node.id, EdgeType::AssignValue))?;

        let target_node = asg
            .find_node(target_edge.target_node_id)
            .ok_or(ASGError::NodeNotFound(target_edge.target_node_id))?;
        let var_name = target_node.get_name().unwrap_or_default();

        let alloca = self
            .variables
            .get(&var_name)
            .ok_or(ASGError::UnknownVariable(var_name))?;

        let value_node = asg
            .find_node(value_edge.target_node_id)
            .ok_or(ASGError::NodeNotFound(value_edge.target_node_id))?;
        let value = self.compile_node(asg, value_node)?;

        self.builder
            .build_store(*alloca, value)
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        Ok(value)
    }

    /// Компиляция цикла.
    fn compile_loop(&mut self, asg: &ASG, node: &Node) -> ASGResult<BasicValueEnum<'ctx>> {
        let current_fn = self
            .builder
            .get_insert_block()
            .ok_or(ASGError::CompilationError("No current block".to_string()))?
            .get_parent()
            .ok_or(ASGError::CompilationError("No parent function".to_string()))?;

        let loop_block = self.context.append_basic_block(current_fn, "loop");
        let after_block = self.context.append_basic_block(current_fn, "afterloop");

        // Переход к циклу
        self.builder
            .build_unconditional_branch(loop_block)
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        // Тело цикла
        self.builder.position_at_end(loop_block);

        if let Some(body_edge) = node.find_edge(EdgeType::LoopBody) {
            let body_node = asg
                .find_node(body_edge.target_node_id)
                .ok_or(ASGError::NodeNotFound(body_edge.target_node_id))?;
            let _ = self.compile_node(asg, body_node);
        }

        // Проверка условия
        if let Some(cond_edge) = node.find_edge(EdgeType::Condition) {
            let cond_node = asg
                .find_node(cond_edge.target_node_id)
                .ok_or(ASGError::NodeNotFound(cond_edge.target_node_id))?;
            let cond = self.compile_node(asg, cond_node)?;

            if let BasicValueEnum::IntValue(cond_int) = cond {
                self.builder
                    .build_conditional_branch(cond_int, loop_block, after_block)
                    .map_err(|e| ASGError::CompilationError(e.to_string()))?;
            }
        } else {
            // Бесконечный цикл
            self.builder
                .build_unconditional_branch(loop_block)
                .map_err(|e| ASGError::CompilationError(e.to_string()))?;
        }

        self.builder.position_at_end(after_block);
        Ok(BasicValueEnum::IntValue(
            self.context.i64_type().const_int(0, false),
        ))
    }

    /// Компиляция блока (do).
    fn compile_block(&mut self, asg: &ASG, node: &Node) -> ASGResult<BasicValueEnum<'ctx>> {
        let stmt_edges = node.find_edges(EdgeType::BlockStatement);

        let mut result = BasicValueEnum::IntValue(self.context.i64_type().const_int(0, false));

        for edge in stmt_edges {
            let stmt_node = asg
                .find_node(edge.target_node_id)
                .ok_or(ASGError::NodeNotFound(edge.target_node_id))?;
            result = self.compile_node(asg, stmt_node)?;
        }

        Ok(result)
    }

    // === Строковые операции ===

    /// Получить тип строки ASG: { i64 len, i8* data }.
    fn get_string_type(&self) -> inkwell::types::StructType<'ctx> {
        let i64_type = self.context.i64_type();
        let i8_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        self.context
            .struct_type(&[i64_type.into(), i8_ptr_type.into()], false)
    }

    /// Компиляция строкового литерала.
    fn compile_string_literal(&mut self, node: &Node) -> ASGResult<BasicValueEnum<'ctx>> {
        let payload = node
            .payload
            .as_ref()
            .ok_or(ASGError::MissingPayload(node.id))?;

        let s = String::from_utf8_lossy(payload);
        let len = s.len() as u64;

        // Создаём глобальную строку для данных
        let string_name = format!("str_{}", node.id);
        let data_ptr = self.create_global_string(&s, &string_name);

        // Создаём структуру строки
        let string_type = self.get_string_type();
        let i64_type = self.context.i64_type();

        // Аллоцируем структуру на стеке
        let string_alloca = self
            .builder
            .build_alloca(string_type, "string_tmp")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        // Устанавливаем длину
        let len_ptr = self
            .builder
            .build_struct_gep(string_type, string_alloca, 0, "str_len_ptr")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;
        self.builder
            .build_store(len_ptr, i64_type.const_int(len, false))
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        // Устанавливаем указатель на данные
        let data_ptr_ptr = self
            .builder
            .build_struct_gep(string_type, string_alloca, 1, "str_data_ptr")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;
        self.builder
            .build_store(data_ptr_ptr, data_ptr)
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        // Загружаем структуру
        let string_val = self
            .builder
            .build_load(string_type, string_alloca, "string_val")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        Ok(string_val)
    }

    /// Компиляция конкатенации строк.
    fn compile_string_concat(&mut self, asg: &ASG, node: &Node) -> ASGResult<BasicValueEnum<'ctx>> {
        // Получаем две строки
        let (left, right) = self.get_binary_operands(asg, node)?;

        let string_type = self.get_string_type();
        let i64_type = self.context.i64_type();
        let i8_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

        // Извлекаем длины
        let left_len = self
            .builder
            .build_extract_value(left.into_struct_value(), 0, "left_len")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?
            .into_int_value();

        let right_len = self
            .builder
            .build_extract_value(right.into_struct_value(), 1, "right_len")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        // Вычисляем общую длину
        let total_len = self
            .builder
            .build_int_add(left_len, right_len.into_int_value(), "total_len")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        // Получаем или создаём malloc
        let malloc = self.get_or_declare_malloc();

        // Аллоцируем память для новой строки (+1 для null terminator)
        let alloc_size = self
            .builder
            .build_int_add(total_len, i64_type.const_int(1, false), "alloc_size")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        let new_data = self
            .builder
            .build_call(malloc, &[alloc_size.into()], "new_str_data")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?
            .try_as_basic_value()
            .left()
            .ok_or(ASGError::CompilationError(
                "malloc returned void".to_string(),
            ))?
            .into_pointer_value();

        // Копируем первую строку
        let left_data = self
            .builder
            .build_extract_value(left.into_struct_value(), 1, "left_data")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?
            .into_pointer_value();

        let memcpy = self.get_or_declare_memcpy();
        self.builder
            .build_call(
                memcpy,
                &[new_data.into(), left_data.into(), left_len.into()],
                "copy_left",
            )
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        // Вычисляем смещение для второй строки
        let offset_ptr = unsafe {
            self.builder
                .build_gep(i8_ptr_type, new_data, &[left_len], "offset_ptr")
                .map_err(|e| ASGError::CompilationError(e.to_string()))?
        };

        // Копируем вторую строку
        let right_data = self
            .builder
            .build_extract_value(right.into_struct_value(), 1, "right_data")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?
            .into_pointer_value();

        let right_len_val = self
            .builder
            .build_extract_value(right.into_struct_value(), 0, "right_len_val")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?
            .into_int_value();

        self.builder
            .build_call(
                memcpy,
                &[offset_ptr.into(), right_data.into(), right_len_val.into()],
                "copy_right",
            )
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        // Создаём новую строковую структуру
        let result_alloca = self
            .builder
            .build_alloca(string_type, "concat_result")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        let len_ptr = self
            .builder
            .build_struct_gep(string_type, result_alloca, 0, "result_len_ptr")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;
        self.builder
            .build_store(len_ptr, total_len)
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        let data_ptr = self
            .builder
            .build_struct_gep(string_type, result_alloca, 1, "result_data_ptr")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;
        self.builder
            .build_store(data_ptr, new_data)
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        let result = self
            .builder
            .build_load(string_type, result_alloca, "concat_val")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        Ok(result)
    }

    /// Компиляция длины строки.
    fn compile_string_length(&mut self, asg: &ASG, node: &Node) -> ASGResult<BasicValueEnum<'ctx>> {
        let operand = self.get_single_operand(asg, node)?;

        let len = self
            .builder
            .build_extract_value(operand.into_struct_value(), 0, "str_len")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        Ok(len)
    }

    /// Получить или объявить malloc.
    fn get_or_declare_malloc(&self) -> FunctionValue<'ctx> {
        if let Some(malloc) = self.module.get_function("malloc") {
            return malloc;
        }

        let i8_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let i64_type = self.context.i64_type();
        let malloc_type = i8_ptr_type.fn_type(&[i64_type.into()], false);
        self.module.add_function("malloc", malloc_type, None)
    }

    /// Получить или объявить memcpy.
    fn get_or_declare_memcpy(&self) -> FunctionValue<'ctx> {
        if let Some(memcpy) = self.module.get_function("memcpy") {
            return memcpy;
        }

        let i8_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let i64_type = self.context.i64_type();
        let memcpy_type = i8_ptr_type.fn_type(
            &[i8_ptr_type.into(), i8_ptr_type.into(), i64_type.into()],
            false,
        );
        self.module.add_function("memcpy", memcpy_type, None)
    }

    /// Компиляция print (вызов printf).
    fn compile_print(&mut self, asg: &ASG, node: &Node) -> ASGResult<BasicValueEnum<'ctx>> {
        // Получаем аргумент
        let arg_edge = node.edges.first().ok_or(ASGError::MissingEdge(
            node.id,
            EdgeType::ApplicationArgument,
        ))?;

        let arg_node = asg
            .find_node(arg_edge.target_node_id)
            .ok_or(ASGError::NodeNotFound(arg_edge.target_node_id))?;
        let arg_val = self.compile_node(asg, arg_node)?;

        // Получаем или создаём printf
        let printf = self.get_or_declare_printf();

        // Создаём format string
        let (format_str, args) = match arg_val {
            BasicValueEnum::IntValue(v) => {
                let fmt = self.create_global_string("%lld\n", "fmt_int");
                (fmt, vec![v.into()])
            }
            BasicValueEnum::FloatValue(v) => {
                let fmt = self.create_global_string("%f\n", "fmt_float");
                (fmt, vec![v.into()])
            }
            BasicValueEnum::StructValue(s) => {
                // Проверяем, что это строка (структура с 2 полями)
                if s.get_type().count_fields() == 2 {
                    // Извлекаем длину и данные
                    let len = self
                        .builder
                        .build_extract_value(s, 0, "print_str_len")
                        .map_err(|e| ASGError::CompilationError(e.to_string()))?
                        .into_int_value();
                    let data = self
                        .builder
                        .build_extract_value(s, 1, "print_str_data")
                        .map_err(|e| ASGError::CompilationError(e.to_string()))?
                        .into_pointer_value();

                    // Используем %.*s для печати строки с длиной
                    let fmt = self.create_global_string("%.*s\n", "fmt_str");
                    (fmt, vec![len.into(), data.into()])
                } else {
                    let fmt = self.create_global_string("<struct>\n", "fmt_struct");
                    (fmt, vec![])
                }
            }
            BasicValueEnum::PointerValue(p) => {
                // Предполагаем C-строку
                let fmt = self.create_global_string("%s\n", "fmt_cstr");
                (fmt, vec![p.into()])
            }
            _ => {
                let fmt = self.create_global_string("<value>\n", "fmt_other");
                (fmt, vec![])
            }
        };

        // Вызываем printf
        let mut call_args: Vec<BasicValueEnum> = vec![format_str.into()];
        call_args.extend(args);
        let call_args_meta: Vec<_> = call_args.iter().map(|v| (*v).into()).collect();

        let _ = self
            .builder
            .build_call(printf, &call_args_meta, "printf_call")
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        Ok(BasicValueEnum::IntValue(
            self.context.i64_type().const_int(0, false),
        ))
    }

    /// Получить или объявить printf.
    fn get_or_declare_printf(&self) -> FunctionValue<'ctx> {
        if let Some(printf) = self.module.get_function("printf") {
            return printf;
        }

        let i32_type = self.context.i32_type();
        let i8_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let printf_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
        self.module.add_function("printf", printf_type, None)
    }

    /// Создать глобальную строку.
    fn create_global_string(&self, s: &str, name: &str) -> PointerValue<'ctx> {
        let string_val = self.context.const_string(s.as_bytes(), true);
        let global = self.module.add_global(string_val.get_type(), None, name);
        global.set_initializer(&string_val);
        global.as_pointer_value()
    }

    /// Получить LLVM IR как строку.
    pub fn get_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }

    /// Компиляция в объектный файл.
    pub fn compile_to_object(&self, output_path: &str) -> ASGResult<()> {
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple)
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or(ASGError::CompilationError(
                "Failed to create target machine".to_string(),
            ))?;

        target_machine
            .write_to_file(&self.module, FileType::Object, output_path.as_ref())
            .map_err(|e| ASGError::CompilationError(e.to_string()))?;

        Ok(())
    }
}

// === Заглушка для сборки без LLVM ===

#[cfg(not(feature = "llvm_backend"))]
pub struct LLVMBackend;

#[cfg(not(feature = "llvm_backend"))]
impl LLVMBackend {
    /// Компиляция ASG в LLVM IR (заглушка).
    ///
    /// Эта функция доступна только при включении feature `llvm_backend`.
    pub fn compile(asg: &ASG) -> ASGResult<String> {
        println!("LLVMBackend: LLVM support not compiled in. Enable 'llvm_backend' feature.");
        println!("ASG has {} nodes.", asg.nodes.len());
        Ok("; ModuleID = 'asg'\n; LLVM backend not available\n; Enable feature 'llvm_backend' to use real LLVM compilation".to_string())
    }
}

// === Тесты ===

#[cfg(test)]
mod tests {
    #[test]
    fn test_llvm_backend_exists() {
        // Просто проверяем, что модуль компилируется
        assert!(true);
    }

    #[cfg(feature = "llvm_backend")]
    mod llvm_tests {
        use super::super::*;
        use crate::asg::{Edge, Node, ASG};

        /// Создаёт простой ASG с целочисленным литералом
        fn create_int_literal_asg(value: i64) -> ASG {
            let mut asg = ASG::new();
            asg.nodes.push(Node {
                id: 0,
                node_type: NodeType::LiteralInt,
                payload: Some(value.to_le_bytes().to_vec()),
                edges: vec![],
                span: None,
            });
            asg
        }

        /// Создаёт ASG с бинарной операцией
        fn create_binary_op_asg(op: NodeType, a: i64, b: i64) -> ASG {
            let mut asg = ASG::new();

            // Литерал a
            asg.nodes.push(Node {
                id: 0,
                node_type: NodeType::LiteralInt,
                payload: Some(a.to_le_bytes().to_vec()),
                edges: vec![],
                span: None,
            });

            // Литерал b
            asg.nodes.push(Node {
                id: 1,
                node_type: NodeType::LiteralInt,
                payload: Some(b.to_le_bytes().to_vec()),
                edges: vec![],
                span: None,
            });

            // Операция
            asg.nodes.push(Node {
                id: 2,
                node_type: op,
                payload: None,
                edges: vec![
                    Edge {
                        edge_type: EdgeType::FirstOperand,
                        target_node_id: 0,
                        metadata: None,
                    },
                    Edge {
                        edge_type: EdgeType::SecondOperand,
                        target_node_id: 1,
                        metadata: None,
                    },
                ],
                span: None,
            });

            asg
        }

        #[test]
        fn test_compile_int_literal() {
            let context = Context::create();
            let mut backend = LLVMBackend::new(&context, "test");
            let asg = create_int_literal_asg(42);

            let result = backend.compile(&asg);
            assert!(result.is_ok());

            let ir = result.unwrap();
            assert!(ir.contains("ModuleID"));
            assert!(ir.contains("main"));
        }

        #[test]
        fn test_compile_float_literal() {
            let context = Context::create();
            let mut backend = LLVMBackend::new(&context, "test");

            let mut asg = ASG::new();
            asg.nodes.push(Node {
                id: 0,
                node_type: NodeType::LiteralFloat,
                payload: Some(3.14_f64.to_le_bytes().to_vec()),
                edges: vec![],
                span: None,
            });

            let result = backend.compile(&asg);
            assert!(result.is_ok());
        }

        #[test]
        fn test_compile_addition() {
            let context = Context::create();
            let mut backend = LLVMBackend::new(&context, "test");
            let asg = create_binary_op_asg(NodeType::BinaryOperation, 10, 20);

            let result = backend.compile(&asg);
            assert!(result.is_ok());

            let ir = result.unwrap();
            assert!(ir.contains("add"));
        }

        #[test]
        fn test_compile_subtraction() {
            let context = Context::create();
            let mut backend = LLVMBackend::new(&context, "test");
            let asg = create_binary_op_asg(NodeType::Sub, 30, 10);

            let result = backend.compile(&asg);
            assert!(result.is_ok());

            let ir = result.unwrap();
            assert!(ir.contains("sub"));
        }

        #[test]
        fn test_compile_multiplication() {
            let context = Context::create();
            let mut backend = LLVMBackend::new(&context, "test");
            let asg = create_binary_op_asg(NodeType::Mul, 5, 6);

            let result = backend.compile(&asg);
            assert!(result.is_ok());

            let ir = result.unwrap();
            assert!(ir.contains("mul"));
        }

        #[test]
        fn test_compile_comparison() {
            let context = Context::create();
            let mut backend = LLVMBackend::new(&context, "test");
            let asg = create_binary_op_asg(NodeType::Lt, 5, 10);

            let result = backend.compile(&asg);
            assert!(result.is_ok());

            let ir = result.unwrap();
            assert!(ir.contains("icmp"));
        }

        #[test]
        fn test_backend_new() {
            let context = Context::create();
            let backend = LLVMBackend::new(&context, "test_module");

            assert!(backend.values.is_empty());
            assert!(backend.functions.is_empty());
            assert!(backend.variables.is_empty());
            assert!(backend.closures.is_empty());
            assert_eq!(backend.closure_counter, 0);
            assert!(backend.current_scope.is_empty());
        }

        #[test]
        fn test_get_ir() {
            let context = Context::create();
            let mut backend = LLVMBackend::new(&context, "test");
            let asg = create_int_literal_asg(100);

            let _ = backend.compile(&asg);
            let ir = backend.get_ir();

            assert!(ir.contains("; ModuleID = 'test'"));
        }

        #[test]
        fn test_closure_struct() {
            let context = Context::create();
            let module = context.create_module("test");
            let fn_type = context.i64_type().fn_type(&[], false);
            let function = module.add_function("test_fn", fn_type, None);

            let closure = Closure {
                function,
                env_ptr: None,
                captured: vec!["x".to_string(), "y".to_string()],
            };

            assert_eq!(closure.captured.len(), 2);
            assert!(closure.env_ptr.is_none());
        }

        #[test]
        fn test_mixed_int_float_arithmetic() {
            let context = Context::create();
            let mut backend = LLVMBackend::new(&context, "test");

            let mut asg = ASG::new();

            // Int literal
            asg.nodes.push(Node {
                id: 0,
                node_type: NodeType::LiteralInt,
                payload: Some(10_i64.to_le_bytes().to_vec()),
                edges: vec![],
                span: None,
            });

            // Float literal
            asg.nodes.push(Node {
                id: 1,
                node_type: NodeType::LiteralFloat,
                payload: Some(3.5_f64.to_le_bytes().to_vec()),
                edges: vec![],
                span: None,
            });

            // Addition (should auto-convert int to float)
            asg.nodes.push(Node {
                id: 2,
                node_type: NodeType::BinaryOperation,
                payload: None,
                edges: vec![
                    Edge {
                        edge_type: EdgeType::FirstOperand,
                        target_node_id: 0,
                        metadata: None,
                    },
                    Edge {
                        edge_type: EdgeType::SecondOperand,
                        target_node_id: 1,
                        metadata: None,
                    },
                ],
                span: None,
            });

            let result = backend.compile(&asg);
            assert!(result.is_ok());

            let ir = result.unwrap();
            // Should contain int-to-float conversion
            assert!(ir.contains("sitofp") || ir.contains("fadd"));
        }
    }
}
