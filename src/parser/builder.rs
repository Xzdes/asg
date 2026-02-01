//! Построитель ASG из S-Expression.

use super::error::ParseError;
use super::parser::{Atom, SExpr};
use super::token::Spanned;
use crate::asg::{Edge, Node, NodeID, ASG};
use crate::nodecodes::{EdgeType, NodeType};

/// Построитель ASG из S-Expression.
pub struct AsgBuilder {
    asg: ASG,
    next_id: NodeID,
}

impl AsgBuilder {
    /// Создать новый построитель.
    pub fn new() -> Self {
        Self {
            asg: ASG::new(),
            next_id: 1,
        }
    }

    /// Построить ASG из списка S-выражений.
    /// Возвращает ASG и список ID корневых узлов (top-level expressions).
    pub fn build(mut self, exprs: Vec<SExpr>) -> Result<(ASG, Vec<NodeID>), ParseError> {
        let mut root_ids = Vec::new();
        for expr in exprs {
            let root_id = self.build_expr(&expr)?;
            root_ids.push(root_id);
        }
        Ok((self.asg, root_ids))
    }

    /// Построить ASG из одного S-выражения.
    pub fn build_single(mut self, expr: &SExpr) -> Result<(ASG, NodeID), ParseError> {
        let root_id = self.build_expr(expr)?;
        Ok((self.asg, root_id))
    }

