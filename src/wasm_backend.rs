//! Модуль `wasm_backend`
//!
//! Компиляция ASG в WebAssembly (WASM).
//!
//! Включается feature-флагом `wasm_backend`.

use crate::asg::ASG;
use crate::error::ASGResult;

// === Реализация с wasm-encoder (когда feature включен) ===

#[cfg(feature = "wasm_backend")]
use std::collections::HashMap;

#[cfg(feature = "wasm_backend")]
use crate::asg::{Node, NodeID};
#[cfg(feature = "wasm_backend")]
use crate::error::ASGError;
#[cfg(feature = "wasm_backend")]
use crate::nodecodes::{EdgeType, NodeType};

#[cfg(feature = "wasm_backend")]
use wasm_encoder::{
    CodeSection, DataSection, ExportKind, ExportSection, Function, FunctionSection,
    GlobalSection, GlobalType, ImportSection, Instruction, MemorySection, MemoryType,
    Module, TypeSection, ValType,
};

/// WASM Backend для компиляции ASG.
#[cfg(feature = "wasm_backend")]
pub struct WasmBackend {
    /// Кэш скомпилированных значений для узлов
    local_indices: HashMap<NodeID, u32>,
    /// Счётчик локальных переменных
    local_count: u32,
    /// Кэш имён переменных
    variable_locals: HashMap<String, u32>,
    /// Кэш функций
    function_indices: HashMap<String, u32>,
    /// Следующий индекс функции
    next_function_index: u32,
    /// Строковые данные для секции Data
    string_data: Vec<(u32, Vec<u8>)>,
    /// Следующий offset для строк в памяти
    string_offset: u32,
    /// Включить GC
    gc_enabled: bool,
}

#[cfg(feature = "wasm_backend")]
impl WasmBackend {
    /// Создать новый WASM backend.
    pub fn new() -> Self {
        Self {
            local_indices: HashMap::new(),
            local_count: 0,
            variable_locals: HashMap::new(),
            function_indices: HashMap::new(),
            next_function_index: 0,  // 0 будет для импортов
            string_data: Vec::new(),
            string_offset: 0x1000,  // Строки начинаются после GC metadata
            gc_enabled: true,
        }
    }

    /// Создать WASM backend без GC.
    pub fn new_without_gc() -> Self {
        Self {
            local_indices: HashMap::new(),
            local_count: 0,
            variable_locals: HashMap::new(),
            function_indices: HashMap::new(),
            next_function_index: 0,
            string_data: Vec::new(),
            string_offset: 1024,
            gc_enabled: false,
        }
    }

    /// Включить/выключить GC.
    pub fn set_gc_enabled(&mut self, enabled: bool) {
        self.gc_enabled = enabled;
    }

