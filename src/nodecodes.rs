//! Определения кодов для узлов и ребер.

use serde::{Deserialize, Serialize};

/// Типы узлов ASG
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    // === Литералы ===
    /// Целочисленный литерал (i64, payload: 8 bytes little-endian)
    LiteralInt,
    /// Литерал с плавающей точкой (f64, payload: 8 bytes little-endian)
    LiteralFloat,
    /// Булевый литерал (payload: 1 byte, 0=false, 1=true)
    LiteralBool,
    /// Строковый литерал (payload: UTF-8 bytes)
    LiteralString,
    /// Unit значение (нет payload)
    LiteralUnit,
    /// Тензорный литерал (payload: f32 little-endian)
    LiteralTensor,

    // === Арифметические операции ===
    /// Сложение (Add) - для целых чисел
    BinaryOperation,
    /// Вычитание
    Sub,
    /// Умножение
    Mul,
    /// Деление
    Div,
    /// Целочисленное деление (//)
    IntDiv,
    /// Остаток от деления
    Mod,
    /// Унарный минус
    Neg,

    // === Операции сравнения ===
    /// Равенство (==)
    Eq,
    /// Неравенство (!=)
    Ne,
    /// Меньше (<)
    Lt,
    /// Меньше или равно (<=)
    Le,
    /// Больше (>)
    Gt,
    /// Больше или равно (>=)
    Ge,

    // === Логические операции ===
    /// Логическое И (&&)
    And,
    /// Логическое ИЛИ (||)
    Or,
    /// Логическое НЕ (!)
    Not,

    // === Управляющие конструкции ===
    /// Условное выражение if/else
    If,
    /// Блок выражений (последовательное выполнение)
    Block,
    /// Цикл
    Loop,
    /// Выход из цикла
    Break,
    /// Продолжение цикла
    Continue,
    /// Возврат из функции
    Return,

    // === Функции ===
    /// Определение функции (payload: имя функции UTF-8)
    Function,
    /// Вызов функции
    Call,
    /// Лямбда-выражение
    Lambda,
    /// Параметр функции (payload: имя параметра UTF-8)
    Parameter,

    // === Переменные ===
    /// Объявление переменной (payload: имя переменной UTF-8)
    Variable,
    /// Ссылка на переменную (payload: имя переменной UTF-8)
    VarRef,
    /// Присваивание
    Assign,

    // === Тензорные операции (ML) ===
    /// Сложение тензоров
    TensorAdd,
    /// Умножение тензоров (поэлементное)
    TensorMul,
    /// Матричное умножение
    TensorMatMul,
    /// Градиент (для автодифференцирования)
    TensorGrad,

    // === Структуры данных ===
    /// Запись/структура
    Record,
    /// Доступ к полю записи (payload: имя поля UTF-8)
    RecordField,
    /// Массив
    Array,
    /// Индексирование массива
    ArrayIndex,
    /// Длина массива
    ArrayLength,
    /// Последний элемент массива
    ArrayLast,
    /// Установка элемента массива
    ArraySetIndex,
    /// map по массиву: (map arr fn)
    ArrayMap,
    /// filter по массиву: (filter arr fn)
    ArrayFilter,
    /// reduce по массиву: (reduce arr init fn)
    ArrayReduce,
    /// Создание диапазона: (range start end) или (range start end step)
    Range,
    /// Цикл for: (for var iterable body)
    For,
    /// Обратный массив: (reverse arr)
    ArrayReverse,
    /// Сортировка массива: (sort arr)
    ArraySort,
    /// Сумма элементов: (sum arr)
    ArraySum,
    /// Произведение элементов: (product arr)
    ArrayProduct,
    /// Есть ли элемент: (contains arr elem)
    ArrayContains,
    /// Найти индекс: (index-of arr elem)
    ArrayIndexOf,
    /// Взять первые n: (take arr n)
    ArrayTake,
    /// Пропустить первые n: (drop arr n)
    ArrayDrop,
    /// Добавить элемент в конец: (append arr elem)
    ArrayAppend,
    /// Объединить два массива: (array-concat arr1 arr2)
    ArrayConcat,
    /// Срез массива: (slice arr start end)
    ArraySlice,

    // === Словари (Dict) ===
    /// Создание словаря: (dict k1 v1 k2 v2 ...)
    Dict,
    /// Получение значения: (dict-get d key)
    DictGet,
    /// Установка значения: (dict-set d key value)
    DictSet,
    /// Проверка наличия ключа: (dict-has d key)
    DictHas,
    /// Удаление ключа: (dict-remove d key)
    DictRemove,
    /// Получение всех ключей: (dict-keys d)
    DictKeys,
    /// Получение всех значений: (dict-values d)
    DictValues,
    /// Слияние словарей: (dict-merge d1 d2)
    DictMerge,
    /// Размер словаря: (dict-size d)
    DictSize,

    // === Pipe и Composition ===
    /// Pipe operator: (|> value fn1 fn2 ...)
    Pipe,
    /// Compose functions: (compose fn1 fn2 ...)
    Compose,

    // === Destructuring ===
    /// Деструктуризация в let: (let (a b c) expr)
    LetDestructure,

    // === List Comprehension ===
    /// List comprehension: (list-comp expr var iter [condition])
    ListComprehension,

    // === Lazy Sequences ===
    /// Iterate: (iterate f init) -> lazy [init, f(init), f(f(init)), ...]
    Iterate,
    /// Repeat: (repeat val) -> lazy [val, val, val, ...]
    Repeat,
    /// Cycle: (cycle arr) -> lazy [a,b,c,a,b,c,...]
    Cycle,
    /// Lazy range: (lazy-range start end [step])
    LazyRange,
    /// Take from lazy: (take-lazy n seq)
    TakeLazy,
    /// Lazy map: (lazy-map fn seq)
    LazyMap,
    /// Lazy filter: (lazy-filter fn seq)
    LazyFilter,
    /// Collect lazy to array: (collect seq)
    Collect,

    // === Строковые операции ===
    /// Конкатенация строк: (concat s1 s2)
    StringConcat,
    /// Длина строки: (str-length s)
    StringLength,
    /// Подстрока: (substring s start end)
    StringSubstring,
    /// Разбиение строки: (str-split s delimiter)
    StringSplit,
    /// Объединение массива строк: (str-join arr delimiter)
    StringJoin,
    /// Содержит ли подстроку: (str-contains s substr)
    StringContains,
    /// Замена подстроки: (str-replace s from to)
    StringReplace,
    /// Преобразование в строку: (to-string value)
    ToString,
    /// Преобразование в число: (parse-int s), (parse-float s)
    ParseInt,
    ParseFloat,
    /// Trim пробелов: (str-trim s)
    StringTrim,
    /// Uppercase/lowercase: (str-upper s), (str-lower s)
    StringUpper,
    StringLower,

    // === Математические функции ===
    MathSqrt,
    MathSin,
    MathCos,
    MathTan,
    MathAsin,
    MathAcos,
    MathAtan,
    MathExp,
    MathLn,
    MathLog10,
    MathPow,
    MathAbs,
    MathFloor,
    MathCeil,
    MathRound,
    MathMin,
    MathMax,
    MathPi,
    MathE,

    // === Обработка ошибок ===
    /// Try-catch блок: (try expr (catch e handler))
    TryCatch,
    /// Выброс ошибки: (throw message)
    Throw,
    /// Проверка на ошибку: (is-error value)
    IsError,
    /// Получение сообщения ошибки: (error-message err)
    ErrorMessage,

    // === Алгебраические типы данных ===
    /// Конструктор варианта ADT (payload: имя варианта UTF-8)
    ADTConstructor,
    /// Pattern matching
    Match,
    /// Ветка match
    MatchArm,

    // === Ввод/вывод ===
    /// Печать значения
    Print,
    /// Чтение строки с консоли: (input) или (input prompt)
    Input,
    /// Чтение целого числа: (input-int) или (input-int prompt)
    InputInt,
    /// Чтение float: (input-float) или (input-float prompt)
    InputFloat,
    /// Очистка экрана: (clear-screen)
    ClearScreen,
    /// Чтение файла: (read-file path)
    ReadFile,
    /// Запись в файл: (write-file path content)
    WriteFile,
    /// Добавление в файл: (append-file path content)
    AppendFile,
    /// Проверка существования файла: (file-exists path)
    FileExists,

    // === Эффекты ===
    /// Выполнение эффекта
    EffectPerform,
    /// Обработка эффекта
    EffectHandle,

    // === Модули ===
    /// Модуль (payload: имя модуля UTF-8)
    Module,
    /// Импорт (payload: путь импорта UTF-8)
    Import,
    /// Экспорт
    Export,

    // === Аннотации ===
    /// Явная аннотация типа
    TypeAnnotation,

    // === Web/HTTP ===
    /// HTTP сервер: (http-serve port handler)
    HttpServe,
    /// HTTP ответ: (http-response status headers body)
    HttpResponse,
    /// HTML тег: (html ...), (div ...), (span ...), etc.
    HtmlElement,
    /// HTML атрибут
    HtmlAttr,
    /// JSON кодирование: (json-encode value)
    JsonEncode,
    /// JSON декодирование: (json-decode string)
    JsonDecode,

    // === Native GUI ===
    /// Создание окна: (window title width height body)
    GuiWindow,
    /// Кнопка: (button text onclick)
    GuiButton,
    /// Текстовое поле: (text-field id value onchange)
    GuiTextField,
    /// Метка: (label text)
    GuiLabel,
    /// Вертикальный layout: (vbox ...)
    GuiVBox,
    /// Горизонтальный layout: (hbox ...)
    GuiHBox,
    /// Canvas для рисования: (canvas width height ondraw)
    GuiCanvas,
    /// Запуск GUI приложения: (gui-run window)
    GuiRun,
}

