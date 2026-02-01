//! Garbage Collection для WASM.
//!
//! Реализует reference counting с mark-sweep для циклических ссылок.
//!
//! # Архитектура
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    WASM Linear Memory                       │
//! ├─────────────────────────────────────────────────────────────┤
//! │ 0x0000 - 0x03FF │ Reserved (stack, globals)                 │
//! │ 0x0400 - 0x0FFF │ GC Metadata (free list, roots)            │
//! │ 0x1000+         │ Heap (managed objects)                    │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Object Layout
//!
//! ```text
//! ┌────────┬────────┬──────────┬──────────┬─────────────────┐
//! │ Type   │ Flags  │ Reserved │ Refcount │ Size            │
//! │ 1 byte │ 1 byte │ 2 bytes  │ 4 bytes  │ 8 bytes         │
//! ├────────┴────────┴──────────┴──────────┴─────────────────┤
//! │                    Data (variable)                       │
//! └──────────────────────────────────────────────────────────┘
//! ```

use super::types::*;

/// Конфигурация GC.
#[derive(Debug, Clone)]
pub struct GcConfig {
    /// Начало heap области
    pub heap_start: u32,
    /// Размер heap в байтах
    pub heap_size: u32,
    /// Порог для запуска GC (в байтах)
    pub gc_threshold: u32,
    /// Включить mark-sweep для циклов
    pub enable_mark_sweep: bool,
}

impl Default for GcConfig {
    fn default() -> Self {
        Self {
            heap_start: 0x1000,    // 4KB offset
            heap_size: 0x100000,   // 1MB heap
            gc_threshold: 0x80000, // 512KB threshold
            enable_mark_sweep: true,
        }
    }
}

/// Метаданные GC в памяти.
///
/// Располагаются в области 0x400-0x1000.
pub mod gc_metadata {
    /// Указатель на начало free list
    pub const FREE_LIST_HEAD: u32 = 0x400;
    /// Текущий heap pointer (bump allocator)
    pub const HEAP_PTR: u32 = 0x404;
    /// Количество выделенных байт
    pub const ALLOCATED_BYTES: u32 = 0x408;
    /// Количество объектов
    pub const OBJECT_COUNT: u32 = 0x40C;
    /// Начало таблицы корней (root set)
    pub const ROOTS_TABLE: u32 = 0x500;
    /// Максимальное количество корней
    pub const MAX_ROOTS: u32 = 256;
}

/// Генератор WASM кода для GC runtime.
pub struct GcCodegen {
    /// Конфигурация
    pub config: GcConfig,
}

impl GcCodegen {
    /// Создать новый генератор.
    pub fn new(config: GcConfig) -> Self {
        Self { config }
    }

    /// Генерирует WASM инструкции для инициализации GC.
    ///
    /// Должен вызываться в начале программы.
    pub fn generate_init(&self) -> Vec<WasmInstruction> {
        vec![
            // Инициализируем heap pointer
            WasmInstruction::I32Const(self.config.heap_start as i32),
            WasmInstruction::I32Store(gc_metadata::HEAP_PTR),
            // Инициализируем allocated_bytes = 0
            WasmInstruction::I32Const(0),
            WasmInstruction::I32Store(gc_metadata::ALLOCATED_BYTES),
            // Инициализируем object_count = 0
            WasmInstruction::I32Const(0),
            WasmInstruction::I32Store(gc_metadata::OBJECT_COUNT),
            // Инициализируем free_list = null
            WasmInstruction::I32Const(0),
            WasmInstruction::I32Store(gc_metadata::FREE_LIST_HEAD),
        ]
    }