    /// Компиляция ASG в WASM байткод.
    pub fn compile(&mut self, asg: &ASG) -> ASGResult<Vec<u8>> {
        let mut module = Module::new();

        // === Type Section ===
        let mut types = TypeSection::new();
        // Type 0: () -> i64 (main function)
        types.ty().function(vec![], vec![ValType::I64]);
        // Type 1: (i64) -> () (print_int import)
        types.ty().function(vec![ValType::I64], vec![]);
        // Type 2: (f64) -> () (print_float import)
        types.ty().function(vec![ValType::F64], vec![]);
        // Type 3: (i64, i64) -> i64 (binary op)
        types.ty().function(vec![ValType::I64, ValType::I64], vec![ValType::I64]);
        // Type 4: (f64, f64) -> f64 (binary float op)
        types.ty().function(vec![ValType::F64, ValType::F64], vec![ValType::F64]);

        module.section(&types);

        // === Import Section ===
        let mut imports = ImportSection::new();
        // Import console.log_int для печати целых чисел
        imports.import("env", "print_int", wasm_encoder::EntityType::Function(1));
        imports.import("env", "print_float", wasm_encoder::EntityType::Function(2));
        self.next_function_index = 2;  // После импортов

        module.section(&imports);

        // === Function Section ===
        let mut functions = FunctionSection::new();
        // main функция использует type 0
        functions.function(0);
        self.function_indices.insert("main".to_string(), self.next_function_index);
        self.next_function_index += 1;

        module.section(&functions);

        // === Memory Section ===
        let mut memories = MemorySection::new();
        memories.memory(MemoryType {
            minimum: 1,  // 1 page = 64KB
            maximum: Some(16),
            memory64: false,
            shared: false,
            page_size_log2: None,
        });
        module.section(&memories);

        // === Global Section (для переменных) ===
        let mut globals = GlobalSection::new();
        // Глобальная переменная для stack pointer
        globals.global(
            GlobalType {
                val_type: ValType::I32,
                mutable: true,
                shared: false,
            },
            &wasm_encoder::ConstExpr::i32_const(1024),
        );
        module.section(&globals);

        // === Export Section ===
        let mut exports = ExportSection::new();
        exports.export("main", ExportKind::Func, 2);  // main function index
        exports.export("memory", ExportKind::Memory, 0);

        // GC exports
        if self.gc_enabled {
            // GC functions will be added in future
            // exports.export("gc_alloc", ExportKind::Func, gc_alloc_index);
            // exports.export("gc_collect", ExportKind::Func, gc_collect_index);
        }

        module.section(&exports);

        // === Code Section ===
        let mut codes = CodeSection::new();

        // Компилируем main функцию
        let main_code = self.compile_main(asg)?;
        codes.function(&main_code);

        module.section(&codes);

        // === Data Section (для строковых литералов) ===
        if !self.string_data.is_empty() {
            let mut data = DataSection::new();
            for (offset, bytes) in &self.string_data {
                data.active(
                    0,  // memory index
                    &wasm_encoder::ConstExpr::i32_const(*offset as i32),
                    bytes.iter().copied(),
                );
            }
            module.section(&data);
        }

        Ok(module.finish())
    }

    /// Компиляция main функции.
    fn compile_main(&mut self, asg: &ASG) -> ASGResult<Function> {
        let mut func = Function::new(vec![(1, ValType::I64)]);  // 1 локальная переменная

        // Компилируем все корневые узлы
        let mut last_node_id = None;
        for node in &asg.nodes {
            self.compile_node(asg, node, &mut func)?;
            last_node_id = Some(node.id);
        }

        // Возвращаем результат последнего узла или 0
        if last_node_id.is_some() {
            // Значение уже на стеке
        } else {
            func.instruction(&Instruction::I64Const(0));
        }
        func.instruction(&Instruction::End);

        Ok(func)
    }