    /// Получить следующий ID узла.
    fn alloc_id(&mut self) -> NodeID {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Построить узел из S-выражения.
    fn build_expr(&mut self, expr: &SExpr) -> Result<NodeID, ParseError> {
        match expr {
            SExpr::Atom(atom) => self.build_atom(atom),
            SExpr::List(list) => self.build_list(list),
        }
    }

    /// Построить атомарный узел.
    fn build_atom(&mut self, atom: &Spanned<Atom>) -> Result<NodeID, ParseError> {
        let id = self.alloc_id();
        let span = atom.span;

        let node = match &atom.value {
            Atom::Int(n) => Node::with_span(
                id,
                NodeType::LiteralInt,
                Some(n.to_le_bytes().to_vec()),
                span,
            ),
            Atom::Float(f) => Node::with_span(
                id,
                NodeType::LiteralFloat,
                Some(f.to_le_bytes().to_vec()),
                span,
            ),
            Atom::String(s) => Node::with_span(
                id,
                NodeType::LiteralString,
                Some(s.as_bytes().to_vec()),
                span,
            ),
            Atom::Ident(s) => {
                // Специальные идентификаторы
                match s.as_str() {
                    "true" => Node::with_span(id, NodeType::LiteralBool, Some(vec![1]), span),
                    "false" => Node::with_span(id, NodeType::LiteralBool, Some(vec![0]), span),
                    _ => {
                        // Ссылка на переменную
                        Node::with_span(id, NodeType::VarRef, Some(s.as_bytes().to_vec()), span)
                    }
                }
            }
            Atom::Symbol(_) => {
                return Err(ParseError::InvalidLiteral {
                    span: atom.span,
                    message: "Unexpected symbol outside of list".to_string(),
                });
            }
        };

        self.asg.add_node(node);
        Ok(id)
    }

    /// Построить узел из списка.
    fn build_list(&mut self, list: &Spanned<Vec<SExpr>>) -> Result<NodeID, ParseError> {
        let elements = &list.value;

        // Пустой список = Unit
        if elements.is_empty() {
            let id = self.alloc_id();
            self.asg
                .add_node(Node::with_span(id, NodeType::LiteralUnit, None, list.span));
            return Ok(id);
        }

        // Получаем имя формы
        let first = &elements[0];
        let form_name = first
            .as_ident()
            .or_else(|| first.as_symbol())
            .ok_or_else(|| ParseError::InvalidLiteral {
                span: first.span(),
                message: "Expected identifier or symbol as first element".to_string(),
            })?;

        // Диспетчеризация по форме
        match form_name {
            // Арифметика (variadic + и *)
            "+" => self.build_variadic_add(elements, list.span),
            "-" => self.build_binop_or_unary(elements, NodeType::Sub, NodeType::Neg, list.span),
            "*" => self.build_variadic_mul(elements, list.span),
            "/" => self.build_binop(elements, NodeType::Div, list.span),
            "//" => self.build_binop(elements, NodeType::IntDiv, list.span),
            "%" => self.build_binop(elements, NodeType::Mod, list.span),
            "neg" => self.build_unop(elements, NodeType::Neg, list.span),

            // Сравнение
            "==" => self.build_binop(elements, NodeType::Eq, list.span),
            "!=" => self.build_binop(elements, NodeType::Ne, list.span),
            "<" => self.build_binop(elements, NodeType::Lt, list.span),
            "<=" => self.build_binop(elements, NodeType::Le, list.span),
            ">" => self.build_binop(elements, NodeType::Gt, list.span),
            ">=" => self.build_binop(elements, NodeType::Ge, list.span),

            // Логика
            "and" | "&&" => self.build_binop(elements, NodeType::And, list.span),
            "or" | "||" => self.build_binop(elements, NodeType::Or, list.span),
            "not" | "!" => self.build_unop(elements, NodeType::Not, list.span),

            // Переменные
            "let" => self.build_let(elements, list.span),
            "set" => self.build_set(elements, list.span),

            // Управление
            "if" => self.build_if(elements, list.span),
            "do" => self.build_do(elements, list.span),
            "loop" => self.build_loop(elements, list.span),
            "while" => self.build_while(elements, list.span),
            "break" => self.build_break(elements, list.span),
            "continue" => self.build_continue(list.span),
            "return" => self.build_return(elements, list.span),

            // Функции
            "fn" => self.build_fn(elements, list.span),
            "lambda" => self.build_lambda(elements, list.span),

            // Структуры данных
            "array" => self.build_array(elements, list.span),
            "index" => self.build_index(elements, list.span),
            "nth" => self.build_index(elements, list.span), // alias
            "first" => self.build_nth_shorthand(elements, 0, list.span),
            "second" => self.build_nth_shorthand(elements, 1, list.span),
            "third" => self.build_nth_shorthand(elements, 2, list.span),
            "last" => self.build_last(elements, list.span),
            "length" => self.build_length(elements, list.span),
            "set-index" => self.build_set_index(elements, list.span),
            "map" => self.build_map(elements, list.span),
            "filter" => self.build_filter(elements, list.span),
            "reduce" => self.build_reduce(elements, list.span),
            "record" => self.build_record(elements, list.span),
            "field" => self.build_field(elements, list.span),

            // I/O
            "print" => self.build_print(elements, list.span),
            "input" => self.build_input(elements, NodeType::Input, list.span),
            "input-int" => self.build_input(elements, NodeType::InputInt, list.span),
            "input-float" => self.build_input(elements, NodeType::InputFloat, list.span),
            "clear-screen" => self.build_constant(NodeType::ClearScreen),
            "read-file" => self.build_unary(elements, NodeType::ReadFile, list.span),
            "write-file" => self.build_binop(elements, NodeType::WriteFile, list.span),
            "append-file" => self.build_binop(elements, NodeType::AppendFile, list.span),
            "file-exists" => self.build_unary(elements, NodeType::FileExists, list.span),

            // Строковые операции
            "concat" => self.build_binop(elements, NodeType::StringConcat, list.span),
            "str-length" => self.build_unary(elements, NodeType::StringLength, list.span),
            "substring" => self.build_substring(elements, list.span),
            "str-split" => self.build_binop(elements, NodeType::StringSplit, list.span),
            "str-join" => self.build_binop(elements, NodeType::StringJoin, list.span),
            "str-contains" => self.build_binop(elements, NodeType::StringContains, list.span),
            "str-replace" => self.build_str_replace(elements, list.span),
            "to-string" | "str" => self.build_unary(elements, NodeType::ToString, list.span),
            "parse-int" => self.build_unary(elements, NodeType::ParseInt, list.span),
            "parse-float" => self.build_unary(elements, NodeType::ParseFloat, list.span),
            "str-trim" => self.build_unary(elements, NodeType::StringTrim, list.span),
            "str-upper" => self.build_unary(elements, NodeType::StringUpper, list.span),
            "str-lower" => self.build_unary(elements, NodeType::StringLower, list.span),

            // Math functions
            "sqrt" => self.build_unary(elements, NodeType::MathSqrt, list.span),
            "sin" => self.build_unary(elements, NodeType::MathSin, list.span),
            "cos" => self.build_unary(elements, NodeType::MathCos, list.span),
            "tan" => self.build_unary(elements, NodeType::MathTan, list.span),
            "asin" => self.build_unary(elements, NodeType::MathAsin, list.span),
            "acos" => self.build_unary(elements, NodeType::MathAcos, list.span),
            "atan" => self.build_unary(elements, NodeType::MathAtan, list.span),
            "exp" => self.build_unary(elements, NodeType::MathExp, list.span),
            "ln" => self.build_unary(elements, NodeType::MathLn, list.span),
            "log10" => self.build_unary(elements, NodeType::MathLog10, list.span),
            "pow" => self.build_binop(elements, NodeType::MathPow, list.span),
            "abs" => self.build_unary(elements, NodeType::MathAbs, list.span),
            "floor" => self.build_unary(elements, NodeType::MathFloor, list.span),
            "ceil" => self.build_unary(elements, NodeType::MathCeil, list.span),
            "round" => self.build_unary(elements, NodeType::MathRound, list.span),
            "min" => self.build_binop(elements, NodeType::MathMin, list.span),
            "max" => self.build_binop(elements, NodeType::MathMax, list.span),
            "PI" => self.build_constant(NodeType::MathPi),
            "E" => self.build_constant(NodeType::MathE),

            // Error handling
            "try" => self.build_try_catch(elements, list.span),
            "throw" => self.build_unary(elements, NodeType::Throw, list.span),
            "is-error" => self.build_unary(elements, NodeType::IsError, list.span),
            "error-message" => self.build_unary(elements, NodeType::ErrorMessage, list.span),

            // Pattern matching
            "match" => self.build_match(elements, list.span),

            // Range and iterators
            "range" => self.build_range(elements, list.span),
            "for" => self.build_for(elements, list.span),
            "list-comp" => self.build_list_comp(elements, list.span),

            // Lazy sequences
            "iterate" => self.build_binop(elements, NodeType::Iterate, list.span),
            "repeat" => self.build_unary(elements, NodeType::Repeat, list.span),
            "cycle" => self.build_unary(elements, NodeType::Cycle, list.span),
            "lazy-range" => self.build_lazy_range(elements, list.span),
            "take-lazy" => self.build_binop(elements, NodeType::TakeLazy, list.span),
            "lazy-map" => self.build_binop(elements, NodeType::LazyMap, list.span),
            "lazy-filter" => self.build_binop(elements, NodeType::LazyFilter, list.span),
            "collect" => self.build_unary(elements, NodeType::Collect, list.span),

            "reverse" => self.build_unary(elements, NodeType::ArrayReverse, list.span),
            "sort" => self.build_unary(elements, NodeType::ArraySort, list.span),
            "sum" => self.build_unary(elements, NodeType::ArraySum, list.span),
            "product" => self.build_unary(elements, NodeType::ArrayProduct, list.span),
            "contains" => self.build_binop(elements, NodeType::ArrayContains, list.span),
            "index-of" => self.build_binop(elements, NodeType::ArrayIndexOf, list.span),
            "take" => self.build_binop(elements, NodeType::ArrayTake, list.span),
            "drop" => self.build_binop(elements, NodeType::ArrayDrop, list.span),
            "append" => self.build_binop(elements, NodeType::ArrayAppend, list.span),
            "array-concat" => self.build_binop(elements, NodeType::ArrayConcat, list.span),
            "slice" => self.build_ternary(elements, NodeType::ArraySlice, list.span),

            // Dict operations
            "dict" => self.build_dict(elements, list.span),
            "dict-get" => self.build_binop(elements, NodeType::DictGet, list.span),
            "dict-set" => self.build_ternary(elements, NodeType::DictSet, list.span),
            "dict-has" => self.build_binop(elements, NodeType::DictHas, list.span),
            "dict-remove" => self.build_binop(elements, NodeType::DictRemove, list.span),
            "dict-keys" => self.build_unary(elements, NodeType::DictKeys, list.span),
            "dict-values" => self.build_unary(elements, NodeType::DictValues, list.span),
            "dict-merge" => self.build_binop(elements, NodeType::DictMerge, list.span),
            "dict-size" => self.build_unary(elements, NodeType::DictSize, list.span),

            // Pipe and composition
            "|>" => self.build_pipe(elements, list.span),
            "pipe" => self.build_pipe(elements, list.span),
            "compose" => self.build_compose(elements, list.span),

            // Тензоры
            "tensor" => self.build_tensor(elements, list.span),
            "tensor-add" => self.build_binop(elements, NodeType::TensorAdd, list.span),
            "tensor-mul" => self.build_binop(elements, NodeType::TensorMul, list.span),
            "tensor-matmul" => self.build_binop(elements, NodeType::TensorMatMul, list.span),

            // Модули
            "module" => self.build_module(elements, list.span),
            "import" => self.build_import(elements, list.span),

            // Web/HTTP
            "http-serve" => self.build_binop(elements, NodeType::HttpServe, list.span),
            "http-response" => self.build_http_response(elements, list.span),
            "json-encode" => self.build_unary(elements, NodeType::JsonEncode, list.span),
            "json-decode" => self.build_unary(elements, NodeType::JsonDecode, list.span),

            // HTML elements (html-input instead of input to avoid conflict with input function)
            "html" | "head" | "body" | "div" | "span" | "p" | "h1" | "h2" | "h3" | "ul" | "ol"
            | "li" | "a" | "img" | "form" | "html-input" | "html-button" | "table" | "tr"
            | "td" | "th" | "style" | "script" | "meta" | "link" | "title" | "header"
            | "footer" | "nav" | "main" | "section" | "article" | "textarea" | "select"
            | "option" | "label" | "br" | "hr" => {
                self.build_html_element(form_name, elements, list.span)
            }

            // Native GUI
            "window" => self.build_gui_window(elements, list.span),
            "gui-button" => self.build_gui_button(elements, list.span),
            "text-field" => self.build_gui_text_field(elements, list.span),
            "gui-label" => self.build_unary(elements, NodeType::GuiLabel, list.span),
            "vbox" => self.build_gui_container(elements, NodeType::GuiVBox, list.span),
            "hbox" => self.build_gui_container(elements, NodeType::GuiHBox, list.span),
            "canvas" => self.build_gui_canvas(elements, list.span),
            "gui-run" => self.build_unary(elements, NodeType::GuiRun, list.span),

            // По умолчанию — вызов функции
            _ => self.build_call(elements, list.span),
        }
    }

    /// Построить variadic сложение: (+ a b c ...) = (+ (+ a b) c) ...
    fn build_variadic_add(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() < 3 {
            return Err(ParseError::wrong_arity(
                span,
                "+",
                "at least 2",
                elements.len() - 1,
            ));
        }

        let mut result = self.build_expr(&elements[1])?;
        for elem in &elements[2..] {
            let right = self.build_expr(elem)?;
            let id = self.alloc_id();
            self.asg.add_node(Node::with_edges(
                id,
                NodeType::BinaryOperation,
                None,
                vec![
                    Edge::new(EdgeType::FirstOperand, result),
                    Edge::new(EdgeType::SecondOperand, right),
                ],
            ));
            result = id;
        }
        Ok(result)
    }

    /// Построить variadic умножение: (* a b c ...) = (* (* a b) c) ...
    fn build_variadic_mul(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() < 3 {
            return Err(ParseError::wrong_arity(
                span,
                "*",
                "at least 2",
                elements.len() - 1,
            ));
        }

        let mut result = self.build_expr(&elements[1])?;
        for elem in &elements[2..] {
            let right = self.build_expr(elem)?;
            let id = self.alloc_id();
            self.asg.add_node(Node::with_edges(
                id,
                NodeType::Mul,
                None,
                vec![
                    Edge::new(EdgeType::FirstOperand, result),
                    Edge::new(EdgeType::SecondOperand, right),
                ],
            ));
            result = id;
        }
        Ok(result)
    }

    /// Построить бинарную операцию.
    fn build_binop(
        &mut self,
        elements: &[SExpr],
        node_type: NodeType,
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() != 3 {
            return Err(ParseError::wrong_arity(
                span,
                format!("{:?}", node_type),
                "2",
                elements.len() - 1,
            ));
        }

        let left_id = self.build_expr(&elements[1])?;
        let right_id = self.build_expr(&elements[2])?;

        let id = self.alloc_id();
        let node = Node::with_edges_and_span(
            id,
            node_type,
            None,
            vec![
                Edge::new(EdgeType::FirstOperand, left_id),
                Edge::new(EdgeType::SecondOperand, right_id),
            ],
            span,
        );
        self.asg.add_node(node);
        Ok(id)
    }

    /// Построить тернарную операцию (3 аргумента).
    fn build_ternary(
        &mut self,
        elements: &[SExpr],
        node_type: NodeType,
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() != 4 {
            return Err(ParseError::wrong_arity(
                span,
                format!("{:?}", node_type),
                "3",
                elements.len() - 1,
            ));
        }

        let first_id = self.build_expr(&elements[1])?;
        let second_id = self.build_expr(&elements[2])?;
        let third_id = self.build_expr(&elements[3])?;

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges_and_span(
            id,
            node_type,
            None,
            vec![
                Edge::new(EdgeType::FirstOperand, first_id),
                Edge::new(EdgeType::SecondOperand, second_id),
                Edge::new(EdgeType::ApplicationArgument, third_id),
            ],
            span,
        ));
        Ok(id)
    }