    /// Генерирует функцию gc_alloc(size: i32) -> i32.
    ///
    /// Выделяет память для объекта заданного размера.
    /// Возвращает указатель или 0 при ошибке.
    pub fn generate_alloc_function(&self) -> GcFunction {
        GcFunction {
            name: "gc_alloc".to_string(),
            params: vec![("size", WasmType::I32)],
            result: Some(WasmType::I32),
            locals: vec![
                ("total_size", WasmType::I32),
                ("ptr", WasmType::I32),
                ("new_heap_ptr", WasmType::I32),
            ],
            body: vec![
                // total_size = size + HEAP_HEADER_SIZE
                WasmInstruction::LocalGet(0), // size
                WasmInstruction::I32Const(HEAP_HEADER_SIZE as i32),
                WasmInstruction::I32Add,
                WasmInstruction::LocalSet(1), // total_size
                // Проверяем free list
                WasmInstruction::Comment("Check free list".into()),
                WasmInstruction::I32Load(gc_metadata::FREE_LIST_HEAD),
                WasmInstruction::If(
                    vec![
                        // TODO: реализовать выделение из free list
                        WasmInstruction::I32Const(0),
                    ],
                    Some(vec![
                        // Bump allocation
                        WasmInstruction::Comment("Bump allocation".into()),
                        // ptr = heap_ptr
                        WasmInstruction::I32Load(gc_metadata::HEAP_PTR),
                        WasmInstruction::LocalSet(2), // ptr
                        // new_heap_ptr = ptr + total_size
                        WasmInstruction::LocalGet(2),
                        WasmInstruction::LocalGet(1),
                        WasmInstruction::I32Add,
                        WasmInstruction::LocalSet(3), // new_heap_ptr
                        // Проверяем границу heap
                        WasmInstruction::LocalGet(3),
                        WasmInstruction::I32Const(
                            (self.config.heap_start + self.config.heap_size) as i32,
                        ),
                        WasmInstruction::I32GtU,
                        WasmInstruction::If(
                            vec![
                                // Нужен GC или out of memory
                                WasmInstruction::Call("gc_collect".into()),
                                // Retry allocation после GC
                                WasmInstruction::I32Load(gc_metadata::HEAP_PTR),
                                WasmInstruction::LocalSet(2),
                                WasmInstruction::LocalGet(2),
                                WasmInstruction::LocalGet(1),
                                WasmInstruction::I32Add,
                                WasmInstruction::LocalSet(3),
                            ],
                            None,
                        ),
                        // Обновляем heap_ptr
                        WasmInstruction::LocalGet(3),
                        WasmInstruction::I32Store(gc_metadata::HEAP_PTR),
                        // Обновляем allocated_bytes
                        WasmInstruction::I32Load(gc_metadata::ALLOCATED_BYTES),
                        WasmInstruction::LocalGet(1),
                        WasmInstruction::I32Add,
                        WasmInstruction::I32Store(gc_metadata::ALLOCATED_BYTES),
                        // Инициализируем заголовок
                        // type = 0
                        WasmInstruction::LocalGet(2),
                        WasmInstruction::I32Const(0),
                        WasmInstruction::I32Store8(OFFSET_TYPE),
                        // flags = 0
                        WasmInstruction::LocalGet(2),
                        WasmInstruction::I32Const(0),
                        WasmInstruction::I32Store8(OFFSET_FLAGS),
                        // refcount = 1
                        WasmInstruction::LocalGet(2),
                        WasmInstruction::I32Const(1),
                        WasmInstruction::I32Store(OFFSET_REFCOUNT),
                        // size = size (параметр)
                        WasmInstruction::LocalGet(2),
                        WasmInstruction::LocalGet(0),
                        WasmInstruction::I64ExtendI32U,
                        WasmInstruction::I64Store(OFFSET_SIZE),
                        // Увеличиваем счётчик объектов
                        WasmInstruction::I32Load(gc_metadata::OBJECT_COUNT),
                        WasmInstruction::I32Const(1),
                        WasmInstruction::I32Add,
                        WasmInstruction::I32Store(gc_metadata::OBJECT_COUNT),
                        // Возвращаем ptr
                        WasmInstruction::LocalGet(2),
                    ]),
                ),
            ],
        }
    }

    /// Генерирует функцию gc_retain(ptr: i32).
    ///
    /// Увеличивает счётчик ссылок объекта.
    pub fn generate_retain_function(&self) -> GcFunction {
        GcFunction {
            name: "gc_retain".to_string(),
            params: vec![("ptr", WasmType::I32)],
            result: None,
            locals: vec![("refcount", WasmType::I32)],
            body: vec![
                // Проверяем null
                WasmInstruction::LocalGet(0),
                WasmInstruction::I32Eqz,
                WasmInstruction::BrIf(0), // return if null
                // refcount = ptr[OFFSET_REFCOUNT]
                WasmInstruction::LocalGet(0),
                WasmInstruction::I32Load(OFFSET_REFCOUNT),
                WasmInstruction::LocalSet(1),
                // refcount++
                WasmInstruction::LocalGet(0),
                WasmInstruction::LocalGet(1),
                WasmInstruction::I32Const(1),
                WasmInstruction::I32Add,
                WasmInstruction::I32Store(OFFSET_REFCOUNT),
            ],
        }
    }