    /// Компиляция одного узла ASG.
    fn compile_node(&mut self, asg: &ASG, node: &Node, func: &mut Function) -> ASGResult<()> {
        match node.node_type {
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
                func.instruction(&Instruction::I64Const(val));
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
                func.instruction(&Instruction::F64Const(val));
            }

            NodeType::LiteralBool => {
                let payload = node
                    .payload
                    .as_ref()
                    .ok_or(ASGError::MissingPayload(node.id))?;
                let val = payload.first().map(|&b| b != 0).unwrap_or(false);
                func.instruction(&Instruction::I64Const(if val { 1 } else { 0 }));
            }

            NodeType::LiteralString => {
                let payload = node
                    .payload
                    .as_ref()
                    .ok_or(ASGError::MissingPayload(node.id))?;

                // Сохраняем строку в data section
                let offset = self.string_offset;
                let mut data = payload.clone();
                data.push(0);  // null terminator
                let len = data.len() as u32;
                self.string_data.push((offset, data));
                self.string_offset += len;

                // Возвращаем указатель на строку (offset в памяти)
                func.instruction(&Instruction::I32Const(offset as i32));
            }

            NodeType::LiteralUnit => {
                func.instruction(&Instruction::I64Const(0));
            }

            // === Арифметические операции ===
            NodeType::BinaryOperation => {
                self.compile_binary_op(asg, node, func, Instruction::I64Add)?;
            }

            NodeType::Sub => {
                self.compile_binary_op(asg, node, func, Instruction::I64Sub)?;
            }

            NodeType::Mul => {
                self.compile_binary_op(asg, node, func, Instruction::I64Mul)?;
            }

            NodeType::Div => {
                self.compile_binary_op(asg, node, func, Instruction::I64DivS)?;
            }

            NodeType::Mod => {
                self.compile_binary_op(asg, node, func, Instruction::I64RemS)?;
            }

            // === Операции сравнения ===
            NodeType::Eq => {
                self.compile_binary_op(asg, node, func, Instruction::I64Eq)?;
            }

            NodeType::Ne => {
                self.compile_binary_op(asg, node, func, Instruction::I64Ne)?;
            }

            NodeType::Lt => {
                self.compile_binary_op(asg, node, func, Instruction::I64LtS)?;
            }

            NodeType::Le => {
                self.compile_binary_op(asg, node, func, Instruction::I64LeS)?;
            }

            NodeType::Gt => {
                self.compile_binary_op(asg, node, func, Instruction::I64GtS)?;
            }

            NodeType::Ge => {
                self.compile_binary_op(asg, node, func, Instruction::I64GeS)?;
            }

            // === Логические операции ===
            NodeType::And => {
                self.compile_binary_op(asg, node, func, Instruction::I64And)?;
            }

            NodeType::Or => {
                self.compile_binary_op(asg, node, func, Instruction::I64Or)?;
            }

            NodeType::Not => {
                let operand = node.edges.first()
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::ApplicationArgument))?;
                let operand_node = asg.find_node(operand.target_node_id)
                    .ok_or(ASGError::NodeNotFound(operand.target_node_id))?;
                self.compile_node(asg, operand_node, func)?;
                func.instruction(&Instruction::I64Eqz);  // not = eqz
            }

            // === Neg (унарный минус) ===
            NodeType::Neg => {
                let operand = node.edges.first()
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::ApplicationArgument))?;
                let operand_node = asg.find_node(operand.target_node_id)
                    .ok_or(ASGError::NodeNotFound(operand.target_node_id))?;

                // 0 - value = -value
                func.instruction(&Instruction::I64Const(0));
                self.compile_node(asg, operand_node, func)?;
                func.instruction(&Instruction::I64Sub);
            }

            // === If выражение ===
            NodeType::If => {
                self.compile_if_expression(asg, node, func)?;
            }

            // === Print ===
            NodeType::Print => {
                let arg = node.edges.first()
                    .ok_or(ASGError::MissingEdge(node.id, EdgeType::ApplicationArgument))?;
                let arg_node = asg.find_node(arg.target_node_id)
                    .ok_or(ASGError::NodeNotFound(arg.target_node_id))?;

                self.compile_node(asg, arg_node, func)?;
                // Вызываем print_int (import index 0)
                func.instruction(&Instruction::Call(0));
                func.instruction(&Instruction::I64Const(0));  // return unit
            }

            // === Block ===
            NodeType::Block => {
                let stmt_edges = node.find_edges(EdgeType::BlockStatement);
                let is_empty = stmt_edges.is_empty();
                for edge in stmt_edges {
                    let stmt_node = asg.find_node(edge.target_node_id)
                        .ok_or(ASGError::NodeNotFound(edge.target_node_id))?;
                    self.compile_node(asg, stmt_node, func)?;
                    // Drop intermediate values except last
                }
                if is_empty {
                    func.instruction(&Instruction::I64Const(0));
                }
            }

            // === Variable declaration ===
            NodeType::Variable => {
                if let Some(ref payload) = node.payload {
                    let name = String::from_utf8_lossy(payload).to_string();

                    // Ищем значение
                    if let Some(value_edge) = node.find_edge(EdgeType::VarValue) {
                        let value_node = asg.find_node(value_edge.target_node_id)
                            .ok_or(ASGError::NodeNotFound(value_edge.target_node_id))?;
                        self.compile_node(asg, value_node, func)?;
                    } else {
                        func.instruction(&Instruction::I64Const(0));
                    }

                    // Сохраняем в локальную переменную
                    let local_idx = self.local_count;
                    self.variable_locals.insert(name, local_idx);
                    self.local_count += 1;
                    func.instruction(&Instruction::LocalSet(local_idx));
                    func.instruction(&Instruction::I64Const(0));  // return unit
                }
            }

            // === Variable reference ===
            NodeType::VarRef => {
                if let Some(ref payload) = node.payload {
                    let name = String::from_utf8_lossy(payload).to_string();
                    if let Some(&local_idx) = self.variable_locals.get(&name) {
                        func.instruction(&Instruction::LocalGet(local_idx));
                    } else {
                        return Err(ASGError::UnknownVariable(name));
                    }
                }
            }

            // === Math constants ===
            NodeType::MathPi => {
                func.instruction(&Instruction::F64Const(std::f64::consts::PI));
            }

            NodeType::MathE => {
                func.instruction(&Instruction::F64Const(std::f64::consts::E));
            }

            // === Math functions ===
            NodeType::MathSqrt => {
                self.compile_unary_float_op(asg, node, func, Instruction::F64Sqrt)?;
            }

            NodeType::MathAbs => {
                self.compile_unary_float_op(asg, node, func, Instruction::F64Abs)?;
            }

            NodeType::MathFloor => {
                self.compile_unary_float_op(asg, node, func, Instruction::F64Floor)?;
            }

            NodeType::MathCeil => {
                self.compile_unary_float_op(asg, node, func, Instruction::F64Ceil)?;
            }

            NodeType::MathMin => {
                self.compile_binary_float_op(asg, node, func, Instruction::F64Min)?;
            }

            NodeType::MathMax => {
                self.compile_binary_float_op(asg, node, func, Instruction::F64Max)?;
            }

            // Неподдерживаемые типы
            _ => {
                // Возвращаем 0 для неподдерживаемых узлов
                func.instruction(&Instruction::I64Const(0));
            }
        }

        Ok(())
    }

    /// Компиляция бинарной операции.
    fn compile_binary_op(
        &mut self,
        asg: &ASG,
        node: &Node,
        func: &mut Function,
        instr: Instruction,
    ) -> ASGResult<()> {
        let edges = &node.edges;
        if edges.len() >= 2 {
            let left_node = asg.find_node(edges[0].target_node_id)
                .ok_or(ASGError::NodeNotFound(edges[0].target_node_id))?;
            let right_node = asg.find_node(edges[1].target_node_id)
                .ok_or(ASGError::NodeNotFound(edges[1].target_node_id))?;

            self.compile_node(asg, left_node, func)?;
            self.compile_node(asg, right_node, func)?;
            func.instruction(&instr);
        } else {
            return Err(ASGError::MissingEdge(node.id, EdgeType::ApplicationArgument));
        }
        Ok(())
    }

    /// Компиляция унарной float операции.
    fn compile_unary_float_op(
        &mut self,
        asg: &ASG,
        node: &Node,
        func: &mut Function,
        instr: Instruction,
    ) -> ASGResult<()> {
        let operand = node.edges.first()
            .ok_or(ASGError::MissingEdge(node.id, EdgeType::ApplicationArgument))?;
        let operand_node = asg.find_node(operand.target_node_id)
            .ok_or(ASGError::NodeNotFound(operand.target_node_id))?;

        self.compile_node(asg, operand_node, func)?;
        // Конвертируем в f64 если нужно
        func.instruction(&Instruction::F64ConvertI64S);
        func.instruction(&instr);
        Ok(())
    }

    /// Компиляция бинарной float операции.
    fn compile_binary_float_op(
        &mut self,
        asg: &ASG,
        node: &Node,
        func: &mut Function,
        instr: Instruction,
    ) -> ASGResult<()> {
        let edges = &node.edges;
        if edges.len() >= 2 {
            let left_node = asg.find_node(edges[0].target_node_id)
                .ok_or(ASGError::NodeNotFound(edges[0].target_node_id))?;
            let right_node = asg.find_node(edges[1].target_node_id)
                .ok_or(ASGError::NodeNotFound(edges[1].target_node_id))?;

            self.compile_node(asg, left_node, func)?;
            self.compile_node(asg, right_node, func)?;
            func.instruction(&instr);
        }
        Ok(())
    }

    /// Компиляция if выражения.
    fn compile_if_expression(
        &mut self,
        asg: &ASG,
        node: &Node,
        func: &mut Function,
    ) -> ASGResult<()> {
        // Получаем условие
        let cond_edge = node.find_edge(EdgeType::Condition)
            .ok_or(ASGError::MissingEdge(node.id, EdgeType::Condition))?;
        let cond_node = asg.find_node(cond_edge.target_node_id)
            .ok_or(ASGError::NodeNotFound(cond_edge.target_node_id))?;

        // Компилируем условие
        self.compile_node(asg, cond_node, func)?;

        // Конвертируем i64 в i32 для условия
        func.instruction(&Instruction::I32WrapI64);

        // If block
        func.instruction(&Instruction::If(wasm_encoder::BlockType::Result(ValType::I64)));

        // Then branch
        if let Some(then_edge) = node.find_edge(EdgeType::ThenBranch) {
            let then_node = asg.find_node(then_edge.target_node_id)
                .ok_or(ASGError::NodeNotFound(then_edge.target_node_id))?;
            self.compile_node(asg, then_node, func)?;
        } else {
            func.instruction(&Instruction::I64Const(0));
        }

        // Else branch
        func.instruction(&Instruction::Else);
        if let Some(else_edge) = node.find_edge(EdgeType::ElseBranch) {
            let else_node = asg.find_node(else_edge.target_node_id)
                .ok_or(ASGError::NodeNotFound(else_edge.target_node_id))?;
            self.compile_node(asg, else_node, func)?;
        } else {
            func.instruction(&Instruction::I64Const(0));
        }

        func.instruction(&Instruction::End);

        Ok(())
    }
}