    /// Построить унарную операцию.
    fn build_unop(
        &mut self,
        elements: &[SExpr],
        node_type: NodeType,
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() != 2 {
            return Err(ParseError::wrong_arity(
                span,
                format!("{:?}", node_type),
                "1",
                elements.len() - 1,
            ));
        }

        let operand_id = self.build_expr(&elements[1])?;

        let id = self.alloc_id();
        let node = Node::with_edges_and_span(
            id,
            node_type,
            None,
            vec![Edge::new(EdgeType::ApplicationArgument, operand_id)],
            span,
        );
        self.asg.add_node(node);
        Ok(id)
    }

    /// Построить бинарную или унарную операцию (для -).
    fn build_binop_or_unary(
        &mut self,
        elements: &[SExpr],
        binop_type: NodeType,
        unop_type: NodeType,
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        match elements.len() {
            2 => self.build_unop(elements, unop_type, span),
            3 => self.build_binop(elements, binop_type, span),
            _ => Err(ParseError::wrong_arity(
                span,
                "-",
                "1 or 2",
                elements.len() - 1,
            )),
        }
    }

    /// Построить let.
    fn build_let(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (let name value) или (let name Type value) или
        // (let [a b c] array-expr) - destructuring
        if elements.len() < 3 || elements.len() > 4 {
            return Err(ParseError::wrong_arity(
                span,
                "let",
                "2 or 3",
                elements.len() - 1,
            ));
        }

        // Проверяем, является ли второй элемент списком (destructuring)
        if let SExpr::List(pattern_list) = &elements[1] {
            return self.build_let_destructure(&pattern_list.value, &elements[2], span);
        }

        let name = elements[1]
            .as_ident()
            .ok_or_else(|| ParseError::InvalidLiteral {
                span: elements[1].span(),
                message: "Expected identifier for variable name".to_string(),
            })?;

        let value_expr = if elements.len() == 3 {
            &elements[2]
        } else {
            &elements[3]
        };

        let value_id = self.build_expr(value_expr)?;

        let id = self.alloc_id();
        let node = Node::with_edges(
            id,
            NodeType::Variable,
            Some(name.as_bytes().to_vec()),
            vec![Edge::new(EdgeType::VarValue, value_id)],
        );
        self.asg.add_node(node);
        Ok(id)
    }

    /// Построить let с destructuring: (let [a b c] expr) или (let (a b c) expr)
    fn build_let_destructure(
        &mut self,
        pattern: &[SExpr],
        value_expr: &SExpr,
        _span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // Собираем имена переменных из паттерна
        let mut names = Vec::new();
        for elem in pattern {
            let name = elem.as_ident().ok_or_else(|| ParseError::InvalidLiteral {
                span: elem.span(),
                message: "Expected identifier in destructuring pattern".to_string(),
            })?;
            names.push(name);
        }

        // Вычисляем выражение-источник
        let value_id = self.build_expr(value_expr)?;

        // Кодируем имена: количество + имена через нуль-байт
        let mut payload = Vec::new();
        payload.extend_from_slice(&(names.len() as u32).to_le_bytes());
        for name in &names {
            payload.extend_from_slice(name.as_bytes());
            payload.push(0); // разделитель
        }

        let id = self.alloc_id();
        let node = Node::with_edges(
            id,
            NodeType::LetDestructure,
            Some(payload),
            vec![Edge::new(EdgeType::VarValue, value_id)],
        );
        self.asg.add_node(node);
        Ok(id)
    }

    /// Построить set (присваивание).
    fn build_set(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() != 3 {
            return Err(ParseError::wrong_arity(
                span,
                "set",
                "2",
                elements.len() - 1,
            ));
        }

        let name = elements[1]
            .as_ident()
            .ok_or_else(|| ParseError::InvalidLiteral {
                span: elements[1].span(),
                message: "Expected identifier for variable name".to_string(),
            })?;

        // Создаем VarRef для цели
        let target_id = self.alloc_id();
        self.asg.add_node(Node::new(
            target_id,
            NodeType::VarRef,
            Some(name.as_bytes().to_vec()),
        ));

        let value_id = self.build_expr(&elements[2])?;

        let id = self.alloc_id();
        let node = Node::with_edges(
            id,
            NodeType::Assign,
            None,
            vec![
                Edge::new(EdgeType::AssignTarget, target_id),
                Edge::new(EdgeType::AssignValue, value_id),
            ],
        );
        self.asg.add_node(node);
        Ok(id)
    }