    /// Генерирует функцию gc_release(ptr: i32).
    ///
    /// Уменьшает счётчик ссылок. При достижении 0 освобождает память.
    pub fn generate_release_function(&self) -> GcFunction {
        GcFunction {
            name: "gc_release".to_string(),
            params: vec![("ptr", WasmType::I32)],
            result: None,
            locals: vec![("refcount", WasmType::I32)],
            body: vec![
                // Проверяем null
                WasmInstruction::LocalGet(0),
                WasmInstruction::I32Eqz,
                WasmInstruction::BrIf(0), // return if null
                // refcount = ptr[OFFSET_REFCOUNT]
                WasmInstruction::LocalGet(0),
                WasmInstruction::I32Load(OFFSET_REFCOUNT),
                WasmInstruction::LocalSet(1),
                // refcount--
                WasmInstruction::LocalGet(1),
                WasmInstruction::I32Const(1),
                WasmInstruction::I32Sub,
                WasmInstruction::LocalTee(1),
                // if refcount == 0, free
                WasmInstruction::I32Eqz,
                WasmInstruction::If(
                    vec![
                        WasmInstruction::LocalGet(0),
                        WasmInstruction::Call("gc_free".into()),
                    ],
                    Some(vec![
                        // Сохраняем новый refcount
                        WasmInstruction::LocalGet(0),
                        WasmInstruction::LocalGet(1),
                        WasmInstruction::I32Store(OFFSET_REFCOUNT),
                    ]),
                ),
            ],
        }
    }

    /// Генерирует функцию gc_free(ptr: i32).
    ///
    /// Освобождает объект и рекурсивно освобождает вложенные ссылки.
    pub fn generate_free_function(&self) -> GcFunction {
        GcFunction {
            name: "gc_free".to_string(),
            params: vec![("ptr", WasmType::I32)],
            result: None,
            locals: vec![
                ("obj_type", WasmType::I32),
                ("size", WasmType::I32),
                ("i", WasmType::I32),
                ("child_ptr", WasmType::I32),
            ],
            body: vec![
                // Проверяем null
                WasmInstruction::LocalGet(0),
                WasmInstruction::I32Eqz,
                WasmInstruction::BrIf(0),
                // Получаем тип объекта
                WasmInstruction::LocalGet(0),
                WasmInstruction::I32Load8U(OFFSET_TYPE),
                WasmInstruction::LocalSet(1), // obj_type
                // Для массивов: освобождаем элементы
                WasmInstruction::LocalGet(1),
                WasmInstruction::I32Const(ValueType::Array as i32),
                WasmInstruction::I32Eq,
                WasmInstruction::If(
                    vec![
                        // Получаем длину массива
                        WasmInstruction::LocalGet(0),
                        WasmInstruction::I32Load(ARRAY_LENGTH_OFFSET),
                        WasmInstruction::LocalSet(2), // size = length
                        // Цикл по элементам
                        WasmInstruction::I32Const(0),
                        WasmInstruction::LocalSet(3), // i = 0
                        WasmInstruction::Loop(vec![
                            WasmInstruction::LocalGet(3),
                            WasmInstruction::LocalGet(2),
                            WasmInstruction::I32LtU,
                            WasmInstruction::If(
                                vec![
                                    // child_ptr = ptr + ARRAY_DATA_OFFSET + i * 8
                                    WasmInstruction::LocalGet(0),
                                    WasmInstruction::I32Const(ARRAY_DATA_OFFSET as i32),
                                    WasmInstruction::I32Add,
                                    WasmInstruction::LocalGet(3),
                                    WasmInstruction::I32Const(8),
                                    WasmInstruction::I32Mul,
                                    WasmInstruction::I32Add,
                                    WasmInstruction::I64Load(0),
                                    // Проверяем, является ли указателем
                                    WasmInstruction::Call("gc_release_if_ptr".into()),
                                    // i++
                                    WasmInstruction::LocalGet(3),
                                    WasmInstruction::I32Const(1),
                                    WasmInstruction::I32Add,
                                    WasmInstruction::LocalSet(3),
                                    WasmInstruction::Br(1), // continue loop
                                ],
                                None,
                            ),
                        ]),
                    ],
                    None,
                ),
                // Добавляем в free list
                WasmInstruction::Comment("Add to free list".into()),
                // next = free_list_head
                WasmInstruction::LocalGet(0),
                WasmInstruction::I32Load(gc_metadata::FREE_LIST_HEAD),
                WasmInstruction::I32Store(OFFSET_DATA), // используем data как next pointer
                // free_list_head = ptr
                WasmInstruction::LocalGet(0),
                WasmInstruction::I32Store(gc_metadata::FREE_LIST_HEAD),
                // Уменьшаем счётчик объектов
                WasmInstruction::I32Load(gc_metadata::OBJECT_COUNT),
                WasmInstruction::I32Const(1),
                WasmInstruction::I32Sub,
                WasmInstruction::I32Store(gc_metadata::OBJECT_COUNT),
            ],
        }
    }