/// Типы рёбер ASG
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeType {
    // === Общие аргументы ===
    /// Аргумент применения (универсальный)
    ApplicationArgument,
    /// Первый операнд бинарной операции
    FirstOperand,
    /// Второй операнд бинарной операции
    SecondOperand,

    // === Управляющий поток ===
    /// Условие (для If, Loop, Match)
    Condition,
    /// Ветка "then" (для If)
    ThenBranch,
    /// Ветка "else" (для If)
    ElseBranch,
    /// Тело цикла
    LoopBody,
    /// Инициализация цикла
    LoopInit,
    /// Шаг цикла
    LoopStep,
    /// Выражение в блоке
    BlockStatement,
    /// Try-выражение
    TryBody,
    /// Catch-обработчик
    CatchHandler,
    /// Имя переменной для ошибки
    CatchVariable,

    // === Функции ===
    /// Тело функции
    FunctionBody,
    /// Параметр функции
    FunctionParameter,
    /// Целевая функция для вызова
    CallTarget,
    /// Аргумент вызова функции
    CallArgument,
    /// Возвращаемое значение
    ReturnValue,

    // === Переменные ===
    /// Объявление переменной
    VarDeclaration,
    /// Значение переменной
    VarValue,
    /// Цель присваивания
    AssignTarget,
    /// Присваиваемое значение
    AssignValue,

    // === Структуры ===
    /// Определение поля записи
    RecordFieldDef,
    /// Доступ к полю
    RecordFieldAccess,
    /// Элемент массива
    ArrayElement,
    /// Выражение индекса
    ArrayIndexExpr,
    /// Массив для map/filter/reduce
    SourceArray,
    /// Функция-трансформер для map
    MapFunction,
    /// Предикат для filter
    FilterPredicate,
    /// Начальное значение для reduce
    ReduceInit,
    /// Функция-аккумулятор для reduce
    ReduceFunction,

    // === Pattern Matching ===
    /// Субъект сопоставления (что матчим)
    MatchSubject,
    /// Паттерн
    MatchPattern,
    /// Тело ветки match
    MatchBody,

    // === Типы ===
    /// Связь с аннотацией типа
    TypeAnnotationEdge,

    // === Эффекты ===
    /// Тип эффекта
    EffectType,
    /// Обработчик эффекта
    EffectHandler,

    // === Модули ===
    /// Содержимое модуля
    ModuleContent,
    /// Источник импорта
    ImportSource,
}