    /// Построить if.
    fn build_if(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() < 3 || elements.len() > 4 {
            return Err(ParseError::wrong_arity(
                span,
                "if",
                "2 or 3",
                elements.len() - 1,
            ));
        }

        let cond_id = self.build_expr(&elements[1])?;
        let then_id = self.build_expr(&elements[2])?;

        let mut edges = vec![
            Edge::new(EdgeType::Condition, cond_id),
            Edge::new(EdgeType::ThenBranch, then_id),
        ];

        if elements.len() == 4 {
            let else_id = self.build_expr(&elements[3])?;
            edges.push(Edge::new(EdgeType::ElseBranch, else_id));
        }

        let id = self.alloc_id();
        self.asg
            .add_node(Node::with_edges(id, NodeType::If, None, edges));
        Ok(id)
    }

    /// Построить do (sequence of expressions).
    fn build_do(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() < 2 {
            return Err(ParseError::wrong_arity(
                span,
                "do",
                "1+",
                elements.len() - 1,
            ));
        }

        // Строим все выражения последовательно
        let mut body_ids = Vec::new();

        for expr in &elements[1..] {
            let expr_id = self.build_expr(expr)?;
            body_ids.push(expr_id);
        }

        // Если одно выражение - возвращаем его напрямую
        if body_ids.len() == 1 {
            return Ok(body_ids[0]);
        }

        // Создаём узел Block со всеми выражениями
        let id = self.alloc_id();
        let edges: Vec<Edge> = body_ids
            .into_iter()
            .map(|e| Edge::new(EdgeType::BlockStatement, e))
            .collect();

        self.asg
            .add_node(Node::with_edges(id, NodeType::Block, None, edges));
        Ok(id)
    }