    /// Генерирует функцию gc_collect().
    ///
    /// Запускает mark-sweep сборку мусора.
    pub fn generate_collect_function(&self) -> GcFunction {
        GcFunction {
            name: "gc_collect".to_string(),
            params: vec![],
            result: None,
            locals: vec![("ptr", WasmType::I32), ("i", WasmType::I32)],
            body: if self.config.enable_mark_sweep {
                vec![
                    WasmInstruction::Comment("Mark phase".into()),
                    // Помечаем все объекты как не-marked
                    WasmInstruction::Call("gc_unmark_all".into()),
                    // Помечаем достижимые из корней
                    WasmInstruction::I32Const(0),
                    WasmInstruction::LocalSet(1), // i = 0
                    WasmInstruction::Loop(vec![
                        WasmInstruction::LocalGet(1),
                        WasmInstruction::I32Const(gc_metadata::MAX_ROOTS as i32),
                        WasmInstruction::I32LtU,
                        WasmInstruction::If(
                            vec![
                                // ptr = roots[i]
                                WasmInstruction::I32Const(gc_metadata::ROOTS_TABLE as i32),
                                WasmInstruction::LocalGet(1),
                                WasmInstruction::I32Const(4),
                                WasmInstruction::I32Mul,
                                WasmInstruction::I32Add,
                                WasmInstruction::I32Load(0),
                                WasmInstruction::LocalTee(0),
                                // if ptr != 0, mark
                                WasmInstruction::If(
                                    vec![
                                        WasmInstruction::LocalGet(0),
                                        WasmInstruction::Call("gc_mark".into()),
                                    ],
                                    None,
                                ),
                                // i++
                                WasmInstruction::LocalGet(1),
                                WasmInstruction::I32Const(1),
                                WasmInstruction::I32Add,
                                WasmInstruction::LocalSet(1),
                                WasmInstruction::Br(1),
                            ],
                            None,
                        ),
                    ]),
                    WasmInstruction::Comment("Sweep phase".into()),
                    WasmInstruction::Call("gc_sweep".into()),
                ]
            } else {
                vec![WasmInstruction::Comment("GC disabled".into())]
            },
        }
    }

    /// Создаёт строку в heap.
    pub fn generate_string_new_function(&self) -> GcFunction {
        GcFunction {
            name: "string_new".to_string(),
            params: vec![("data_ptr", WasmType::I32), ("len", WasmType::I32)],
            result: Some(WasmType::I32),
            locals: vec![("obj", WasmType::I32), ("i", WasmType::I32)],
            body: vec![
                // Выделяем память: len + 1 (null terminator)
                WasmInstruction::LocalGet(1),
                WasmInstruction::I32Const(1),
                WasmInstruction::I32Add,
                WasmInstruction::Call("gc_alloc".into()),
                WasmInstruction::LocalSet(2), // obj
                // Устанавливаем тип = String
                WasmInstruction::LocalGet(2),
                WasmInstruction::I32Const(ValueType::String as i32),
                WasmInstruction::I32Store8(OFFSET_TYPE),
                // Копируем данные
                WasmInstruction::LocalGet(2),
                WasmInstruction::I32Const(STRING_DATA_OFFSET as i32),
                WasmInstruction::I32Add,
                WasmInstruction::LocalGet(0), // data_ptr
                WasmInstruction::LocalGet(1), // len
                WasmInstruction::MemoryCopy,
                // Добавляем null terminator
                WasmInstruction::LocalGet(2),
                WasmInstruction::I32Const(STRING_DATA_OFFSET as i32),
                WasmInstruction::I32Add,
                WasmInstruction::LocalGet(1),
                WasmInstruction::I32Add,
                WasmInstruction::I32Const(0),
                WasmInstruction::I32Store8(0),
                // Возвращаем obj
                WasmInstruction::LocalGet(2),
            ],
        }
    }

