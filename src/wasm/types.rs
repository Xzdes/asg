//! WASM Type definitions for ASG.
//!
//! Определения типов для представления ASG значений в WASM.

/// Тип значения в runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ValueType {
    /// Целое число i64
    Int = 0,
    /// Число с плавающей точкой f64
    Float = 1,
    /// Булево значение
    Bool = 2,
    /// Указатель на строку
    String = 3,
    /// Указатель на массив
    Array = 4,
    /// Указатель на словарь
    Dict = 5,
    /// Указатель на замыкание
    Closure = 6,
    /// Unit (пустое значение)
    Unit = 7,
}

/// Заголовок heap-объекта.
///
/// Структура в памяти:
/// ```text
/// Offset 0: type (1 byte) - ValueType
/// Offset 1: flags (1 byte) - флаги (marked, etc)
/// Offset 2-3: reserved (2 bytes)
/// Offset 4-7: refcount (4 bytes) - счётчик ссылок
/// Offset 8-15: size (8 bytes) - размер данных
/// Offset 16+: data - данные объекта
/// ```
pub const HEAP_HEADER_SIZE: u32 = 16;

/// Смещения в заголовке.
pub const OFFSET_TYPE: u32 = 0;
pub const OFFSET_FLAGS: u32 = 1;
pub const OFFSET_REFCOUNT: u32 = 4;
pub const OFFSET_SIZE: u32 = 8;
pub const OFFSET_DATA: u32 = 16;

/// Флаги объекта.
pub mod flags {
    /// Объект помечен для GC
    pub const MARKED: u8 = 0x01;
    /// Объект закреплён (не удаляется)
    pub const PINNED: u8 = 0x02;
}

/// Структура строки в памяти:
/// ```text
/// Header (16 bytes)
/// Data: UTF-8 bytes + null terminator
/// ```
pub const STRING_DATA_OFFSET: u32 = OFFSET_DATA;

/// Структура массива в памяти:
/// ```text
/// Header (16 bytes)
/// Offset 16-19: length (4 bytes)
/// Offset 20-23: capacity (4 bytes)
/// Offset 24+: elements (8 bytes each - tagged values)
/// ```
pub const ARRAY_LENGTH_OFFSET: u32 = OFFSET_DATA;
pub const ARRAY_CAPACITY_OFFSET: u32 = OFFSET_DATA + 4;
pub const ARRAY_DATA_OFFSET: u32 = OFFSET_DATA + 8;
pub const ARRAY_ELEMENT_SIZE: u32 = 8;

/// Структура замыкания в памяти:
/// ```text
/// Header (16 bytes)
/// Offset 16-19: function index (4 bytes)
/// Offset 20-23: env count (4 bytes)
/// Offset 24+: captured values (8 bytes each)
/// ```
pub const CLOSURE_FUNC_OFFSET: u32 = OFFSET_DATA;
pub const CLOSURE_ENV_COUNT_OFFSET: u32 = OFFSET_DATA + 4;
pub const CLOSURE_ENV_DATA_OFFSET: u32 = OFFSET_DATA + 8;

/// Tagged value representation.
///
/// Используем NaN-boxing для эффективного представления:
/// - Если верхние 16 бит = 0xFFFF, это указатель (нижние 48 бит)
/// - Иначе это f64 или специальное значение
///
/// Альтернативно, используем простую схему:
/// - Bits 0-2: tag (type)
/// - Bits 3-63: value or pointer
pub const TAG_INT: u64 = 0;
pub const TAG_FLOAT: u64 = 1;
pub const TAG_BOOL: u64 = 2;
pub const TAG_PTR: u64 = 3;
pub const TAG_UNIT: u64 = 7;

pub const TAG_BITS: u64 = 3;
pub const TAG_MASK: u64 = 0x7;
pub const VALUE_SHIFT: u64 = 3;

/// Создать tagged int.
#[inline]
pub const fn tag_int(value: i64) -> u64 {
    ((value as u64) << VALUE_SHIFT) | TAG_INT
}

/// Создать tagged pointer.
#[inline]
pub const fn tag_ptr(ptr: u32) -> u64 {
    ((ptr as u64) << VALUE_SHIFT) | TAG_PTR
}

/// Извлечь tag.
#[inline]
pub const fn get_tag(value: u64) -> u64 {
    value & TAG_MASK
}

/// Извлечь значение int.
#[inline]
pub const fn untag_int(value: u64) -> i64 {
    // Arithmetic shift right to preserve sign
    (value as i64) >> VALUE_SHIFT
}

/// Извлечь указатель.
#[inline]
pub const fn untag_ptr(value: u64) -> u32 {
    (value >> VALUE_SHIFT) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_int() {
        let tagged = tag_int(42);
        assert_eq!(get_tag(tagged), TAG_INT);
        assert_eq!(untag_int(tagged), 42);
    }

    #[test]
    fn test_tag_ptr() {
        let tagged = tag_ptr(0x1000);
        assert_eq!(get_tag(tagged), TAG_PTR);
        assert_eq!(untag_ptr(tagged), 0x1000);
    }

    #[test]
    fn test_negative_int() {
        let tagged = tag_int(-42);
        assert_eq!(get_tag(tagged), TAG_INT);
        assert_eq!(untag_int(tagged), -42);
    }
}
