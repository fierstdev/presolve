use std::collections::BTreeMap;
use std::path::Path;

use crate::component_graph::UnaryOperator;
use crate::{
    ComponentNode, ComputedExpression, ComputedExpressionKind, ConstantEvaluationError,
    ConstantExpression, ConstantExpressionKind, EffectExpression, EffectExpressionKind,
    EffectStatementSyntaxKind, SemanticId, SerializableValue, SourceProvenance,
};

/// Canonical compiler-owned graph for all lowered state initializer and computed expressions.
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
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionNodeKind {
    Literal(SerializableValue),
    Boolean(bool),
    Identifier(String),
    ThisMember {
        name: String,
    },
    MemberAccess {
        object: SemanticId,
        property: String,
    },
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

impl ExpressionNode {
    #[must_use]
    pub fn dependencies(&self) -> Vec<&SemanticId> {
        match &self.kind {
            ExpressionNodeKind::Literal(_)
            | ExpressionNodeKind::Boolean(_)
            | ExpressionNodeKind::Identifier(_)
            | ExpressionNodeKind::ThisMember { .. } => Vec::new(),
            ExpressionNodeKind::MemberAccess { object, .. } => vec![object],
            ExpressionNodeKind::Arithmetic { left, right, .. }
            | ExpressionNodeKind::Comparison { left, right, .. }
            | ExpressionNodeKind::Logical { left, right, .. }
            | ExpressionNodeKind::NullishCoalescing { left, right } => vec![left, right],
            ExpressionNodeKind::Unary { operand, .. } => vec![operand],
        }
    }
}

impl ExpressionGraph {
    /// # Panics
    ///
    /// Panics when an expression owner has no canonical source provenance.
    #[must_use]
    pub fn from_components(
        components: &[ComponentNode],
        provenance: &BTreeMap<SemanticId, SourceProvenance>,
    ) -> Self {
        let mut graph = Self::default();
        for component in components {
            for field in &component.state_fields {
                let Some(expression) = &field.initial_expression else {
                    continue;
                };
                let field_provenance = provenance
                    .get(&field.id)
                    .expect("state fields with expressions should have source provenance");
                let root = graph.insert_expression(&field.id, "root", expression, field_provenance);
                graph.roots.insert(field.id.clone(), root);
            }
            for method in component
                .methods
                .iter()
                .filter(|method| method.is_computed())
            {
                let Some(expression) = &method.computed_expression else {
                    continue;
                };
                let method_provenance = provenance
                    .get(&method.id)
                    .expect("computed methods with expressions should have source provenance");
                let computed = component.id.computed(&method.name);
                let root = graph.insert_computed_expression(
                    &computed,
                    "root",
                    expression,
                    method_provenance,
                );
                graph.roots.insert(computed, root);
            }
            for method in component.methods.iter().filter(|method| method.is_effect()) {
                let Some(body) = &method.effect_body else {
                    continue;
                };
                let effect = component.id.effect(&method.name);
                let provenance = provenance
                    .get(&method.id)
                    .expect("effect methods should have canonical provenance");
                for (index, statement) in body.statements.iter().enumerate() {
                    let path = format!("statement:{index}");
                    match &statement.kind {
                        EffectStatementSyntaxKind::StaticMemberAssignment { target, value } => {
                            graph.insert_effect_expression(
                                &effect,
                                &format!("{path}/target"),
                                target,
                                provenance,
                            );
                            graph.insert_effect_expression(
                                &effect,
                                &format!("{path}/value"),
                                value,
                                provenance,
                            );
                        }
                        EffectStatementSyntaxKind::CapabilityCall { callee, arguments } => {
                            graph.insert_effect_expression(
                                &effect,
                                &format!("{path}/callee"),
                                callee,
                                provenance,
                            );
                            for (argument_index, argument) in arguments.iter().enumerate() {
                                graph.insert_effect_expression(
                                    &effect,
                                    &format!("{path}/argument:{argument_index}"),
                                    argument,
                                    provenance,
                                );
                            }
                        }
                        EffectStatementSyntaxKind::EffectReturn { value: Some(value) } => {
                            graph.insert_effect_expression(
                                &effect,
                                &format!("{path}/return"),
                                value,
                                provenance,
                            );
                        }
                        EffectStatementSyntaxKind::EffectReturn { value: None }
                        | EffectStatementSyntaxKind::Empty
                        | EffectStatementSyntaxKind::Unsupported(_) => {}
                    }
                }
            }
        }
        graph
    }

    #[must_use]
    pub fn root_for(&self, owner: &SemanticId) -> Option<&SemanticId> {
        self.roots.get(owner)
    }

    #[must_use]
    pub fn node(&self, id: &SemanticId) -> Option<&ExpressionNode> {
        self.nodes.get(id)
    }

    #[must_use]
    pub fn nodes_for(&self, owner: &SemanticId) -> Vec<&ExpressionNode> {
        self.nodes
            .values()
            .filter(|node| node.owner == *owner)
            .collect()
    }

    #[must_use]
    pub fn dependencies_of(&self, id: &SemanticId) -> Vec<&SemanticId> {
        self.node(id)
            .map_or_else(Vec::new, ExpressionNode::dependencies)
    }

    #[must_use]
    pub fn dependents_of(&self, id: &SemanticId) -> Vec<&ExpressionNode> {
        self.nodes
            .values()
            .filter(|node| node.dependencies().contains(&id))
            .collect()
    }

    #[must_use]
    pub fn owner_of(&self, id: &SemanticId) -> Option<&SemanticId> {
        self.node(id).map(|node| &node.owner)
    }

    #[must_use]
    pub fn provenance_of(&self, id: &SemanticId) -> Option<&SourceProvenance> {
        self.node(id).map(|node| &node.provenance)
    }

    #[must_use]
    pub fn nodes_in_file(&self, path: &Path) -> Vec<&ExpressionNode> {
        self.nodes
            .values()
            .filter(|node| node.provenance.path == path)
            .collect()
    }

    #[must_use]
    pub fn nodes_at(&self, path: &Path, offset: usize) -> Vec<&ExpressionNode> {
        self.nodes
            .values()
            .filter(|node| {
                node.provenance.path == path
                    && node.provenance.span.start <= offset
                    && offset < node.provenance.span.end
            })
            .collect()
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
        owner_provenance: &SourceProvenance,
    ) -> SemanticId {
        let id = owner.expression(path);
        let child = |graph: &mut Self, child_path: &str, child: &ConstantExpression| {
            graph.insert_expression(owner, child_path, child, owner_provenance)
        };
        let kind = match &expression.kind {
            ConstantExpressionKind::Literal(value) => ExpressionNodeKind::Literal(value.clone()),
            ConstantExpressionKind::Boolean(value) => ExpressionNodeKind::Boolean(*value),
            ConstantExpressionKind::Arithmetic(arithmetic) => {
                return self.insert_arithmetic(owner, path, arithmetic, owner_provenance)
            }
            ConstantExpressionKind::Comparison {
                left,
                right,
                operator,
            } => ExpressionNodeKind::Comparison {
                left: self.insert_arithmetic(owner, &format!("{path}.0"), left, owner_provenance),
                right: self.insert_arithmetic(owner, &format!("{path}.1"), right, owner_provenance),
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
                provenance: SourceProvenance::new(&owner_provenance.path, expression.span),
            },
        );
        id
    }

    fn insert_arithmetic(
        &mut self,
        owner: &SemanticId,
        path: &str,
        expression: &crate::ArithmeticExpression,
        owner_provenance: &SourceProvenance,
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
                left: self.insert_arithmetic(owner, &format!("{path}.0"), left, owner_provenance),
                right: self.insert_arithmetic(owner, &format!("{path}.1"), right, owner_provenance),
                operator: *operator,
            },
        };
        self.nodes.insert(
            id.clone(),
            ExpressionNode {
                id: id.clone(),
                owner: owner.clone(),
                kind,
                provenance: SourceProvenance::new(&owner_provenance.path, expression.span),
            },
        );
        id
    }

    fn insert_computed_expression(
        &mut self,
        owner: &SemanticId,
        path: &str,
        expression: &ComputedExpression,
        owner_provenance: &SourceProvenance,
    ) -> SemanticId {
        let id = owner.expression(path);
        let child = |graph: &mut Self, child_path: &str, child: &ComputedExpression| {
            graph.insert_computed_expression(owner, child_path, child, owner_provenance)
        };
        let kind = match &expression.kind {
            ComputedExpressionKind::Literal(value) => ExpressionNodeKind::Literal(value.clone()),
            ComputedExpressionKind::ThisMember(name) => {
                ExpressionNodeKind::ThisMember { name: name.clone() }
            }
            ComputedExpressionKind::MemberAccess { object, property } => {
                ExpressionNodeKind::MemberAccess {
                    object: child(self, &format!("{path}.0"), object),
                    property: property.clone(),
                }
            }
            ComputedExpressionKind::Arithmetic {
                left,
                right,
                operator,
            } => ExpressionNodeKind::Arithmetic {
                left: child(self, &format!("{path}.0"), left),
                right: child(self, &format!("{path}.1"), right),
                operator: *operator,
            },
            ComputedExpressionKind::Comparison {
                left,
                right,
                operator,
            } => ExpressionNodeKind::Comparison {
                left: child(self, &format!("{path}.0"), left),
                right: child(self, &format!("{path}.1"), right),
                operator: *operator,
            },
            ComputedExpressionKind::Logical {
                left,
                right,
                operator,
            } => ExpressionNodeKind::Logical {
                left: child(self, &format!("{path}.0"), left),
                right: child(self, &format!("{path}.1"), right),
                operator: *operator,
            },
            ComputedExpressionKind::NullishCoalescing { left, right } => {
                ExpressionNodeKind::NullishCoalescing {
                    left: child(self, &format!("{path}.0"), left),
                    right: child(self, &format!("{path}.1"), right),
                }
            }
            ComputedExpressionKind::Unary { operand, operator } => ExpressionNodeKind::Unary {
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
                provenance: SourceProvenance::new(&owner_provenance.path, expression.span),
            },
        );
        id
    }

    fn insert_effect_expression(
        &mut self,
        owner: &SemanticId,
        path: &str,
        expression: &EffectExpression,
        owner_provenance: &SourceProvenance,
    ) -> SemanticId {
        let id = owner.expression(path);
        let child = |graph: &mut Self, child_path: &str, child: &EffectExpression| {
            graph.insert_effect_expression(owner, child_path, child, owner_provenance)
        };
        let kind = match &expression.kind {
            EffectExpressionKind::Literal(value) => ExpressionNodeKind::Literal(value.clone()),
            EffectExpressionKind::Identifier(name) => ExpressionNodeKind::Identifier(name.clone()),
            EffectExpressionKind::ThisMember(name) => {
                ExpressionNodeKind::ThisMember { name: name.clone() }
            }
            EffectExpressionKind::MemberAccess { object, property } => {
                ExpressionNodeKind::MemberAccess {
                    object: child(self, &format!("{path}.0"), object),
                    property: property.clone(),
                }
            }
            EffectExpressionKind::Arithmetic {
                left,
                right,
                operator,
            } => ExpressionNodeKind::Arithmetic {
                left: child(self, &format!("{path}.0"), left),
                right: child(self, &format!("{path}.1"), right),
                operator: *operator,
            },
            EffectExpressionKind::Comparison {
                left,
                right,
                operator,
            } => ExpressionNodeKind::Comparison {
                left: child(self, &format!("{path}.0"), left),
                right: child(self, &format!("{path}.1"), right),
                operator: *operator,
            },
            EffectExpressionKind::Logical {
                left,
                right,
                operator,
            } => ExpressionNodeKind::Logical {
                left: child(self, &format!("{path}.0"), left),
                right: child(self, &format!("{path}.1"), right),
                operator: *operator,
            },
            EffectExpressionKind::NullishCoalescing { left, right } => {
                ExpressionNodeKind::NullishCoalescing {
                    left: child(self, &format!("{path}.0"), left),
                    right: child(self, &format!("{path}.1"), right),
                }
            }
            EffectExpressionKind::Unary { operand, operator } => ExpressionNodeKind::Unary {
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
                provenance: SourceProvenance::new(&owner_provenance.path, expression.span),
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
            ExpressionNodeKind::Identifier(_)
            | ExpressionNodeKind::ThisMember { .. }
            | ExpressionNodeKind::MemberAccess { .. } => {
                return None;
            }
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
                span: node.provenance.span,
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
            span: node.provenance.span,
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
            span: node.provenance.span,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::component_graph::UnaryOperator;
    use crate::{build_application_semantic_model, ExpressionNodeKind, SerializableValue};

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
        let root = asm
            .expression_graph
            .nodes
            .get(root)
            .expect("expression root node");
        assert_eq!(root.provenance.path, std::path::Path::new("src/Graph.tsx"));
        assert_eq!(root.provenance.span.line, 4);
        assert!(asm.expression_graph.nodes.values().all(|node| {
            node.provenance.path == std::path::Path::new("src/Graph.tsx")
                && root.provenance.span.start <= node.provenance.span.start
                && node.provenance.span.end <= root.provenance.span.end
        }));
    }

    #[test]
    fn lowers_supported_computed_getter_expressions_into_the_canonical_graph() {
        let parsed = ezc_parser::parse_file(
            "src/ComputedExpressions.tsx",
            r#"
@component("x-computed-expressions")
class ComputedExpressions extends Component {
  count = state(1);

  @computed()
  get doubled(): number { return this.count * 2; }

  @computed()
  get adjusted(): number { return +this.doubled - -1; }

  @computed()
  get visible(): boolean {
    return ((this.doubled >= 2 && !this.profile.hidden) ?? false) || true;
  }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let doubled = component.id.computed("doubled");
        let visible = component.id.computed("visible");
        assert!(asm.expression_graph.root_for(&doubled).is_some());
        let root = asm
            .expression_graph
            .root_for(&visible)
            .expect("computed expression root");

        assert_eq!(root.as_str(), "module:src/ComputedExpressions.tsx/component:x-computed-expressions/computed:visible/expression:root");
        assert!(asm
            .expression_graph
            .nodes_for(&visible)
            .iter()
            .all(|node| node.owner == visible));
        assert!(asm.expression_graph.nodes.values().any(|node| {
            matches!(&node.kind, ExpressionNodeKind::ThisMember { name } if name == "count")
        }));
        assert!(asm.expression_graph.nodes.values().any(|node| {
            matches!(&node.kind, ExpressionNodeKind::ThisMember { name } if name == "doubled")
        }));
        assert!(asm.expression_graph.nodes.values().any(|node| {
            matches!(&node.kind, ExpressionNodeKind::ThisMember { name } if name == "profile")
        }));
        assert!(asm.expression_graph.nodes.values().any(|node| {
            matches!(&node.kind, ExpressionNodeKind::MemberAccess { property, .. } if property == "hidden")
        }));
        assert!(asm
            .expression_graph
            .nodes
            .values()
            .any(|node| { matches!(node.kind, ExpressionNodeKind::Arithmetic { .. }) }));
        assert!(asm
            .expression_graph
            .nodes
            .values()
            .any(|node| { matches!(node.kind, ExpressionNodeKind::Comparison { .. }) }));
        assert!(asm
            .expression_graph
            .nodes
            .values()
            .any(|node| { matches!(node.kind, ExpressionNodeKind::Logical { .. }) }));
        assert!(asm
            .expression_graph
            .nodes
            .values()
            .any(|node| { matches!(node.kind, ExpressionNodeKind::NullishCoalescing { .. }) }));
        let unary_operators = asm
            .expression_graph
            .nodes
            .values()
            .filter_map(|node| match node.kind {
                ExpressionNodeKind::Unary { operator, .. } => Some(operator),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(unary_operators.contains(&UnaryOperator::Not));
        assert!(unary_operators.contains(&UnaryOperator::Plus));
        assert!(unary_operators.contains(&UnaryOperator::Minus));
        assert!(asm
            .expression_graph
            .nodes_for(&visible)
            .iter()
            .all(|node| asm.semantic_types.assignments.contains_key(&node.id)));
        assert!(asm.references.iter().any(|reference| {
            reference.kind == crate::SemanticReferenceKind::ComputedState
                && reference.source == doubled
        }));
    }
}