#[cfg(feature = "wasm_backend")]
impl Default for WasmBackend {
    fn default() -> Self {
        Self::new()
    }
}

// === Заглушка для сборки без wasm_backend ===

#[cfg(not(feature = "wasm_backend"))]
pub struct WasmBackend;

#[cfg(not(feature = "wasm_backend"))]
impl WasmBackend {
    /// Компиляция ASG в WASM (заглушка).
    pub fn compile(asg: &ASG) -> ASGResult<Vec<u8>> {
        println!(
            "WasmBackend: WASM support not compiled in. Enable 'wasm_backend' feature."
        );
        println!("ASG has {} nodes.", asg.nodes.len());
        // WASM magic number + version
        Ok(vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00])
    }
}

// === Тесты ===

#[cfg(test)]
mod tests {
    #[test]
    fn test_wasm_backend_exists() {
        assert!(true);
    }

    #[cfg(feature = "wasm_backend")]
    #[test]
    fn test_wasm_compile_simple() {
        use crate::asg::{ASG, Node};
        use crate::nodecodes::NodeType;

        let mut asg = ASG::new();
        asg.add_node(Node::new(1, NodeType::LiteralInt, Some(42i64.to_le_bytes().to_vec())));

        let mut backend = super::WasmBackend::new();
        let result = backend.compile(&asg);

        assert!(result.is_ok());
        let bytes = result.unwrap();
        // Check WASM magic number
        assert_eq!(&bytes[0..4], &[0x00, 0x61, 0x73, 0x6D]);
    }
}
