use std::collections::BTreeMap;

use crate::component_graph::UnaryOperator;
use crate::{
    ComponentNode, ConstantEvaluationError, ConstantExpression, ConstantExpressionKind, SemanticId,
    SerializableValue,
};
use ezc_parser::SourceSpan;

/// Canonical compiler-owned graph for all lowered state initializer expressions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExpressionGraph {
    pub roots: BTreeMap<SemanticId, SemanticId>,
    pub nodes: BTreeMap<SemanticId, ExpressionNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionNode {
    pub id: SemanticId,
    pub owner: SemanticId,
    pub kind: ExpressionNodeKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionNodeKind {
    Literal(SerializableValue),
    Boolean(bool),
    Arithmetic {
        left: SemanticId,
        right: SemanticId,
        operator: crate::ArithmeticOperator,
    },
    Comparison {
        left: SemanticId,
        right: SemanticId,
        operator: crate::ComparisonOperator,
    },
    Logical {
        left: SemanticId,
        right: SemanticId,
        operator: crate::LogicalOperator,
    },
    NullishCoalescing {
        left: SemanticId,
        right: SemanticId,
    },
    Unary {
        operand: SemanticId,
        operator: UnaryOperator,
    },
}

impl ExpressionGraph {
    #[must_use]
    pub fn from_components(components: &[ComponentNode]) -> Self {
        let mut graph = Self::default();
        for component in components {
            for field in &component.state_fields {
                let Some(expression) = &field.initial_expression else {
                    continue;
                };
                let root = graph.insert_expression(&field.id, "root", expression);
                graph.roots.insert(field.id.clone(), root);
            }
        }
        graph
    }

    #[must_use]
    pub fn root_for(&self, owner: &SemanticId) -> Option<&SemanticId> {
        self.roots.get(owner)
    }

    #[must_use]
    pub fn evaluate(
        &self,
        owner: &SemanticId,
    ) -> Option<Result<SerializableValue, ConstantEvaluationError>> {
        Some(self.expression_for(owner)?.evaluate())
    }

    #[must_use]
    pub fn render(&self, owner: &SemanticId) -> Option<String> {
        self.expression_for(owner)
            .map(|expression| expression.to_string())
    }

    fn insert_expression(
        &mut self,
        owner: &SemanticId,
        path: &str,
        expression: &ConstantExpression,
    ) -> SemanticId {
        let id = owner.expression(path);
        let child = |graph: &mut Self, child_path: &str, child: &ConstantExpression| {
            graph.insert_expression(owner, child_path, child)
        };
        let kind = match &expression.kind {
            ConstantExpressionKind::Literal(value) => ExpressionNodeKind::Literal(value.clone()),
            ConstantExpressionKind::Boolean(value) => ExpressionNodeKind::Boolean(*value),
            ConstantExpressionKind::Arithmetic(arithmetic) => {
                return self.insert_arithmetic(owner, path, arithmetic)
            }
            ConstantExpressionKind::Comparison {
                left,
                right,
                operator,
            } => ExpressionNodeKind::Comparison {
                left: self.insert_arithmetic(owner, &format!("{path}.0"), left),
                right: self.insert_arithmetic(owner, &format!("{path}.1"), right),
                operator: *operator,
            },
            ConstantExpressionKind::Logical {
                left,
                right,
                operator,
            } => ExpressionNodeKind::Logical {
                left: child(self, &format!("{path}.0"), left),
                right: child(self, &format!("{path}.1"), right),
                operator: *operator,
            },
            ConstantExpressionKind::NullishCoalescing { left, right } => {
                ExpressionNodeKind::NullishCoalescing {
                    left: child(self, &format!("{path}.0"), left),
                    right: child(self, &format!("{path}.1"), right),
                }
            }
            ConstantExpressionKind::Unary { operand, operator } => ExpressionNodeKind::Unary {
                operand: child(self, &format!("{path}.0"), operand),
                operator: *operator,
            },
        };
        self.nodes.insert(
            id.clone(),
            ExpressionNode {
                id: id.clone(),
                owner: owner.clone(),
                kind,
                span: expression.span,
            },
        );
        id
    }

    fn insert_arithmetic(
        &mut self,
        owner: &SemanticId,
        path: &str,
        expression: &crate::ArithmeticExpression,
    ) -> SemanticId {
        let id = owner.expression(path);
        let kind = match &expression.kind {
            crate::ArithmeticExpressionKind::Number(value) => {
                ExpressionNodeKind::Literal(SerializableValue::Number(value.clone()))
            }
            crate::ArithmeticExpressionKind::Binary {
                left,
                right,
                operator,
            } => ExpressionNodeKind::Arithmetic {
                left: self.insert_arithmetic(owner, &format!("{path}.0"), left),
                right: self.insert_arithmetic(owner, &format!("{path}.1"), right),
                operator: *operator,
            },
        };
        self.nodes.insert(
            id.clone(),
            ExpressionNode {
                id: id.clone(),
                owner: owner.clone(),
                kind,
                span: expression.span,
            },
        );
        id
    }

    fn expression_for(&self, owner: &SemanticId) -> Option<ConstantExpression> {
        self.expression_from_node(self.root_for(owner)?)
    }

    fn expression_from_node(&self, id: &SemanticId) -> Option<ConstantExpression> {
        let node = self.nodes.get(id)?;
        let kind = match &node.kind {
            ExpressionNodeKind::Literal(value) => ConstantExpressionKind::Literal(value.clone()),
            ExpressionNodeKind::Boolean(value) => ConstantExpressionKind::Boolean(*value),
            ExpressionNodeKind::Arithmetic {
                left,
                right,
                operator,
            } => ConstantExpressionKind::Arithmetic(crate::ArithmeticExpression {
                kind: crate::ArithmeticExpressionKind::Binary {
                    operator: *operator,
                    left: Box::new(self.arithmetic_from_node(left)?),
                    right: Box::new(self.arithmetic_from_node(right)?),
                },
                span: node.span,
            }),
            ExpressionNodeKind::Comparison {
                left,
                right,
                operator,
            } => ConstantExpressionKind::Comparison {
                operator: *operator,
                left: self.arithmetic_from_node(left)?,
                right: self.arithmetic_from_node(right)?,
            },
            ExpressionNodeKind::Logical {
                left,
                right,
                operator,
            } => ConstantExpressionKind::Logical {
                operator: *operator,
                left: Box::new(self.expression_from_node(left)?),
                right: Box::new(self.expression_from_node(right)?),
            },
            ExpressionNodeKind::NullishCoalescing { left, right } => {
                ConstantExpressionKind::NullishCoalescing {
                    left: Box::new(self.expression_from_node(left)?),
                    right: Box::new(self.expression_from_node(right)?),
                }
            }
            ExpressionNodeKind::Unary { operand, operator } => ConstantExpressionKind::Unary {
                operator: *operator,
                operand: Box::new(self.expression_from_node(operand)?),
            },
        };
        Some(ConstantExpression {
            kind,
            span: node.span,
        })
    }

    fn arithmetic_from_node(&self, id: &SemanticId) -> Option<crate::ArithmeticExpression> {
        let node = self.nodes.get(id)?;
        let kind = match &node.kind {
            ExpressionNodeKind::Literal(SerializableValue::Number(value)) => {
                crate::ArithmeticExpressionKind::Number(value.clone())
            }
            ExpressionNodeKind::Arithmetic {
                left,
                right,
                operator,
            } => crate::ArithmeticExpressionKind::Binary {
                operator: *operator,
                left: Box::new(self.arithmetic_from_node(left)?),
                right: Box::new(self.arithmetic_from_node(right)?),
            },
            _ => return None,
        };
        Some(crate::ArithmeticExpression {
            kind,
            span: node.span,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{build_application_semantic_model, SerializableValue};

    #[test]
    fn shares_one_canonical_graph_for_lowered_expression_nodes() {
        let parsed = ezc_parser::parse_file(
            "src/Graph.tsx",
            r#"
@component("x-graph")
class Graph extends Component {
  total = state((1 + 2) * 3);
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let field = &asm.components[0].state_fields[0];
        let root = asm
            .expression_graph
            .root_for(&field.id)
            .expect("expression root");

        assert_eq!(asm.expression_graph.nodes.len(), 5);
        assert_eq!(
            asm.expression_graph.evaluate(&field.id),
            Some(Ok(SerializableValue::Number("9".to_string())))
        );
        assert_eq!(
            asm.expression_graph.render(&field.id).as_deref(),
            Some("((1 + 2) * 3)")
        );
        assert!(asm.expression_graph.nodes.contains_key(root));
    }
}