    /// Построить loop.
    fn build_loop(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() != 2 {
            return Err(ParseError::wrong_arity(
                span,
                "loop",
                "1",
                elements.len() - 1,
            ));
        }

        let body_id = self.build_expr(&elements[1])?;

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::Loop,
            None,
            vec![Edge::new(EdgeType::LoopBody, body_id)],
        ));
        Ok(id)
    }

    /// Построить while.
    fn build_while(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() != 3 {
            return Err(ParseError::wrong_arity(
                span,
                "while",
                "2",
                elements.len() - 1,
            ));
        }

        let cond_id = self.build_expr(&elements[1])?;
        let body_id = self.build_expr(&elements[2])?;

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::Loop,
            None,
            vec![
                Edge::new(EdgeType::Condition, cond_id),
                Edge::new(EdgeType::LoopBody, body_id),
            ],
        ));
        Ok(id)
    }

    /// Построить break.
    fn build_break(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() > 2 {
            return Err(ParseError::wrong_arity(
                span,
                "break",
                "0 or 1",
                elements.len() - 1,
            ));
        }

        let id = self.alloc_id();

        let edges = if elements.len() == 2 {
            let value_id = self.build_expr(&elements[1])?;
            vec![Edge::new(EdgeType::ReturnValue, value_id)]
        } else {
            vec![]
        };

        self.asg
            .add_node(Node::with_edges(id, NodeType::Break, None, edges));
        Ok(id)
    }

    /// Построить continue.
    fn build_continue(&mut self, _span: super::token::Span) -> Result<NodeID, ParseError> {
        let id = self.alloc_id();
        self.asg.add_node(Node::new(id, NodeType::Continue, None));
        Ok(id)
    }

    /// Построить return.
    fn build_return(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() > 2 {
            return Err(ParseError::wrong_arity(
                span,
                "return",
                "0 or 1",
                elements.len() - 1,
            ));
        }

        let id = self.alloc_id();

        let edges = if elements.len() == 2 {
            let value_id = self.build_expr(&elements[1])?;
            vec![Edge::new(EdgeType::ReturnValue, value_id)]
        } else {
            vec![]
        };

        self.asg
            .add_node(Node::with_edges(id, NodeType::Return, None, edges));
        Ok(id)
    }

    /// Построить fn.
    fn build_fn(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (fn name (params...) body)
        if elements.len() != 4 {
            return Err(ParseError::wrong_arity(span, "fn", "3", elements.len() - 1));
        }

        let name = elements[1]
            .as_ident()
            .ok_or_else(|| ParseError::InvalidLiteral {
                span: elements[1].span(),
                message: "Expected identifier for function name".to_string(),
            })?;

        let params_list = elements[2]
            .as_list()
            .ok_or_else(|| ParseError::InvalidLiteral {
                span: elements[2].span(),
                message: "Expected parameter list".to_string(),
            })?;

        let mut edges = Vec::new();

        // Создаем узлы параметров
        for param_expr in params_list {
            let param_name = param_expr
                .as_ident()
                .ok_or_else(|| ParseError::InvalidLiteral {
                    span: param_expr.span(),
                    message: "Expected identifier for parameter name".to_string(),
                })?;

            let param_id = self.alloc_id();
            self.asg.add_node(Node::new(
                param_id,
                NodeType::Parameter,
                Some(param_name.as_bytes().to_vec()),
            ));
            edges.push(Edge::new(EdgeType::FunctionParameter, param_id));
        }

        // Строим тело функции
        let body_id = self.build_expr(&elements[3])?;
        edges.push(Edge::new(EdgeType::FunctionBody, body_id));

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::Function,
            Some(name.as_bytes().to_vec()),
            edges,
        ));
        Ok(id)
    }

    /// Построить lambda.
    fn build_lambda(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (lambda (params...) body)
        if elements.len() != 3 {
            return Err(ParseError::wrong_arity(
                span,
                "lambda",
                "2",
                elements.len() - 1,
            ));
        }

        let params_list = elements[1]
            .as_list()
            .ok_or_else(|| ParseError::InvalidLiteral {
                span: elements[1].span(),
                message: "Expected parameter list".to_string(),
            })?;

        let mut edges = Vec::new();

        for param_expr in params_list {
            let param_name = param_expr
                .as_ident()
                .ok_or_else(|| ParseError::InvalidLiteral {
                    span: param_expr.span(),
                    message: "Expected identifier for parameter name".to_string(),
                })?;

            let param_id = self.alloc_id();
            self.asg.add_node(Node::new(
                param_id,
                NodeType::Parameter,
                Some(param_name.as_bytes().to_vec()),
            ));
            edges.push(Edge::new(EdgeType::FunctionParameter, param_id));
        }

        let body_id = self.build_expr(&elements[2])?;
        edges.push(Edge::new(EdgeType::FunctionBody, body_id));

        let id = self.alloc_id();
        self.asg
            .add_node(Node::with_edges(id, NodeType::Lambda, None, edges));
        Ok(id)
    }

    /// Построить вызов функции.
    fn build_call(
        &mut self,
        elements: &[SExpr],
        _span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (func arg1 arg2 ...)
        if elements.is_empty() {
            let id = self.alloc_id();
            self.asg
                .add_node(Node::new(id, NodeType::LiteralUnit, None));
            return Ok(id);
        }

        // Создаем VarRef для имени функции
        let func_name = elements[0]
            .as_ident()
            .ok_or_else(|| ParseError::InvalidLiteral {
                span: elements[0].span(),
                message: "Expected function name".to_string(),
            })?;

        let target_id = self.alloc_id();
        self.asg.add_node(Node::new(
            target_id,
            NodeType::VarRef,
            Some(func_name.as_bytes().to_vec()),
        ));

        let mut edges = vec![Edge::new(EdgeType::CallTarget, target_id)];

        // Строим аргументы
        for arg_expr in &elements[1..] {
            let arg_id = self.build_expr(arg_expr)?;
            edges.push(Edge::new(EdgeType::CallArgument, arg_id));
        }

        let id = self.alloc_id();
        self.asg
            .add_node(Node::with_edges(id, NodeType::Call, None, edges));
        Ok(id)
    }

    /// Построить array.
    fn build_array(
        &mut self,
        elements: &[SExpr],
        _span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (array elem1 elem2 ...)
        let mut edges = Vec::new();

        for elem_expr in &elements[1..] {
            let elem_id = self.build_expr(elem_expr)?;
            edges.push(Edge::new(EdgeType::ArrayElement, elem_id));
        }

        let id = self.alloc_id();
        self.asg
            .add_node(Node::with_edges(id, NodeType::Array, None, edges));
        Ok(id)
    }

    /// Построить index (доступ к элементу массива).
    fn build_index(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (index array idx)
        if elements.len() != 3 {
            return Err(ParseError::wrong_arity(
                span,
                "index",
                "2",
                elements.len() - 1,
            ));
        }

        let array_id = self.build_expr(&elements[1])?;
        let index_id = self.build_expr(&elements[2])?;

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::ArrayIndex,
            None,
            vec![
                Edge::new(EdgeType::ApplicationArgument, array_id),
                Edge::new(EdgeType::ArrayIndexExpr, index_id),
            ],
        ));
        Ok(id)
    }

    /// Построить first/second/third shorthand: (first arr) = (index arr 0)
    fn build_nth_shorthand(
        &mut self,
        elements: &[SExpr],
        n: i64,
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() != 2 {
            return Err(ParseError::wrong_arity(
                span,
                "first/second/third",
                "1",
                elements.len() - 1,
            ));
        }

        let array_id = self.build_expr(&elements[1])?;

        // Создаём литерал индекса
        let index_id = self.alloc_id();
        self.asg.add_node(Node::new(
            index_id,
            NodeType::LiteralInt,
            Some(n.to_le_bytes().to_vec()),
        ));

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::ArrayIndex,
            None,
            vec![
                Edge::new(EdgeType::ApplicationArgument, array_id),
                Edge::new(EdgeType::ArrayIndexExpr, index_id),
            ],
        ));
        Ok(id)
    }

    /// Построить last: (last arr) - последний элемент
    fn build_last(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() != 2 {
            return Err(ParseError::wrong_arity(
                span,
                "last",
                "1",
                elements.len() - 1,
            ));
        }

        let array_id = self.build_expr(&elements[1])?;

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::ArrayLast,
            None,
            vec![Edge::new(EdgeType::ApplicationArgument, array_id)],
        ));
        Ok(id)
    }

    /// Построить length (длина массива).
    fn build_length(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (length array)
        if elements.len() != 2 {
            return Err(ParseError::wrong_arity(
                span,
                "length",
                "1",
                elements.len() - 1,
            ));
        }

        let array_id = self.build_expr(&elements[1])?;

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::ArrayLength,
            None,
            vec![Edge::new(EdgeType::ApplicationArgument, array_id)],
        ));
        Ok(id)
    }

    /// Построить set-index (установка элемента массива).
    fn build_set_index(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (set-index array idx value)
        if elements.len() != 4 {
            return Err(ParseError::wrong_arity(
                span,
                "set-index",
                "3",
                elements.len() - 1,
            ));
        }

        let array_id = self.build_expr(&elements[1])?;
        let index_id = self.build_expr(&elements[2])?;
        let value_id = self.build_expr(&elements[3])?;

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::ArraySetIndex,
            None,
            vec![
                Edge::new(EdgeType::ApplicationArgument, array_id),
                Edge::new(EdgeType::ArrayIndexExpr, index_id),
                Edge::new(EdgeType::AssignValue, value_id),
            ],
        ));
        Ok(id)
    }

    /// Построить map: (map array fn)
    fn build_map(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (map array fn)
        if elements.len() != 3 {
            return Err(ParseError::wrong_arity(
                span,
                "map",
                "2",
                elements.len() - 1,
            ));
        }

        let array_id = self.build_expr(&elements[1])?;
        let fn_id = self.build_expr(&elements[2])?;

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::ArrayMap,
            None,
            vec![
                Edge::new(EdgeType::SourceArray, array_id),
                Edge::new(EdgeType::MapFunction, fn_id),
            ],
        ));
        Ok(id)
    }

    /// Построить filter: (filter array predicate)
    fn build_filter(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (filter array predicate)
        if elements.len() != 3 {
            return Err(ParseError::wrong_arity(
                span,
                "filter",
                "2",
                elements.len() - 1,
            ));
        }

        let array_id = self.build_expr(&elements[1])?;
        let pred_id = self.build_expr(&elements[2])?;

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::ArrayFilter,
            None,
            vec![
                Edge::new(EdgeType::SourceArray, array_id),
                Edge::new(EdgeType::FilterPredicate, pred_id),
            ],
        ));
        Ok(id)
    }

    /// Построить reduce: (reduce array init fn)
    fn build_reduce(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (reduce array init fn)
        if elements.len() != 4 {
            return Err(ParseError::wrong_arity(
                span,
                "reduce",
                "3",
                elements.len() - 1,
            ));
        }

        let array_id = self.build_expr(&elements[1])?;
        let init_id = self.build_expr(&elements[2])?;
        let fn_id = self.build_expr(&elements[3])?;

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::ArrayReduce,
            None,
            vec![
                Edge::new(EdgeType::SourceArray, array_id),
                Edge::new(EdgeType::ReduceInit, init_id),
                Edge::new(EdgeType::ReduceFunction, fn_id),
            ],
        ));
        Ok(id)
    }

    /// Построить унарную операцию (один аргумент)
    fn build_unary(
        &mut self,
        elements: &[SExpr],
        node_type: NodeType,
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() != 2 {
            return Err(ParseError::wrong_arity(
                span,
                "unary op",
                "1",
                elements.len() - 1,
            ));
        }

        let arg_id = self.build_expr(&elements[1])?;

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges_and_span(
            id,
            node_type,
            None,
            vec![Edge::new(EdgeType::ApplicationArgument, arg_id)],
            span,
        ));
        Ok(id)
    }

    /// Построить substring: (substring s start end)
    fn build_substring(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() != 4 {
            return Err(ParseError::wrong_arity(
                span,
                "substring",
                "3",
                elements.len() - 1,
            ));
        }

        let str_id = self.build_expr(&elements[1])?;
        let start_id = self.build_expr(&elements[2])?;
        let end_id = self.build_expr(&elements[3])?;

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::StringSubstring,
            None,
            vec![
                Edge::new(EdgeType::ApplicationArgument, str_id),
                Edge::new(EdgeType::FirstOperand, start_id),
                Edge::new(EdgeType::SecondOperand, end_id),
            ],
        ));
        Ok(id)
    }

    /// Построить str-replace: (str-replace s from to)
    fn build_str_replace(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() != 4 {
            return Err(ParseError::wrong_arity(
                span,
                "str-replace",
                "3",
                elements.len() - 1,
            ));
        }

        let str_id = self.build_expr(&elements[1])?;
        let from_id = self.build_expr(&elements[2])?;
        let to_id = self.build_expr(&elements[3])?;

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::StringReplace,
            None,
            vec![
                Edge::new(EdgeType::ApplicationArgument, str_id),
                Edge::new(EdgeType::FirstOperand, from_id),
                Edge::new(EdgeType::SecondOperand, to_id),
            ],
        ));
        Ok(id)
    }

    /// Построить константу (PI, E)
    fn build_constant(&mut self, node_type: NodeType) -> Result<NodeID, ParseError> {
        let id = self.alloc_id();
        self.asg.add_node(Node::new(id, node_type, None));
        Ok(id)
    }

    /// Построить try/catch: (try expr (catch e handler))
    fn build_try_catch(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (try expr (catch e handler))
        if elements.len() != 3 {
            return Err(ParseError::wrong_arity(
                span,
                "try",
                "2 (expr and catch)",
                elements.len() - 1,
            ));
        }

        let try_expr = self.build_expr(&elements[1])?;

        // Parse (catch e handler)
        let catch_list = elements[2]
            .as_list()
            .ok_or_else(|| ParseError::InvalidLiteral {
                span: elements[2].span(),
                message: "Expected (catch e handler)".to_string(),
            })?;

        if catch_list.len() != 3 {
            return Err(ParseError::InvalidLiteral {
                span: elements[2].span(),
                message: "Expected (catch error-var handler)".to_string(),
            });
        }

        let catch_keyword = catch_list[0].as_ident().unwrap_or_default();
        if catch_keyword != "catch" {
            return Err(ParseError::InvalidLiteral {
                span: catch_list[0].span(),
                message: "Expected 'catch' keyword".to_string(),
            });
        }

        let error_var = catch_list[1]
            .as_ident()
            .ok_or_else(|| ParseError::InvalidLiteral {
                span: catch_list[1].span(),
                message: "Expected error variable name".to_string(),
            })?;

        let handler_expr = self.build_expr(&catch_list[2])?;

        // Create variable node for error name
        let var_id = self.alloc_id();
        self.asg.add_node(Node::new(
            var_id,
            NodeType::Variable,
            Some(error_var.as_bytes().to_vec()),
        ));

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::TryCatch,
            None,
            vec![
                Edge::new(EdgeType::TryBody, try_expr),
                Edge::new(EdgeType::CatchVariable, var_id),
                Edge::new(EdgeType::CatchHandler, handler_expr),
            ],
        ));
        Ok(id)
    }

    /// Построить range: (range start end) или (range start end step)
    fn build_range(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() < 3 || elements.len() > 4 {
            return Err(ParseError::wrong_arity(
                span,
                "range",
                "2 or 3",
                elements.len() - 1,
            ));
        }

        let start_id = self.build_expr(&elements[1])?;
        let end_id = self.build_expr(&elements[2])?;

        let mut edges = vec![
            Edge::new(EdgeType::FirstOperand, start_id),
            Edge::new(EdgeType::SecondOperand, end_id),
        ];

        if elements.len() == 4 {
            let step_id = self.build_expr(&elements[3])?;
            edges.push(Edge::new(EdgeType::LoopStep, step_id));
        }

        let id = self.alloc_id();
        self.asg
            .add_node(Node::with_edges(id, NodeType::Range, None, edges));
        Ok(id)
    }

    /// Построить lazy-range: `(lazy-range start end [step])`
    fn build_lazy_range(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() < 3 || elements.len() > 4 {
            return Err(ParseError::wrong_arity(
                span,
                "lazy-range",
                "2 or 3",
                elements.len() - 1,
            ));
        }

        let start_id = self.build_expr(&elements[1])?;
        let end_id = self.build_expr(&elements[2])?;

        let mut edges = vec![
            Edge::new(EdgeType::FirstOperand, start_id),
            Edge::new(EdgeType::SecondOperand, end_id),
        ];

        if elements.len() == 4 {
            let step_id = self.build_expr(&elements[3])?;
            edges.push(Edge::new(EdgeType::LoopStep, step_id));
        }

        let id = self.alloc_id();
        self.asg
            .add_node(Node::with_edges(id, NodeType::LazyRange, None, edges));
        Ok(id)
    }

    /// Построить for: (for var iterable body)
    fn build_for(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() != 4 {
            return Err(ParseError::wrong_arity(
                span,
                "for",
                "3",
                elements.len() - 1,
            ));
        }

        let var_name = elements[1]
            .as_ident()
            .ok_or_else(|| ParseError::InvalidLiteral {
                span: elements[1].span(),
                message: "Expected variable name".to_string(),
            })?;

        let iterable_id = self.build_expr(&elements[2])?;
        let body_id = self.build_expr(&elements[3])?;

        // Create variable node for loop variable
        let var_id = self.alloc_id();
        self.asg.add_node(Node::new(
            var_id,
            NodeType::Variable,
            Some(var_name.as_bytes().to_vec()),
        ));

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::For,
            None,
            vec![
                Edge::new(EdgeType::LoopInit, var_id),
                Edge::new(EdgeType::Condition, iterable_id),
                Edge::new(EdgeType::LoopBody, body_id),
            ],
        ));
        Ok(id)
    }

    /// Построить list comprehension: `(list-comp expr var iter [condition])`
    fn build_list_comp(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (list-comp expr var iter) или (list-comp expr var iter condition)
        if elements.len() < 4 || elements.len() > 5 {
            return Err(ParseError::wrong_arity(
                span,
                "list-comp",
                "3 or 4",
                elements.len() - 1,
            ));
        }

        let expr_id = self.build_expr(&elements[1])?;

        let var_name = elements[2]
            .as_ident()
            .ok_or_else(|| ParseError::InvalidLiteral {
                span: elements[2].span(),
                message: "Expected variable name".to_string(),
            })?;

        let iterable_id = self.build_expr(&elements[3])?;

        // Опциональное условие
        let condition_id = if elements.len() == 5 {
            Some(self.build_expr(&elements[4])?)
        } else {
            None
        };

        let mut edges = vec![
            Edge::new(EdgeType::MapFunction, expr_id),  // expression
            Edge::new(EdgeType::LoopInit, iterable_id), // iterable
        ];

        if let Some(cond_id) = condition_id {
            edges.push(Edge::new(EdgeType::Condition, cond_id));
        }

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::ListComprehension,
            Some(var_name.as_bytes().to_vec()),
            edges,
        ));
        Ok(id)
    }

    /// Построить dict: (dict k1 v1 k2 v2 ...)
    fn build_dict(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (dict) or (dict k1 v1 k2 v2 ...)
        if (elements.len() - 1) % 2 != 0 {
            return Err(ParseError::InvalidLiteral {
                span,
                message: "Dict requires even number of arguments (key-value pairs)".to_string(),
            });
        }

        let mut edges = Vec::new();
        let mut i = 1;
        while i + 1 < elements.len() {
            let key_id = self.build_expr(&elements[i])?;
            let val_id = self.build_expr(&elements[i + 1])?;
            edges.push(Edge::new(EdgeType::FirstOperand, key_id));
            edges.push(Edge::new(EdgeType::SecondOperand, val_id));
            i += 2;
        }

        let id = self.alloc_id();
        self.asg
            .add_node(Node::with_edges(id, NodeType::Dict, None, edges));
        Ok(id)
    }

    /// Построить pipe: (|> value fn1 fn2 ...)
    fn build_pipe(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() < 3 {
            return Err(ParseError::wrong_arity(
                span,
                "pipe",
                "at least 2",
                elements.len() - 1,
            ));
        }

        let mut edges = Vec::new();
        for elem in &elements[1..] {
            let expr_id = self.build_expr(elem)?;
            edges.push(Edge::new(EdgeType::ApplicationArgument, expr_id));
        }

        let id = self.alloc_id();
        self.asg
            .add_node(Node::with_edges(id, NodeType::Pipe, None, edges));
        Ok(id)
    }

    /// Построить compose: (compose fn1 fn2 ...)
    fn build_compose(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() < 3 {
            return Err(ParseError::wrong_arity(
                span,
                "compose",
                "at least 2",
                elements.len() - 1,
            ));
        }

        let mut edges = Vec::new();
        for elem in &elements[1..] {
            let expr_id = self.build_expr(elem)?;
            edges.push(Edge::new(EdgeType::ApplicationArgument, expr_id));
        }

        let id = self.alloc_id();
        self.asg
            .add_node(Node::with_edges(id, NodeType::Compose, None, edges));
        Ok(id)
    }

    /// Построить print.
    fn build_print(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (print expr)
        if elements.len() != 2 {
            return Err(ParseError::wrong_arity(
                span,
                "print",
                "1",
                elements.len() - 1,
            ));
        }

        let expr_id = self.build_expr(&elements[1])?;

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::Print,
            None,
            vec![Edge::new(EdgeType::ApplicationArgument, expr_id)],
        ));
        Ok(id)
    }

    /// Построить input: (input) или (input prompt)
    fn build_input(
        &mut self,
        elements: &[SExpr],
        node_type: NodeType,
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        let id = self.alloc_id();

        if elements.len() == 1 {
            // (input) без prompt
            self.asg.add_node(Node::new(id, node_type, None));
        } else if elements.len() == 2 {
            // (input "prompt")
            let prompt_id = self.build_expr(&elements[1])?;
            self.asg.add_node(Node::with_edges(
                id,
                node_type,
                None,
                vec![Edge::new(EdgeType::ApplicationArgument, prompt_id)],
            ));
        } else {
            return Err(ParseError::wrong_arity(
                span,
                "input",
                "0 or 1",
                elements.len() - 1,
            ));
        }

        Ok(id)
    }

    /// Построить HTML элемент: (div (@ class "foo") "content" (span "nested"))
    fn build_html_element(
        &mut self,
        tag_name: &str,
        elements: &[SExpr],
        _span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        let id = self.alloc_id();
        let mut edges = Vec::new();

        // Обрабатываем детей (атрибуты и содержимое)
        for child in &elements[1..] {
            let child_id = self.build_expr(child)?;
            edges.push(Edge::new(EdgeType::ApplicationArgument, child_id));
        }

        self.asg.add_node(Node::with_edges(
            id,
            NodeType::HtmlElement,
            Some(tag_name.as_bytes().to_vec()),
            edges,
        ));
        Ok(id)
    }

    /// Построить HTTP response: (http-response status headers body)
    fn build_http_response(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() != 4 {
            return Err(ParseError::wrong_arity(
                span,
                "http-response",
                "3",
                elements.len() - 1,
            ));
        }

        let status_id = self.build_expr(&elements[1])?;
        let headers_id = self.build_expr(&elements[2])?;
        let body_id = self.build_expr(&elements[3])?;

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::HttpResponse,
            None,
            vec![
                Edge::new(EdgeType::ApplicationArgument, status_id),
                Edge::new(EdgeType::ApplicationArgument, headers_id),
                Edge::new(EdgeType::ApplicationArgument, body_id),
            ],
        ));
        Ok(id)
    }

    /// Построить GUI window: (window title width height body)
    fn build_gui_window(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() < 4 {
            return Err(ParseError::wrong_arity(
                span,
                "window",
                "at least 3",
                elements.len() - 1,
            ));
        }

        let title_id = self.build_expr(&elements[1])?;
        let width_id = self.build_expr(&elements[2])?;
        let height_id = self.build_expr(&elements[3])?;

        let mut edges = vec![
            Edge::new(EdgeType::ApplicationArgument, title_id),
            Edge::new(EdgeType::ApplicationArgument, width_id),
            Edge::new(EdgeType::ApplicationArgument, height_id),
        ];

        // Остальные аргументы - содержимое окна
        for child in &elements[4..] {
            let child_id = self.build_expr(child)?;
            edges.push(Edge::new(EdgeType::ApplicationArgument, child_id));
        }

        let id = self.alloc_id();
        self.asg
            .add_node(Node::with_edges(id, NodeType::GuiWindow, None, edges));
        Ok(id)
    }

    /// Построить GUI button: (gui-button text onclick)
    fn build_gui_button(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() != 3 {
            return Err(ParseError::wrong_arity(
                span,
                "gui-button",
                "2",
                elements.len() - 1,
            ));
        }

        let text_id = self.build_expr(&elements[1])?;
        let onclick_id = self.build_expr(&elements[2])?;

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::GuiButton,
            None,
            vec![
                Edge::new(EdgeType::ApplicationArgument, text_id),
                Edge::new(EdgeType::ApplicationArgument, onclick_id),
            ],
        ));
        Ok(id)
    }

    /// Построить text-field: (text-field id value onchange)
    fn build_gui_text_field(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() < 2 {
            return Err(ParseError::wrong_arity(
                span,
                "text-field",
                "at least 1",
                elements.len() - 1,
            ));
        }

        let mut edges = Vec::new();
        for child in &elements[1..] {
            let child_id = self.build_expr(child)?;
            edges.push(Edge::new(EdgeType::ApplicationArgument, child_id));
        }

        let id = self.alloc_id();
        self.asg
            .add_node(Node::with_edges(id, NodeType::GuiTextField, None, edges));
        Ok(id)
    }

    /// Построить GUI container (vbox, hbox): (vbox child1 child2 ...)
    fn build_gui_container(
        &mut self,
        elements: &[SExpr],
        node_type: NodeType,
        _span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        let mut edges = Vec::new();
        for child in &elements[1..] {
            let child_id = self.build_expr(child)?;
            edges.push(Edge::new(EdgeType::ApplicationArgument, child_id));
        }

        let id = self.alloc_id();
        self.asg
            .add_node(Node::with_edges(id, node_type, None, edges));
        Ok(id)
    }

    /// Построить canvas: (canvas width height ondraw)
    fn build_gui_canvas(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        if elements.len() != 4 {
            return Err(ParseError::wrong_arity(
                span,
                "canvas",
                "3",
                elements.len() - 1,
            ));
        }

        let width_id = self.build_expr(&elements[1])?;
        let height_id = self.build_expr(&elements[2])?;
        let ondraw_id = self.build_expr(&elements[3])?;

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::GuiCanvas,
            None,
            vec![
                Edge::new(EdgeType::ApplicationArgument, width_id),
                Edge::new(EdgeType::ApplicationArgument, height_id),
                Edge::new(EdgeType::ApplicationArgument, ondraw_id),
            ],
        ));
        Ok(id)
    }

    /// Построить record.
    fn build_record(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (record Name (field1 val1) (field2 val2) ...)
        if elements.len() < 2 {
            return Err(ParseError::wrong_arity(
                span,
                "record",
                "at least 1",
                elements.len() - 1,
            ));
        }

        let name = elements[1]
            .as_ident()
            .ok_or_else(|| ParseError::InvalidLiteral {
                span: elements[1].span(),
                message: "Expected record name".to_string(),
            })?;

        let mut edges = Vec::new();

        for field_expr in &elements[2..] {
            let field_list = field_expr
                .as_list()
                .ok_or_else(|| ParseError::InvalidLiteral {
                    span: field_expr.span(),
                    message: "Expected (field value) pair".to_string(),
                })?;

            if field_list.len() != 2 {
                return Err(ParseError::InvalidLiteral {
                    span: field_expr.span(),
                    message: "Expected (field value) pair".to_string(),
                });
            }

            let field_name =
                field_list[0]
                    .as_ident()
                    .ok_or_else(|| ParseError::InvalidLiteral {
                        span: field_list[0].span(),
                        message: "Expected field name".to_string(),
                    })?;

            let value_id = self.build_expr(&field_list[1])?;

            // Создаем узел для поля
            let field_id = self.alloc_id();
            self.asg.add_node(Node::with_edges(
                field_id,
                NodeType::RecordField,
                Some(field_name.as_bytes().to_vec()),
                vec![Edge::new(EdgeType::VarValue, value_id)],
            ));

            edges.push(Edge::new(EdgeType::RecordFieldDef, field_id));
        }

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::Record,
            Some(name.as_bytes().to_vec()),
            edges,
        ));
        Ok(id)
    }

    /// Построить field (доступ к полю записи).
    fn build_field(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (field record field-name)
        if elements.len() != 3 {
            return Err(ParseError::wrong_arity(
                span,
                "field",
                "2",
                elements.len() - 1,
            ));
        }

        let record_id = self.build_expr(&elements[1])?;

        let field_name = elements[2]
            .as_ident()
            .ok_or_else(|| ParseError::InvalidLiteral {
                span: elements[2].span(),
                message: "Expected field name".to_string(),
            })?;

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::RecordField,
            Some(field_name.as_bytes().to_vec()),
            vec![Edge::new(EdgeType::RecordFieldAccess, record_id)],
        ));
        Ok(id)
    }

    /// Построить match.
    fn build_match(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (match subject pattern1 body1 pattern2 body2 ...)
        if elements.len() < 4 || (elements.len() - 2) % 2 != 0 {
            return Err(ParseError::wrong_arity(
                span,
                "match",
                "subject + pairs of pattern and body",
                elements.len() - 1,
            ));
        }

        let subject_id = self.build_expr(&elements[1])?;

        let mut edges = vec![Edge::new(EdgeType::MatchSubject, subject_id)];

        // Обрабатываем пары pattern/body
        let mut i = 2;
        while i + 1 < elements.len() {
            let pattern_id = self.build_expr(&elements[i])?;
            let body_id = self.build_expr(&elements[i + 1])?;

            // Создаем узел MatchArm
            let arm_id = self.alloc_id();
            self.asg.add_node(Node::with_edges(
                arm_id,
                NodeType::MatchArm,
                None,
                vec![
                    Edge::new(EdgeType::MatchPattern, pattern_id),
                    Edge::new(EdgeType::MatchBody, body_id),
                ],
            ));

            edges.push(Edge::new(EdgeType::ApplicationArgument, arm_id));
            i += 2;
        }

        let id = self.alloc_id();
        self.asg
            .add_node(Node::with_edges(id, NodeType::Match, None, edges));
        Ok(id)
    }

    /// Построить tensor.
    fn build_tensor(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (tensor value)
        if elements.len() != 2 {
            return Err(ParseError::wrong_arity(
                span,
                "tensor",
                "1",
                elements.len() - 1,
            ));
        }

        let value = elements[1]
            .as_float()
            .or_else(|| elements[1].as_int().map(|i| i as f64))
            .ok_or_else(|| ParseError::InvalidLiteral {
                span: elements[1].span(),
                message: "Expected number for tensor".to_string(),
            })?;

        let id = self.alloc_id();
        self.asg.add_node(Node::new(
            id,
            NodeType::LiteralTensor,
            Some((value as f32).to_le_bytes().to_vec()),
        ));
        Ok(id)
    }

    /// Построить module.
    fn build_module(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (module name body...)
        if elements.len() < 2 {
            return Err(ParseError::wrong_arity(
                span,
                "module",
                "at least 1",
                elements.len() - 1,
            ));
        }

        let name = elements[1]
            .as_ident()
            .ok_or_else(|| ParseError::InvalidLiteral {
                span: elements[1].span(),
                message: "Expected module name".to_string(),
            })?;

        let mut edges = Vec::new();

        for body_expr in &elements[2..] {
            let body_id = self.build_expr(body_expr)?;
            edges.push(Edge::new(EdgeType::ModuleContent, body_id));
        }

        let id = self.alloc_id();
        self.asg.add_node(Node::with_edges(
            id,
            NodeType::Module,
            Some(name.as_bytes().to_vec()),
            edges,
        ));
        Ok(id)
    }

    /// Построить import.
    fn build_import(
        &mut self,
        elements: &[SExpr],
        span: super::token::Span,
    ) -> Result<NodeID, ParseError> {
        // (import "path/to/file.asg") or (import module-name)
        if elements.len() < 2 || elements.len() > 4 {
            return Err(ParseError::wrong_arity(
                span,
                "import",
                "1-3",
                elements.len() - 1,
            ));
        }

        // Путь к модулю (строка или идентификатор)
        let path = if let Some(s) = elements[1].as_string() {
            s
        } else if let Some(s) = elements[1].as_ident() {
            s
        } else {
            return Err(ParseError::InvalidLiteral {
                span: elements[1].span(),
                message: "Expected module path (string or identifier)".to_string(),
            });
        };

        // Опционально: (import "path" as alias)
        let alias = if elements.len() == 4 {
            let as_keyword = elements[2].as_ident().unwrap_or_default();
            if as_keyword != "as" {
                return Err(ParseError::InvalidLiteral {
                    span: elements[2].span(),
                    message: "Expected 'as' keyword".to_string(),
                });
            }
            Some(
                elements[3]
                    .as_ident()
                    .ok_or_else(|| ParseError::InvalidLiteral {
                        span: elements[3].span(),
                        message: "Expected alias name".to_string(),
                    })?,
            )
        } else {
            None
        };

        // Сохраняем путь и опционально alias в payload
        let payload = if let Some(a) = alias {
            format!("{}|{}", path, a)
        } else {
            path.to_string()
        };

        let id = self.alloc_id();
        self.asg.add_node(Node::new(
            id,
            NodeType::Import,
            Some(payload.as_bytes().to_vec()),
        ));
        Ok(id)
    }
}

impl Default for AsgBuilder {
    fn default() -> Self {
        Self::new()
    }
}
