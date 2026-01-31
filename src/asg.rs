//! Основные структуры Абстрактного Синтаксического Графа (ASG).

use crate::nodecodes::{EdgeType, NodeType};
use crate::parser::token::Span;
use serde::{Deserialize, Serialize};

/// Уникальный идентификатор узла в ASG.
pub type NodeID = u64;

/// Ребро графа, соединяющее узлы.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub edge_type: EdgeType,
    pub target_node_id: NodeID,
    pub payload: Option<Vec<u8>>,
}

impl Edge {
    /// Создать новое ребро.
    pub fn new(edge_type: EdgeType, target_node_id: NodeID) -> Self {
        Self {
            edge_type,
            target_node_id,
            payload: None,
        }
    }

    /// Создать ребро с payload.
    pub fn with_payload(edge_type: EdgeType, target_node_id: NodeID, payload: Vec<u8>) -> Self {
        Self {
            edge_type,
            target_node_id,
            payload: Some(payload),
        }
    }
}

/// Узел ASG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeID,
    pub node_type: NodeType,
    pub payload: Option<Vec<u8>>,
    pub edges: Vec<Edge>,
    /// Позиция в исходном коде (для LSP и сообщений об ошибках).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
}

impl Node {
    /// Создать новый узел (без span для обратной совместимости).
    pub fn new(id: NodeID, node_type: NodeType, payload: Option<Vec<u8>>) -> Self {
        Self {
            id,
            node_type,
            payload,
            edges: Vec::new(),
            span: None,
        }
    }

    /// Создать новый узел с позицией в исходном коде.
    pub fn with_span(id: NodeID, node_type: NodeType, payload: Option<Vec<u8>>, span: Span) -> Self {
        Self {
            id,
            node_type,
            payload,
            edges: Vec::new(),
            span: Some(span),
        }
    }

    /// Создать узел с рёбрами.
    pub fn with_edges(id: NodeID, node_type: NodeType, payload: Option<Vec<u8>>, edges: Vec<Edge>) -> Self {
        Self {
            id,
            node_type,
            payload,
            edges,
            span: None,
        }
    }

    /// Создать узел с рёбрами и span.
    pub fn with_edges_and_span(
        id: NodeID,
        node_type: NodeType,
        payload: Option<Vec<u8>>,
        edges: Vec<Edge>,
        span: Span,
    ) -> Self {
        Self {
            id,
            node_type,
            payload,
            edges,
            span: Some(span),
        }
    }

    /// Установить span для узла.
    pub fn set_span(&mut self, span: Span) {
        self.span = Some(span);
    }

    /// Добавить ребро к узлу.
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    /// Найти ребро по типу.
    pub fn find_edge(&self, edge_type: EdgeType) -> Option<&Edge> {
        self.edges.iter().find(|e| e.edge_type == edge_type)
    }

    /// Найти все рёбра заданного типа.
    pub fn find_edges(&self, edge_type: EdgeType) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.edge_type == edge_type).collect()
    }

    /// Получить имя из payload (для Variable, Function и т.д.).
    pub fn get_name(&self) -> Option<String> {
        self.payload.as_ref().and_then(|p| String::from_utf8(p.clone()).ok())
    }
}

/// Абстрактный Синтаксический Граф.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ASG {
    pub nodes: Vec<Node>,
}

impl ASG {
    /// Создать новый пустой ASG.
    pub fn new() -> Self {
        Self::default()
    }

    /// Добавить узел в граф.
    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    /// Найти узел по ID.
    pub fn find_node(&self, id: NodeID) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Найти узел по ID (mutable).
    pub fn find_node_mut(&mut self, id: NodeID) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    /// Получить количество узлов.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Проверить, пуст ли граф.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Получить следующий свободный ID.
    pub fn next_id(&self) -> NodeID {
        self.nodes.iter().map(|n| n.id).max().unwrap_or(0) + 1
    }
}