    /// Создаёт массив в heap.
    pub fn generate_array_new_function(&self) -> GcFunction {
        GcFunction {
            name: "array_new".to_string(),
            params: vec![("capacity", WasmType::I32)],
            result: Some(WasmType::I32),
            locals: vec![("obj", WasmType::I32)],
            body: vec![
                // Выделяем память: 8 (length + capacity) + capacity * 8 (elements)
                WasmInstruction::I32Const(8),
                WasmInstruction::LocalGet(0),
                WasmInstruction::I32Const(8),
                WasmInstruction::I32Mul,
                WasmInstruction::I32Add,
                WasmInstruction::Call("gc_alloc".into()),
                WasmInstruction::LocalSet(1), // obj
                // Устанавливаем тип = Array
                WasmInstruction::LocalGet(1),
                WasmInstruction::I32Const(ValueType::Array as i32),
                WasmInstruction::I32Store8(OFFSET_TYPE),
                // length = 0
                WasmInstruction::LocalGet(1),
                WasmInstruction::I32Const(0),
                WasmInstruction::I32Store(ARRAY_LENGTH_OFFSET),
                // capacity = capacity
                WasmInstruction::LocalGet(1),
                WasmInstruction::LocalGet(0),
                WasmInstruction::I32Store(ARRAY_CAPACITY_OFFSET),
                // Возвращаем obj
                WasmInstruction::LocalGet(1),
            ],
        }
    }

    /// Все GC функции.
    pub fn all_functions(&self) -> Vec<GcFunction> {
        vec![
            self.generate_alloc_function(),
            self.generate_retain_function(),
            self.generate_release_function(),
            self.generate_free_function(),
            self.generate_collect_function(),
            self.generate_string_new_function(),
            self.generate_array_new_function(),
        ]
    }
}

/// Тип WASM.
#[derive(Debug, Clone, Copy)]
pub enum WasmType {
    I32,
    I64,
    F32,
    F64,
}

/// Инструкция WASM (высокоуровневое представление).
#[derive(Debug, Clone)]
pub enum WasmInstruction {
    // Constants
    I32Const(i32),
    I64Const(i64),
    F64Const(f64),

    // Locals
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),

    // Memory
    I32Load(u32),
    I32Load8U(u32),
    I64Load(u32),
    I32Store(u32),
    I32Store8(u32),
    I64Store(u32),
    MemoryCopy,

    // Arithmetic
    I32Add,
    I32Sub,
    I32Mul,
    I64ExtendI32U,

    // Comparison
    I32Eq,
    I32Eqz,
    I32LtU,
    I32GtU,

    // Control
    If(Vec<WasmInstruction>, Option<Vec<WasmInstruction>>),
    Loop(Vec<WasmInstruction>),
    Block(Vec<WasmInstruction>),
    Br(u32),
    BrIf(u32),
    Call(String),
    Return,

    // Comments (for debugging, removed in final output)
    Comment(String),
}

/// Функция GC.
#[derive(Debug, Clone)]
pub struct GcFunction {
    pub name: String,
    pub params: Vec<(&'static str, WasmType)>,
    pub result: Option<WasmType>,
    pub locals: Vec<(&'static str, WasmType)>,
    pub body: Vec<WasmInstruction>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_config_default() {
        let config = GcConfig::default();
        assert_eq!(config.heap_start, 0x1000);
        assert_eq!(config.heap_size, 0x100000);
    }

    #[test]
    fn test_gc_codegen_init() {
        let config = GcConfig::default();
        let codegen = GcCodegen::new(config);
        let init = codegen.generate_init();
        assert!(!init.is_empty());
    }

    #[test]
    fn test_gc_functions() {
        let config = GcConfig::default();
        let codegen = GcCodegen::new(config);
        let functions = codegen.all_functions();
        assert_eq!(functions.len(), 7);
        assert_eq!(functions[0].name, "gc_alloc");
        assert_eq!(functions[1].name, "gc_retain");
        assert_eq!(functions[2].name, "gc_release");
    }
}
