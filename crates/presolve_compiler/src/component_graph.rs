use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::instance_context::{ConsumerInstanceId, ProviderInstanceId};
use crate::semantic_id::{
    ComponentInstanceId, ComponentInvocationId, ComponentStructuralRegionId, ConsumerId,
    ContextDeclarationCandidateId, ContextId, EffectId, EffectStatementId, FieldId,
    FormDeclarationCandidateId, FormFieldDeclarationCandidateId, FormId, ProviderId, SemanticId,
    SemanticOwner, SlotBindingId, SlotDeclarationCandidateId, SlotId,
    SubmissionDeclarationCandidateId, ValidationRuleCandidateId,
};
use crate::semantic_package::SemanticPackagePureOperation;
use crate::semantic_provenance::SourceProvenance;
use crate::semantic_reference::{SemanticReference, SemanticReferenceKind};

use presolve_parser::{
    ParsedArithmeticExpression, ParsedArithmeticExpressionKind, ParsedArithmeticOperator,
    ParsedClass, ParsedComparisonOperator, ParsedComputedExpression, ParsedComputedExpressionKind,
    ParsedConstantExpression, ParsedConstantExpressionKind, ParsedDecorator, ParsedEffectBody,
    ParsedEffectExpression, ParsedEffectExpressionKind, ParsedEffectStatementKind,
    ParsedEventHandler, ParsedFile, ParsedJsxAttribute, ParsedJsxAttributeValue, ParsedJsxChild,
    ParsedJsxConditional, ParsedJsxFragment, ParsedJsxList, ParsedJsxNode, ParsedLogicalOperator,
    ParsedMethod, ParsedMethodCall, ParsedSerializableValue, ParsedStateOperation,
    ParsedStaticMemberDesignator, ParsedUnaryOperator, ParsedUnsupportedEffectStatementKind,
    ParsedValidationRuleArgumentKind, ParsedValidationRuleExpressionKind, SourceSpan,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentGraph {
    pub components: Vec<ComponentNode>,
    pub diagnostics: Vec<ComponentDiagnostic>,
    pub references: Vec<SemanticReference>,
    pub provenance: BTreeMap<SemanticId, SourceProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentNode {
    pub id: SemanticId,
    /// The caller-supplied source module that owns this component declaration.
    pub module_path: PathBuf,
    pub owner: SemanticOwner,
    pub class_name: String,
    pub element_name: Option<String>,
    pub route_path: Option<String>,
    /// Normalized authored heritage retained before unsupported inheritance is
    /// excluded from every executable component product.
    pub heritage: Option<AuthoredComponentHeritage>,
    pub state_fields: Vec<StateField>,
    pub context_declarations: Vec<ContextDeclaration>,
    pub provider_declarations: Vec<ProviderDeclaration>,
    pub consumer_declarations: Vec<ConsumerDeclaration>,
    pub slot_declarations: Vec<SlotDeclaration>,
    /// Source-faithful candidates retained for every Context-family decorator,
    /// including forms that cannot produce a semantic entity.
    pub context_declaration_candidates: Vec<AuthoredContextDeclarationCandidate>,
    /// Source-faithful candidates retained for every `@slot()` decorator,
    /// including forms that cannot produce a canonical Slot entity.
    pub slot_declaration_candidates: Vec<AuthoredSlotDeclarationCandidate>,
    /// Normalized source facts retained for every recognized `@form`
    /// declaration, including targets without canonical Form identity inputs.
    pub form_declaration_candidates: Vec<FormDeclarationCandidate>,
    /// Normalized I3 candidates. Canonical Form resolution, type assignment,
    /// duplicate grouping, and `FieldId` construction occur during ASM
    /// assembly over existing immutable authorities.
    pub form_field_declaration_candidates: Vec<FormFieldDeclarationCandidate>,
    /// Parser-normalized source facts for every recognized `@validate`
    /// placement. I6 lowering consumes these facts without revisiting syntax.
    pub validation_rule_declaration_facts: Vec<AuthoredValidationRuleDeclarationFact>,
    /// Parser-normalized source facts for every recognized `@submit` method.
    pub submission_declaration_facts: Vec<AuthoredSubmissionDeclarationFact>,
    /// Parser-normalized source facts for every recognized `@serialize` placement.
    pub serialization_declaration_facts: Vec<AuthoredSerializationDeclarationFact>,
    /// Module imports that shadow compiler-owned validation intrinsic names.
    pub shadowed_validation_intrinsics: BTreeSet<String>,
    pub methods: Vec<ComponentMethod>,
    pub actions: Vec<ComponentAction>,
    pub render: Option<RenderModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredComponentHeritage {
    pub base: String,
    pub provenance: SourceProvenance,
}

/// Source-faithful I6 declaration facts retained before semantic validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredValidationRuleDeclarationFact {
    pub id: ValidationRuleCandidateId,
    pub owner_component: Option<SemanticId>,
    pub declaration_field: Option<SemanticId>,
    pub authored_name: Option<String>,
    pub declaration_kind: AuthoredDeclarationKind,
    pub is_static: bool,
    pub authored_ordinal: usize,
    pub decorator_invoked: bool,
    pub decorator_argument_count: usize,
    pub expression: Option<AuthoredValidationRuleExpression>,
    pub conflicting_decorators: Vec<String>,
    pub decorator_provenance: SourceProvenance,
    pub name_provenance: Option<SourceProvenance>,
    pub provenance: SourceProvenance,
}

/// Source-faithful I9 facts retained before Form/action resolution.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredSubmissionDeclarationFact {
    pub id: SubmissionDeclarationCandidateId,
    pub owner_component: Option<SemanticId>,
    pub method: Option<SemanticId>,
    pub method_name: Option<String>,
    pub is_static: bool,
    pub is_async: bool,
    pub parameter_count: usize,
    pub return_type: Option<String>,
    pub submit_invoked: bool,
    pub submit_argument_count: usize,
    pub form_designator: Option<String>,
    pub has_action: bool,
    pub action_invoked: bool,
    pub action_argument_count: usize,
    pub inherited: bool,
    pub decorator_provenance: SourceProvenance,
    pub form_designator_provenance: Option<SourceProvenance>,
    pub method_provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredSerializationDeclarationFact {
    pub owner_component: Option<SemanticId>,
    pub declaration_field: Option<SemanticId>,
    pub authored_name: Option<String>,
    pub invoked: bool,
    pub argument_count: usize,
    pub format: Option<String>,
    pub provenance: SourceProvenance,
    pub decorator_provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredValidationRuleExpression {
    pub kind: AuthoredValidationRuleExpressionKind,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthoredValidationRuleExpressionKind {
    Call {
        callee: Option<String>,
        arguments: Vec<AuthoredValidationRuleArgument>,
    },
    Identifier(String),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredValidationRuleArgument {
    pub kind: AuthoredValidationRuleArgumentKind,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthoredValidationRuleArgumentKind {
    StringLiteral(String),
    Constant(ConstantExpression),
    ThisMember {
        name: String,
        name_provenance: SourceProvenance,
    },
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateField {
    pub id: SemanticId,
    pub owner: SemanticOwner,
    pub name: String,
    pub initial_value: Option<SerializableValue>,
    pub initial_expression: Option<ConstantExpression>,
    pub declared_type: Option<DeclaredStateType>,
}

/// Authored source facts for one valid G1 `@context()` component field.
///
/// This is deliberately declaration syntax rather than the later canonical
/// Context entity. ASM lowering owns the semantic identity and exposes it to
/// future provider and consumer products.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextDeclaration {
    pub authored_field: SemanticId,
    pub name: String,
    pub declared_type: DeclaredStateType,
    pub default_expression: Option<ConstantExpression>,
    pub decorator_provenance: SourceProvenance,
    pub name_provenance: SourceProvenance,
    pub provenance: SourceProvenance,
}

/// A compiler-retained, compile-time-only Context designator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextDesignator {
    pub component_symbol: String,
    pub context_member: String,
    pub provenance: SourceProvenance,
    pub component_provenance: SourceProvenance,
    pub member_provenance: SourceProvenance,
}

/// Authored source facts for one valid G2 `@provide()` component field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderDeclaration {
    pub authored_field: SemanticId,
    pub name: String,
    pub context_designator: ContextDesignator,
    pub declared_type: DeclaredStateType,
    pub value_expression: ComputedExpression,
    pub decorator_provenance: SourceProvenance,
    pub name_provenance: SourceProvenance,
    pub provenance: SourceProvenance,
}

/// Authored source facts for one valid G3 `@consume()` component field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerDeclaration {
    pub authored_field: SemanticId,
    pub name: String,
    pub context_designator: ContextDesignator,
    pub requested_type: DeclaredStateType,
    pub decorator_provenance: SourceProvenance,
    pub name_provenance: SourceProvenance,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SlotKind {
    Default,
    Named,
}

/// Authored source facts for one valid H1 `@slot()` component field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotDeclaration {
    pub authored_field: SemanticId,
    pub name: String,
    pub kind: SlotKind,
    pub declared_type: DeclaredStateType,
    pub decorator_provenance: SourceProvenance,
    pub name_provenance: SourceProvenance,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotDeclarationViolation {
    InvalidDeclarationKind { actual: AuthoredDeclarationKind },
    StaticDeclarationUnsupported,
    ConflictingSemanticDecorators,
    DecoratorArity { actual: usize, expected: usize },
    InvalidDeclaredType { actual: Option<String> },
    ForbiddenInitializer,
    DefiniteAssignmentRequired,
    DuplicateSlot,
}

/// Parser/lowering-owned facts retained before canonical Slot construction.
/// Invalid candidates have their own source identity and never acquire a
/// `SlotId`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredSlotDeclarationCandidate {
    pub id: SlotDeclarationCandidateId,
    pub owner_component: SemanticId,
    pub authored_declaration: SemanticId,
    pub declaration_kind: AuthoredDeclarationKind,
    pub field_name: Option<String>,
    pub declared_type: Option<DeclaredStateType>,
    pub decorator_argument_count: usize,
    pub decorator_provenance: SourceProvenance,
    pub name_provenance: Option<SourceProvenance>,
    pub provenance: SourceProvenance,
    pub static_modifier_provenance: Option<SourceProvenance>,
    pub initializer_provenance: Option<SourceProvenance>,
    pub violations: Vec<SlotDeclarationViolation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextDeclarationCandidateKind {
    Context,
    Provider,
    Consumer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuthoredDeclarationKind {
    Class,
    InstanceField,
    StaticField,
    Method,
    Getter,
    Setter,
    Parameter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormDeclarationViolation {
    InvalidOwner,
    InvalidTarget { actual: AuthoredDeclarationKind },
    UnsupportedFieldName,
    InvalidDecoratorInvocation,
    InvalidDecoratorArity { actual: usize, expected: usize },
    DuplicateFormDecorator,
    StaticField,
    InitializedField,
    DeclarationOnlyRequired,
    MissingType,
    InvalidType { actual: Option<String> },
    DuplicateName,
    ConflictingSemanticDecorator,
    InheritedDeclaration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormDeclarationStatus {
    Valid,
    Invalid(Vec<FormDeclarationViolation>),
}

/// Immutable I2 declaration candidate retained before canonical Form
/// construction. `form_id` is present only when owner and authored field name
/// are independently canonical, even when another declaration rule fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormDeclarationCandidate {
    pub id: FormDeclarationCandidateId,
    pub owner_component: Option<SemanticId>,
    pub form_id: Option<FormId>,
    pub authored_field: Option<SemanticId>,
    pub authored_name: Option<String>,
    pub declaration_kind: AuthoredDeclarationKind,
    pub decorator_invoked: bool,
    pub decorator_argument_count: usize,
    pub decorator_argument_provenance: Vec<SourceProvenance>,
    pub declaration_only: bool,
    pub declared_type: Option<DeclaredStateType>,
    pub conflicting_decorators: Vec<String>,
    pub decorator_provenance: SourceProvenance,
    pub name_provenance: Option<SourceProvenance>,
    pub initializer_provenance: Option<SourceProvenance>,
    pub provenance: SourceProvenance,
    pub status: FormDeclarationStatus,
}

impl FormDeclarationCandidate {
    #[must_use]
    pub fn violations(&self) -> &[FormDeclarationViolation] {
        match &self.status {
            FormDeclarationStatus::Valid => &[],
            FormDeclarationStatus::Invalid(violations) => violations,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormDesignatorFact {
    pub authored_name: String,
    pub provenance: SourceProvenance,
    pub name_provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedFormDesignatorFact {
    pub object: String,
    pub member: String,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormFieldDeclarationViolation {
    InvalidOwner,
    InvalidTarget { actual: AuthoredDeclarationKind },
    InvalidDecoratorInvocation,
    InvalidDecoratorArity { actual: usize, expected: usize },
    DuplicateFieldDecorator,
    InvalidFormDesignator,
    UnresolvedForm,
    InvalidForm,
    CrossComponentForm,
    StaticField,
    MissingInitializer,
    UnsupportedInitializer,
    InvalidDeclaredType,
    InitializerTypeMismatch,
    NonSerializableType,
    DuplicateName,
    ConflictingSemanticDecorator,
    InheritedDeclaration,
    UnsupportedFieldName,
}

/// Immutable I3 candidate retained for every recognized `@field` placement.
/// `field_id` and `type_assignment` remain absent unless the declaration is
/// valid after canonical Form, type, value, and duplicate resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormFieldDeclarationCandidate {
    pub id: FormFieldDeclarationCandidateId,
    pub owner_component: Option<SemanticId>,
    pub declaration_field: Option<SemanticId>,
    pub authored_name: Option<String>,
    pub field_id: Option<FieldId>,
    pub decorator_invoked: bool,
    pub decorator_argument_count: usize,
    pub decorator_argument_provenance: Vec<SourceProvenance>,
    pub form_designator: Option<FormDesignatorFact>,
    pub unsupported_form_designator: Option<UnsupportedFormDesignatorFact>,
    pub resolved_form: Option<FormId>,
    pub declaration_kind: AuthoredDeclarationKind,
    pub is_static: bool,
    pub declared_type: Option<DeclaredStateType>,
    pub semantic_type: Option<crate::SemanticType>,
    pub type_assignment: Option<crate::SemanticTypeAssignment>,
    pub initializer: Option<SerializableValue>,
    pub conflicting_decorators: Vec<String>,
    pub decorator_provenance: SourceProvenance,
    pub name_provenance: Option<SourceProvenance>,
    pub initializer_provenance: Option<SourceProvenance>,
    pub provenance: SourceProvenance,
    pub violations: Vec<FormFieldDeclarationViolation>,
}

impl FormFieldDeclarationCandidate {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.violations.is_empty()
    }

    pub(crate) fn add_violation(&mut self, violation: FormFieldDeclarationViolation) {
        self.violations.push(violation);
        canonicalize_form_field_violations(&mut self.violations);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextDeclarationViolation {
    InvalidDeclarationKind { actual: AuthoredDeclarationKind },
    StaticDeclarationUnsupported,
    ConflictingSemanticDecorators,
    DecoratorArity { actual: usize, expected: usize },
    ContextDesignatorUnsupported,
    UnresolvedContextDesignator,
    MissingDeclaredType,
    MissingInitializer,
    ForbiddenInitializer,
    UnsupportedInitializer,
    DefiniteAssignmentRequired,
    DuplicateProvider,
}

/// Parser/lowering-owned facts retained before semantic entity construction.
/// No diagnostic code is assigned here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredContextDeclarationCandidate {
    pub id: ContextDeclarationCandidateId,
    pub kind: ContextDeclarationCandidateKind,
    pub owner_component: SemanticId,
    pub authored_declaration: SemanticId,
    pub declaration_kind: AuthoredDeclarationKind,
    pub field_name: Option<String>,
    pub declared_type: Option<DeclaredStateType>,
    pub context_designator: Option<ContextDesignator>,
    pub decorator_argument_count: usize,
    pub decorator_provenance: SourceProvenance,
    pub provenance: SourceProvenance,
    pub static_modifier_provenance: Option<SourceProvenance>,
    pub initializer_provenance: Option<SourceProvenance>,
    pub violations: Vec<ContextDeclarationViolation>,
}

/// A compiler-owned numeric arithmetic expression lowered from a state initializer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArithmeticExpression {
    pub kind: ArithmeticExpressionKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArithmeticExpressionKind {
    Number(String),
    Binary {
        operator: ArithmeticOperator,
        left: Box<ArithmeticExpression>,
        right: Box<ArithmeticExpression>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArithmeticEvaluationError {
    InvalidNumber(String),
    DivisionByZero,
    NonFiniteResult,
}

/// A compiler-owned constant expression lowered from a state initializer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantExpression {
    pub kind: ConstantExpressionKind,
    pub span: SourceSpan,
}

/// A compiler-owned supported expression lowered from an `@computed()` getter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputedExpression {
    pub kind: ComputedExpressionKind,
    pub span: SourceSpan,
}

/// Expression forms accepted by the E2 computed getter lowering slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComputedExpressionKind {
    Literal(SerializableValue),
    ThisMember(String),
    MemberAccess {
        object: Box<ComputedExpression>,
        property: String,
        optional: bool,
    },
    IndexAccess {
        object: Box<ComputedExpression>,
        index: Box<ComputedExpression>,
    },
    Conditional {
        condition: Box<ComputedExpression>,
        when_true: Box<ComputedExpression>,
        when_false: Box<ComputedExpression>,
    },
    Template {
        quasis: Vec<String>,
        expressions: Vec<ComputedExpression>,
    },
    Call {
        callee: String,
        arguments: Vec<ComputedExpression>,
    },
    SemanticPackagePureCall {
        local_name: String,
        package: String,
        version: String,
        integrity: String,
        export: String,
        runtime_module: String,
        resume_policy: String,
        operation: SemanticPackagePureOperation,
        arguments: Vec<ComputedExpression>,
    },
    Arithmetic {
        left: Box<ComputedExpression>,
        right: Box<ComputedExpression>,
        operator: ArithmeticOperator,
    },
    Comparison {
        left: Box<ComputedExpression>,
        right: Box<ComputedExpression>,
        operator: ComparisonOperator,
    },
    Logical {
        left: Box<ComputedExpression>,
        right: Box<ComputedExpression>,
        operator: LogicalOperator,
    },
    NullishCoalescing {
        left: Box<ComputedExpression>,
        right: Box<ComputedExpression>,
    },
    Unary {
        operand: Box<ComputedExpression>,
        operator: UnaryOperator,
    },
}

/// Ordered source syntax retained from one `@effect()` body before semantic resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectBodySyntax {
    pub statements: Vec<EffectStatementSyntax>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectStatementSyntax {
    pub kind: EffectStatementSyntaxKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectStatementSyntaxKind {
    StaticMemberAssignment {
        target: EffectExpression,
        value: EffectExpression,
    },
    CapabilityCall {
        callee: EffectExpression,
        arguments: Vec<EffectExpression>,
    },
    EffectReturn {
        value: Option<EffectExpression>,
    },
    Empty,
    Unsupported(UnsupportedEffectStatementKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsupportedEffectStatementKind {
    LocalDeclaration,
    Branch,
    Loop,
    NestedBlock,
    ExceptionHandling,
    AsyncOperation,
    CompoundAssignment,
    CleanupReturnCandidate,
    UnsupportedExpression,
}

/// An expression operand retained from an effect statement before resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectExpression {
    pub kind: EffectExpressionKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectExpressionKind {
    Literal(SerializableValue),
    Identifier(String),
    ThisMember(String),
    MemberAccess {
        object: Box<EffectExpression>,
        property: String,
    },
    Arithmetic {
        left: Box<EffectExpression>,
        right: Box<EffectExpression>,
        operator: ArithmeticOperator,
    },
    Comparison {
        left: Box<EffectExpression>,
        right: Box<EffectExpression>,
        operator: ComparisonOperator,
    },
    Logical {
        left: Box<EffectExpression>,
        right: Box<EffectExpression>,
        operator: LogicalOperator,
    },
    NullishCoalescing {
        left: Box<EffectExpression>,
        right: Box<EffectExpression>,
    },
    Unary {
        operand: Box<EffectExpression>,
        operator: UnaryOperator,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstantExpressionKind {
    Literal(SerializableValue),
    Boolean(bool),
    Arithmetic(ArithmeticExpression),
    Comparison {
        operator: ComparisonOperator,
        left: ArithmeticExpression,
        right: ArithmeticExpression,
    },
    Logical {
        operator: LogicalOperator,
        left: Box<ConstantExpression>,
        right: Box<ConstantExpression>,
    },
    NullishCoalescing {
        left: Box<ConstantExpression>,
        right: Box<ConstantExpression>,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<ConstantExpression>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOperator {
    And,
    Or,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Not,
    Plus,
    Minus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstantEvaluationError {
    Arithmetic(ArithmeticEvaluationError),
}

impl ArithmeticExpression {
    /// Evaluate a finite constant numeric expression using `Presolve` arithmetic semantics.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid numeric literal, division or remainder by zero,
    /// or a non-finite intermediate or final result.
    pub fn evaluate(&self) -> Result<SerializableValue, ArithmeticEvaluationError> {
        self.evaluate_number()
            .map(|value| SerializableValue::Number(value.to_string()))
    }

    fn evaluate_number(&self) -> Result<f64, ArithmeticEvaluationError> {
        let value = match &self.kind {
            ArithmeticExpressionKind::Number(value) => value
                .parse::<f64>()
                .map_err(|_| ArithmeticEvaluationError::InvalidNumber(value.clone()))?,
            ArithmeticExpressionKind::Binary {
                operator,
                left,
                right,
            } => {
                let left = left.evaluate_number()?;
                let right = right.evaluate_number()?;
                match operator {
                    ArithmeticOperator::Add => left + right,
                    ArithmeticOperator::Subtract => left - right,
                    ArithmeticOperator::Multiply => left * right,
                    ArithmeticOperator::Divide => {
                        if right == 0.0 {
                            return Err(ArithmeticEvaluationError::DivisionByZero);
                        }
                        left / right
                    }
                    ArithmeticOperator::Remainder => {
                        if right == 0.0 {
                            return Err(ArithmeticEvaluationError::DivisionByZero);
                        }
                        left % right
                    }
                }
            }
        };

        value
            .is_finite()
            .then_some(value)
            .ok_or(ArithmeticEvaluationError::NonFiniteResult)
    }
}

impl ConstantExpression {
    /// Evaluate a constant state initializer using `Presolve` expression semantics.
    ///
    /// # Errors
    ///
    /// Returns an error when one of the expression's numeric arithmetic operands is invalid.
    pub fn evaluate(&self) -> Result<SerializableValue, ConstantEvaluationError> {
        match &self.kind {
            ConstantExpressionKind::Literal(value) => Ok(value.clone()),
            ConstantExpressionKind::Boolean(value) => Ok(SerializableValue::Boolean(*value)),
            ConstantExpressionKind::Arithmetic(expression) => expression
                .evaluate()
                .map_err(ConstantEvaluationError::Arithmetic),
            ConstantExpressionKind::Comparison {
                operator,
                left,
                right,
            } => {
                let left = left
                    .evaluate_number()
                    .map_err(ConstantEvaluationError::Arithmetic)?;
                let right = right
                    .evaluate_number()
                    .map_err(ConstantEvaluationError::Arithmetic)?;
                Ok(SerializableValue::Boolean(match operator {
                    ComparisonOperator::Equal => numbers_are_equal(left, right),
                    ComparisonOperator::NotEqual => !numbers_are_equal(left, right),
                    ComparisonOperator::LessThan => left < right,
                    ComparisonOperator::LessThanOrEqual => left <= right,
                    ComparisonOperator::GreaterThan => left > right,
                    ComparisonOperator::GreaterThanOrEqual => left >= right,
                }))
            }
            ConstantExpressionKind::Logical {
                operator,
                left,
                right,
            } => {
                let left = left.evaluate_boolean()?;
                let value = match (operator, left) {
                    (LogicalOperator::And, false) => false,
                    (LogicalOperator::Or, true) => true,
                    (LogicalOperator::And | LogicalOperator::Or, _) => right.evaluate_boolean()?,
                };
                Ok(SerializableValue::Boolean(value))
            }
            ConstantExpressionKind::NullishCoalescing { left, right } => {
                let value = left.evaluate()?;
                if matches!(value, SerializableValue::Null) {
                    right.evaluate()
                } else {
                    Ok(value)
                }
            }
            ConstantExpressionKind::Unary { operator, operand } => match operator {
                UnaryOperator::Not => Ok(SerializableValue::Boolean(!operand.evaluate_boolean()?)),
                UnaryOperator::Plus | UnaryOperator::Minus => {
                    let SerializableValue::Number(value) = operand.evaluate()? else {
                        unreachable!("parser requires numeric unary operands");
                    };
                    let value = value.parse::<f64>().map_err(|_| {
                        ConstantEvaluationError::Arithmetic(
                            ArithmeticEvaluationError::InvalidNumber(value),
                        )
                    })?;
                    let value = if matches!(operator, UnaryOperator::Minus) {
                        -value
                    } else {
                        value
                    };
                    value
                        .is_finite()
                        .then_some(SerializableValue::Number(value.to_string()))
                        .ok_or(ConstantEvaluationError::Arithmetic(
                            ArithmeticEvaluationError::NonFiniteResult,
                        ))
                }
            },
        }
    }

    fn evaluate_boolean(&self) -> Result<bool, ConstantEvaluationError> {
        let SerializableValue::Boolean(value) = self.evaluate()? else {
            unreachable!("parser only permits boolean constants as logical operands");
        };
        Ok(value)
    }
}

fn numbers_are_equal(left: f64, right: f64) -> bool {
    // Presolve constant expressions deliberately use exact finite numeric equality.
    // No tolerance is compiler semantics because the language has no approximate
    // comparison operator or configurable precision model.
    #[allow(clippy::float_cmp)]
    {
        left == right
    }
}

impl fmt::Display for ArithmeticExpression {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ArithmeticExpressionKind::Number(value) => value.fmt(formatter),
            ArithmeticExpressionKind::Binary {
                operator,
                left,
                right,
            } => write!(formatter, "({left} {operator} {right})"),
        }
    }
}

impl fmt::Display for ConstantExpression {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ConstantExpressionKind::Literal(value) => format_constant_literal(value, formatter),
            ConstantExpressionKind::Boolean(value) => value.fmt(formatter),
            ConstantExpressionKind::Arithmetic(expression) => expression.fmt(formatter),
            ConstantExpressionKind::Comparison {
                operator,
                left,
                right,
            } => write!(formatter, "({left} {operator} {right})"),
            ConstantExpressionKind::Logical {
                operator,
                left,
                right,
            } => write!(formatter, "({left} {operator} {right})"),
            ConstantExpressionKind::NullishCoalescing { left, right } => {
                write!(formatter, "({left} ?? {right})")
            }
            ConstantExpressionKind::Unary { operator, operand } => {
                write!(formatter, "({operator}{operand})")
            }
        }
    }
}

fn format_constant_literal(
    value: &SerializableValue,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match value {
        SerializableValue::Null => formatter.write_str("null"),
        SerializableValue::Number(value) => formatter.write_str(value),
        SerializableValue::String(value) => write!(formatter, "{value:?}"),
        SerializableValue::Boolean(value) => write!(formatter, "{value}"),
        SerializableValue::Array(_) | SerializableValue::Object(_) => {
            unreachable!("nullish constant literals are primitive values")
        }
    }
}

impl fmt::Display for ArithmeticOperator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let operator = match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Remainder => "%",
        };
        formatter.write_str(operator)
    }
}

impl fmt::Display for ComparisonOperator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let operator = match self {
            Self::Equal => "===",
            Self::NotEqual => "!==",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
        };
        formatter.write_str(operator)
    }
}

impl fmt::Display for LogicalOperator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::And => "&&",
            Self::Or => "||",
        })
    }
}
impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Not => "!",
            Self::Plus => "+",
            Self::Minus => "-",
        })
    }
}

impl fmt::Display for ArithmeticEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNumber(value) => {
                write!(formatter, "unsupported numeric literal `{value}`")
            }
            Self::DivisionByZero => formatter.write_str("division or remainder by zero"),
            Self::NonFiniteResult => formatter.write_str("non-finite result"),
        }
    }
}

impl fmt::Display for ConstantEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arithmetic(error) => error.fmt(formatter),
        }
    }
}

/// Explicit authored type metadata carried from the parser into canonical compiler data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredStateType {
    pub text: String,
    pub provenance: SourceProvenance,
    pub kind: Option<DeclaredStateTypeKind>,
}

/// The primitive state type forms currently recognized without type inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclaredStateTypeKind {
    String,
    Number,
    Boolean,
    Null,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SerializableValue {
    Null,
    Number(String),
    String(String),
    Boolean(bool),
    Array(Vec<SerializableValue>),
    Object(BTreeMap<String, SerializableValue>),
}

impl SerializableValue {
    #[must_use]
    pub fn render_text(&self) -> String {
        match self {
            Self::Number(value) | Self::String(value) => value.clone(),
            Self::Boolean(value) => value.to_string(),
            Self::Null | Self::Array(_) | Self::Object(_) => String::new(),
        }
    }

    #[must_use]
    pub fn member_path_value(&self, path: &str) -> Option<&Self> {
        if path.is_empty() {
            return None;
        }

        path.split('.').try_fold(self, |value, member| match value {
            Self::Object(values) if !member.is_empty() => values.get(member),
            _ => None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentMethod {
    pub id: SemanticId,
    pub owner: SemanticOwner,
    pub name: String,
    pub is_getter: bool,
    pub is_async: bool,
    pub is_static: bool,
    pub decorators: Vec<String>,
    pub semantic_role: MethodSemanticRole,
    pub local_variables: Vec<MethodLocalVariable>,
    pub parameters: Vec<MethodParameter>,
    pub declared_return_type: Option<DeclaredStateType>,
    pub return_values: Vec<SerializableValue>,
    pub computed_expression: Option<ComputedExpression>,
    pub effect_body: Option<EffectBodySyntax>,
    pub calls: Vec<MethodCall>,
}

/// A compiler-owned call fact retained for computed-purity analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodCall {
    pub callee: String,
    pub span: SourceSpan,
}

/// Additional semantic role carried by a compiler-owned method declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodSemanticRole {
    Standard,
    Computed,
    Effect,
    Action,
}

impl ComponentMethod {
    #[must_use]
    pub const fn is_computed(&self) -> bool {
        matches!(self.semantic_role, MethodSemanticRole::Computed)
    }

    #[must_use]
    pub const fn is_action(&self) -> bool {
        matches!(self.semantic_role, MethodSemanticRole::Action)
    }

    #[must_use]
    pub const fn is_effect(&self) -> bool {
        matches!(self.semantic_role, MethodSemanticRole::Effect)
    }
}

/// A compiler-owned declaration of a supported method parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodParameter {
    pub id: SemanticId,
    pub owner: SemanticOwner,
    pub name: String,
    pub span: SourceSpan,
    pub declared_type: Option<DeclaredStateType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodLocalVariable {
    pub id: SemanticId,
    pub owner: SemanticOwner,
    pub name: String,
    pub value: SerializableValue,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentAction {
    pub id: SemanticId,
    pub owner: SemanticOwner,
    pub method: String,
    pub operation: StateOperation,
    pub field: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateOperation {
    Increment,
    Decrement,
    AddAssign(SerializableValue),
    SubtractAssign(SerializableValue),
    Assign(SerializableValue),
    Toggle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderChild {
    Text {
        value: String,
        span: SourceSpan,
    },
    Binding {
        expression: String,
        span: SourceSpan,
    },
    Element(RenderElement),
    Fragment(RenderFragment),
    Conditional(RenderConditional),
    List(RenderList),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderElement {
    pub tag_name: String,
    pub tag_name_span: SourceSpan,
    pub span: SourceSpan,
    pub attributes: Vec<RenderAttribute>,
    pub event_handlers: Vec<RenderEventHandler>,
    pub children: Vec<RenderChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderFragment {
    pub span: SourceSpan,
    pub children: Vec<RenderChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderConditional {
    pub condition: String,
    pub span: SourceSpan,
    pub when_true: Vec<RenderChild>,
    pub when_false: Vec<RenderChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderList {
    pub iterable: String,
    pub item_variable: String,
    pub index_variable: Option<String>,
    pub key_expression: String,
    pub span: SourceSpan,
    pub item_template: Vec<RenderChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderAttribute {
    pub name: String,
    pub value: RenderAttributeValue,
    pub name_span: SourceSpan,
    pub value_span: Option<SourceSpan>,
    pub expression_span: Option<SourceSpan>,
    pub this_member: Option<presolve_parser::ParsedThisMemberDesignator>,
    pub constant_value: Option<SerializableValue>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderAttributeValue {
    Boolean,
    Static(String),
    Expression(Option<String>),
    Spread(Option<String>),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderEventHandler {
    pub id: SemanticId,
    pub owner: SemanticOwner,
    pub event: String,
    pub handler: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderModel {
    pub root_element: Option<String>,
    pub root_element_name_span: Option<SourceSpan>,
    pub root_span: Option<SourceSpan>,
    pub root_fragment: Option<RenderFragment>,
    pub attributes: Vec<RenderAttribute>,
    pub bindings: Vec<String>,
    pub event_handlers: Vec<RenderEventHandler>,
    pub children: Vec<RenderChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentDiagnostic {
    pub code: String,
    pub severity: ComponentDiagnosticSeverity,
    pub message: String,
    pub provenance: Option<SourceProvenance>,
    pub effect_id: Option<EffectId>,
    pub statement_id: Option<EffectStatementId>,
    pub context_declaration_candidate_id: Option<ContextDeclarationCandidateId>,
    pub context_id: Option<ContextId>,
    pub provider_id: Option<ProviderId>,
    pub consumer_id: Option<ConsumerId>,
    pub slot_id: Option<SlotId>,
    pub invocation_id: Option<ComponentInvocationId>,
    pub component_instance_id: Option<ComponentInstanceId>,
    pub slot_binding_id: Option<SlotBindingId>,
    pub structural_region_id: Option<ComponentStructuralRegionId>,
    pub component_id: Option<SemanticId>,
    pub provider_instance_id: Option<ProviderInstanceId>,
    pub consumer_instance_id: Option<ConsumerInstanceId>,
    pub secondary_labels: Vec<DiagnosticSecondaryLabel>,
}

impl ComponentDiagnostic {
    pub(crate) fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            severity: ComponentDiagnosticSeverity::Error,
            message: message.into(),
            provenance: None,
            effect_id: None,
            statement_id: None,
            context_declaration_candidate_id: None,
            context_id: None,
            provider_id: None,
            consumer_id: None,
            slot_id: None,
            invocation_id: None,
            component_instance_id: None,
            slot_binding_id: None,
            structural_region_id: None,
            component_id: None,
            provider_instance_id: None,
            consumer_instance_id: None,
            secondary_labels: Vec::new(),
        }
    }
}

/// Shared severity classification for compiler diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComponentDiagnosticSeverity {
    Error,
}

impl ComponentDiagnosticSeverity {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
        }
    }
}

/// A deterministic related authored location for a compiler diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticSecondaryLabel {
    pub provenance: SourceProvenance,
    pub message: String,
}

#[must_use]
pub fn build_component_graph(parsed: &ParsedFile) -> ComponentGraph {
    build_component_graph_with_identity(parsed, false)
}

/// Build compiler semantics with a source-module-qualified identity root.
///
/// The canonical application model and compiler frontend use this path. The
/// legacy graph builder remains available for backend compatibility while
/// runtime artifact contracts are migrated independently.
#[must_use]
pub fn build_component_graph_for_module(parsed: &ParsedFile) -> ComponentGraph {
    build_component_graph_with_identity(parsed, true)
}

fn build_component_graph_with_identity(
    parsed: &ParsedFile,
    module_qualified_identity: bool,
) -> ComponentGraph {
    let builtin_types = crate::BuiltinTypeAuthority::for_file(parsed);
    let mut components = Vec::new();
    let mut diagnostics = Vec::new();
    let mut references = Vec::new();
    let mut provenance = BTreeMap::new();

    for class in &parsed.classes {
        let element_name = decorator_argument(class, "component");
        let id = if module_qualified_identity {
            SemanticId::component_in_module(&parsed.path, element_name.as_deref(), &class.name)
        } else {
            SemanticId::component(element_name.as_deref(), &class.name)
        };
        let shadowed_validation_intrinsics = parsed
            .imports
            .iter()
            .flat_map(|import| import.specifiers.iter())
            .map(|specifier| specifier.local.clone())
            .chain(parsed.local_value_bindings.iter().cloned())
            .filter(|name| is_validation_intrinsic_name(name))
            .collect();
        let component = build_component_node(
            class,
            &parsed.path,
            id,
            builtin_types,
            shadowed_validation_intrinsics,
            &mut diagnostics,
        );
        let component_provenance = collect_component_provenance(class, &component, &parsed.path);
        references.extend(collect_semantic_references(
            &component,
            &component_provenance,
        ));
        provenance.extend(component_provenance);
        components.push(component);
    }

    if parsed.classes.is_empty() && parsed.diagnostics.is_empty() {
        diagnostics.push(ComponentDiagnostic {
            severity: ComponentDiagnosticSeverity::Error,
            effect_id: None,
            statement_id: None,
            context_declaration_candidate_id: None,
            context_id: None,
            provider_id: None,
            consumer_id: None,
            slot_id: None,
            invocation_id: None,
            component_instance_id: None,
            slot_binding_id: None,
            structural_region_id: None,
            component_id: None,
            provider_instance_id: None,
            consumer_instance_id: None,
            secondary_labels: Vec::new(),
            provenance: None,
            code: "PSC1000".to_string(),
            message: "no component classes found".to_string(),
        });
    }

    ComponentGraph {
        components,
        diagnostics,
        references,
        provenance,
    }
}

#[allow(clippy::too_many_lines)]
fn build_component_node(
    class: &ParsedClass,
    path: &Path,
    id: SemanticId,
    builtin_types: crate::BuiltinTypeAuthority,
    shadowed_validation_intrinsics: BTreeSet<String>,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) -> ComponentNode {
    let element_name = decorator_argument(class, "component");
    let route_path = decorator_argument(class, "route");

    if element_name.is_none() {
        diagnostics.push(ComponentDiagnostic::error(
            "PSC1001",
            format!("class `{}` is missing @component(...)", class.name),
        ));
    }

    let state_fields = state_fields_from_class(class, path, &id);
    let context_declarations = context_declarations_from_class(class, path, &id);
    let provider_declarations = provider_declarations_from_class(class, path, &id);
    let consumer_declarations = consumer_declarations_from_class(class, path, &id);
    let context_declaration_candidates =
        context_declaration_candidates_from_class(class, path, &id);
    let slot_declaration_candidates = slot_declaration_candidates_from_class(class, path, &id);
    let slot_declarations = slot_declarations_from_candidates(&slot_declaration_candidates);
    let form_declaration_candidates = form_declaration_candidates_from_class(
        class,
        path,
        element_name.is_some(),
        &id,
        builtin_types,
    );
    let form_field_declaration_candidates =
        form_field_declaration_candidates_from_class(class, path, element_name.is_some(), &id);
    let validation_rule_declaration_facts =
        validation_rule_declaration_facts_from_class(class, path, element_name.is_some(), &id);

    let methods = class
        .methods
        .iter()
        .map(|method| component_method_from_parsed(method, path, &id))
        .collect::<Vec<_>>();

    let actions = class
        .methods
        .iter()
        .flat_map(|method| {
            method
                .state_updates
                .iter()
                .enumerate()
                .map(|(index, update)| ComponentAction {
                    id: id.action(&method.name, index),
                    owner: SemanticOwner::entity(id.method(&method.name)),
                    method: method.name.clone(),
                    operation: state_operation_from_parsed(&update.operation),
                    field: update.field.clone(),
                })
        })
        .collect::<Vec<_>>();

    let render = class
        .methods
        .iter()
        .find(|method| method.name == "render")
        .map(|method| render_model_from_parsed_method(method, &id));

    if render.is_none() {
        diagnostics.push(ComponentDiagnostic::error(
            "PSC1002",
            format!("class `{}` is missing render()", class.name),
        ));
    }

    if let Some(render) = &render {
        collect_render_binding_diagnostics(class, render, diagnostics);
        collect_render_event_diagnostics(class, render, diagnostics);
        collect_duplicate_event_diagnostics(render, &class.name, diagnostics);
        collect_render_attribute_diagnostics(render, &state_fields, &class.name, diagnostics);
        collect_render_list_diagnostics(render, &state_fields, &class.name, diagnostics);
    }

    let submission_declaration_facts =
        submission_declaration_facts_from_class(class, path, element_name.is_some(), &id);
    let serialization_declaration_facts =
        serialization_declaration_facts_from_class(class, path, element_name.is_some(), &id);

    ComponentNode {
        id,
        module_path: path.to_path_buf(),
        owner: SemanticOwner::Application,
        class_name: class.name.clone(),
        element_name,
        route_path,
        heritage: class
            .heritage
            .as_ref()
            .map(|heritage| AuthoredComponentHeritage {
                base: heritage.base.clone(),
                provenance: SourceProvenance::new(path, heritage.span),
            }),
        state_fields,
        context_declarations,
        provider_declarations,
        consumer_declarations,
        slot_declarations,
        context_declaration_candidates,
        slot_declaration_candidates,
        form_declaration_candidates,
        form_field_declaration_candidates,
        validation_rule_declaration_facts,
        submission_declaration_facts,
        serialization_declaration_facts,
        shadowed_validation_intrinsics,
        methods,
        actions,
        render,
    }
}

#[allow(clippy::too_many_lines)]
fn form_declaration_candidates_from_class(
    class: &ParsedClass,
    path: &Path,
    is_canonical_component: bool,
    component: &SemanticId,
    builtin_types: crate::BuiltinTypeAuthority,
) -> Vec<FormDeclarationCandidate> {
    let owner = is_canonical_component.then_some(component.clone());
    let mut candidates = Vec::new();

    retain_non_field_form_candidates(
        &mut candidates,
        owner.as_ref(),
        path,
        &class.name,
        AuthoredDeclarationKind::Class,
        &class.decorators,
        class.span,
    );

    for property in &class.properties {
        let form_decorator_count = property
            .decorators
            .iter()
            .filter(|decorator| decorator.name == "form")
            .count();
        for decorator in property
            .decorators
            .iter()
            .filter(|decorator| decorator.name == "form")
        {
            let declaration_kind = if property.is_static {
                AuthoredDeclarationKind::StaticField
            } else {
                AuthoredDeclarationKind::InstanceField
            };
            let identity_capable = owner.is_some() && property.is_identifier_name;
            let form_id = identity_capable.then(|| {
                FormId::for_owner(
                    owner.as_ref().expect("identity-capable Form has an owner"),
                    &property.name,
                )
            });
            let authored_field = identity_capable.then(|| {
                owner
                    .as_ref()
                    .expect("identity-capable Form has an owner")
                    .form_field(&property.name)
            });
            let declared_type =
                property
                    .type_annotation
                    .as_ref()
                    .map(|annotation| DeclaredStateType {
                        kind: declared_state_type_kind(&annotation.text),
                        text: annotation.text.clone(),
                        provenance: SourceProvenance::new(path, annotation.span),
                    });
            let mut conflicting_decorators = property
                .decorators
                .iter()
                .filter(|other| other.name != "form" && is_presolve_semantic_decorator(&other.name))
                .map(|other| other.name.clone())
                .collect::<Vec<_>>();
            if property.initializer.as_deref() == Some("state(...)") {
                conflicting_decorators.push("state".to_string());
            }
            conflicting_decorators.sort();
            conflicting_decorators.dedup();

            let declaration_only = property.initializer.is_none()
                && (property.is_definite_assignment || property.is_declare);
            let mut violations = Vec::new();
            if owner.is_none() {
                violations.push(FormDeclarationViolation::InvalidOwner);
            }
            if !property.is_identifier_name {
                violations.push(FormDeclarationViolation::UnsupportedFieldName);
            }
            if !decorator.is_invoked {
                violations.push(FormDeclarationViolation::InvalidDecoratorInvocation);
            }
            if decorator.argument_count != 0 {
                violations.push(FormDeclarationViolation::InvalidDecoratorArity {
                    actual: decorator.argument_count,
                    expected: 0,
                });
            }
            if form_decorator_count != 1 {
                violations.push(FormDeclarationViolation::DuplicateFormDecorator);
            }
            if property.is_static {
                violations.push(FormDeclarationViolation::StaticField);
            }
            if property.initializer.is_some() {
                violations.push(FormDeclarationViolation::InitializedField);
            } else if !declaration_only {
                violations.push(FormDeclarationViolation::DeclarationOnlyRequired);
            }
            match &declared_type {
                None => violations.push(FormDeclarationViolation::MissingType),
                Some(declared) if !builtin_types.recognizes_form_marker(&declared.text) => {
                    violations.push(FormDeclarationViolation::InvalidType {
                        actual: Some(declared.text.clone()),
                    });
                }
                Some(_) => {}
            }
            if !conflicting_decorators.is_empty() {
                violations.push(FormDeclarationViolation::ConflictingSemanticDecorator);
            }
            canonicalize_form_violations(&mut violations);

            candidates.push(FormDeclarationCandidate {
                id: FormDeclarationCandidateId::for_source_position(path, decorator.span.start),
                owner_component: owner.clone(),
                form_id,
                authored_field,
                authored_name: property.is_identifier_name.then(|| property.name.clone()),
                declaration_kind,
                decorator_invoked: decorator.is_invoked,
                decorator_argument_count: decorator.argument_count,
                decorator_argument_provenance: decorator
                    .argument_spans
                    .iter()
                    .copied()
                    .map(|span| SourceProvenance::new(path, span))
                    .collect(),
                declaration_only,
                declared_type,
                conflicting_decorators,
                decorator_provenance: SourceProvenance::new(path, decorator.span),
                name_provenance: property
                    .is_identifier_name
                    .then(|| SourceProvenance::new(path, property.name_span)),
                initializer_provenance: property
                    .initializer_span
                    .map(|span| SourceProvenance::new(path, span)),
                provenance: SourceProvenance::new(path, property.span),
                status: form_declaration_status(violations),
            });
        }
    }

    for method in &class.methods {
        retain_non_field_form_candidates(
            &mut candidates,
            owner.as_ref(),
            path,
            &method.name,
            if method.is_getter {
                AuthoredDeclarationKind::Getter
            } else if method.is_setter {
                AuthoredDeclarationKind::Setter
            } else {
                AuthoredDeclarationKind::Method
            },
            &method.decorators,
            method.span,
        );
        for parameter in &method.parameters {
            retain_non_field_form_candidates(
                &mut candidates,
                owner.as_ref(),
                path,
                &parameter.name,
                AuthoredDeclarationKind::Parameter,
                &parameter.decorators,
                parameter.span,
            );
        }
    }

    let mut duplicate_groups = BTreeMap::<(SemanticId, String), Vec<usize>>::new();
    for (index, candidate) in candidates.iter().enumerate() {
        if candidate.form_id.is_none() {
            continue;
        }
        if let (Some(owner), Some(name)) = (
            candidate.owner_component.as_ref(),
            candidate.authored_name.as_ref(),
        ) {
            duplicate_groups
                .entry((owner.clone(), name.clone()))
                .or_default()
                .push(index);
        }
    }
    let duplicate_groups = duplicate_groups
        .into_values()
        .filter(|indexes| {
            indexes
                .iter()
                .map(|index| {
                    let provenance = &candidates[*index].provenance;
                    (
                        provenance.path.as_path(),
                        provenance.span.start,
                        provenance.span.end,
                    )
                })
                .collect::<BTreeSet<_>>()
                .len()
                > 1
        })
        .collect::<Vec<_>>();
    for group in duplicate_groups {
        for index in group {
            let candidate = &mut candidates[index];
            let mut violations = candidate.violations().to_vec();
            violations.push(FormDeclarationViolation::DuplicateName);
            canonicalize_form_violations(&mut violations);
            candidate.status = FormDeclarationStatus::Invalid(violations);
        }
    }
    candidates.sort_by(|left, right| {
        (
            left.provenance.path.as_path(),
            left.provenance.span.start,
            left.provenance.span.end,
            left.id.as_str(),
        )
            .cmp(&(
                right.provenance.path.as_path(),
                right.provenance.span.start,
                right.provenance.span.end,
                right.id.as_str(),
            ))
    });
    candidates
}

fn retain_non_field_form_candidates(
    candidates: &mut Vec<FormDeclarationCandidate>,
    owner: Option<&SemanticId>,
    path: &Path,
    authored_name: &str,
    declaration_kind: AuthoredDeclarationKind,
    decorators: &[presolve_parser::ParsedDecorator],
    span: SourceSpan,
) {
    let form_decorator_count = decorators
        .iter()
        .filter(|decorator| decorator.name == "form")
        .count();
    for decorator in decorators
        .iter()
        .filter(|decorator| decorator.name == "form")
    {
        let mut violations = vec![FormDeclarationViolation::InvalidTarget {
            actual: declaration_kind,
        }];
        if owner.is_none() {
            violations.push(FormDeclarationViolation::InvalidOwner);
        }
        if !decorator.is_invoked {
            violations.push(FormDeclarationViolation::InvalidDecoratorInvocation);
        }
        if decorator.argument_count != 0 {
            violations.push(FormDeclarationViolation::InvalidDecoratorArity {
                actual: decorator.argument_count,
                expected: 0,
            });
        }
        if form_decorator_count != 1 {
            violations.push(FormDeclarationViolation::DuplicateFormDecorator);
        }
        canonicalize_form_violations(&mut violations);
        candidates.push(FormDeclarationCandidate {
            id: FormDeclarationCandidateId::for_source_position(path, decorator.span.start),
            owner_component: owner.cloned(),
            form_id: None,
            authored_field: None,
            authored_name: Some(authored_name.to_string()),
            declaration_kind,
            decorator_invoked: decorator.is_invoked,
            decorator_argument_count: decorator.argument_count,
            decorator_argument_provenance: decorator
                .argument_spans
                .iter()
                .copied()
                .map(|span| SourceProvenance::new(path, span))
                .collect(),
            declaration_only: false,
            declared_type: None,
            conflicting_decorators: Vec::new(),
            decorator_provenance: SourceProvenance::new(path, decorator.span),
            name_provenance: None,
            initializer_provenance: None,
            provenance: SourceProvenance::new(path, span),
            status: FormDeclarationStatus::Invalid(violations),
        });
    }
}

#[allow(clippy::too_many_lines)]
fn form_field_declaration_candidates_from_class(
    class: &ParsedClass,
    path: &Path,
    is_canonical_component: bool,
    component: &SemanticId,
) -> Vec<FormFieldDeclarationCandidate> {
    let owner = is_canonical_component.then_some(component.clone());
    let mut candidates = Vec::new();

    retain_non_property_form_field_candidates(
        &mut candidates,
        owner.as_ref(),
        path,
        &class.name,
        AuthoredDeclarationKind::Class,
        &class.decorators,
        class.span,
    );

    for property in &class.properties {
        let field_decorator_count = property
            .decorators
            .iter()
            .filter(|decorator| decorator.name == "field")
            .count();
        for decorator in property
            .decorators
            .iter()
            .filter(|decorator| decorator.name == "field")
        {
            let declaration_kind = if property.is_static {
                AuthoredDeclarationKind::StaticField
            } else {
                AuthoredDeclarationKind::InstanceField
            };
            let initializer = property
                .initializer_literal
                .as_ref()
                .map(serializable_value_from_parsed)
                .or_else(|| {
                    property
                        .initializer_constant_expression
                        .as_ref()
                        .map(constant_expression_from_parsed)
                        .and_then(|expression| expression.evaluate().ok())
                });
            let declared_type =
                property
                    .type_annotation
                    .as_ref()
                    .map(|annotation| DeclaredStateType {
                        kind: declared_state_type_kind(&annotation.text),
                        text: annotation.text.clone(),
                        provenance: SourceProvenance::new(path, annotation.span),
                    });
            let mut conflicting_decorators = property
                .decorators
                .iter()
                .filter(|other| !matches!(other.name.as_str(), "field" | "validate"))
                .map(|other| other.name.clone())
                .collect::<Vec<_>>();
            conflicting_decorators.sort();
            conflicting_decorators.dedup();
            let (form_designator, unsupported_form_designator) =
                normalized_form_designator(decorator, path);
            let mut violations = Vec::new();
            if owner.is_none() {
                violations.push(FormFieldDeclarationViolation::InvalidOwner);
            }
            if !property.is_identifier_name {
                violations.push(FormFieldDeclarationViolation::UnsupportedFieldName);
            }
            if !decorator.is_invoked {
                violations.push(FormFieldDeclarationViolation::InvalidDecoratorInvocation);
            }
            if decorator.argument_count != 1 {
                violations.push(FormFieldDeclarationViolation::InvalidDecoratorArity {
                    actual: decorator.argument_count,
                    expected: 1,
                });
            }
            if field_decorator_count != 1 {
                violations.push(FormFieldDeclarationViolation::DuplicateFieldDecorator);
            }
            if form_designator.is_none() {
                violations.push(FormFieldDeclarationViolation::InvalidFormDesignator);
            }
            if property.is_static {
                violations.push(FormFieldDeclarationViolation::StaticField);
            }
            if property.initializer_span.is_none() {
                violations.push(FormFieldDeclarationViolation::MissingInitializer);
            } else if initializer.is_none() {
                violations.push(FormFieldDeclarationViolation::UnsupportedInitializer);
            }
            if !conflicting_decorators.is_empty() {
                violations.push(FormFieldDeclarationViolation::ConflictingSemanticDecorator);
            }
            canonicalize_form_field_violations(&mut violations);

            candidates.push(FormFieldDeclarationCandidate {
                id: FormFieldDeclarationCandidateId::for_source_position(
                    path,
                    decorator.span.start,
                ),
                owner_component: owner.clone(),
                declaration_field: (owner.is_some() && property.is_identifier_name).then(|| {
                    owner
                        .as_ref()
                        .expect("authored field identity requires component owner")
                        .field(&property.name)
                }),
                authored_name: property.is_identifier_name.then(|| property.name.clone()),
                field_id: None,
                decorator_invoked: decorator.is_invoked,
                decorator_argument_count: decorator.argument_count,
                decorator_argument_provenance: decorator
                    .argument_spans
                    .iter()
                    .copied()
                    .map(|span| SourceProvenance::new(path, span))
                    .collect(),
                form_designator,
                unsupported_form_designator,
                resolved_form: None,
                declaration_kind,
                is_static: property.is_static,
                declared_type,
                semantic_type: None,
                type_assignment: None,
                initializer,
                conflicting_decorators,
                decorator_provenance: SourceProvenance::new(path, decorator.span),
                name_provenance: property
                    .is_identifier_name
                    .then(|| SourceProvenance::new(path, property.name_span)),
                initializer_provenance: property
                    .initializer_span
                    .map(|span| SourceProvenance::new(path, span)),
                provenance: SourceProvenance::new(path, property.span),
                violations,
            });
        }
    }

    for method in &class.methods {
        let kind = if method.is_getter {
            AuthoredDeclarationKind::Getter
        } else if method.is_setter {
            AuthoredDeclarationKind::Setter
        } else {
            AuthoredDeclarationKind::Method
        };
        retain_non_property_form_field_candidates(
            &mut candidates,
            owner.as_ref(),
            path,
            &method.name,
            kind,
            &method.decorators,
            method.span,
        );
        for parameter in &method.parameters {
            retain_non_property_form_field_candidates(
                &mut candidates,
                owner.as_ref(),
                path,
                &parameter.name,
                AuthoredDeclarationKind::Parameter,
                &parameter.decorators,
                parameter.span,
            );
        }
    }

    candidates.sort_by(form_field_candidate_source_order);
    candidates
}

#[allow(clippy::too_many_lines)]
fn validation_rule_declaration_facts_from_class(
    class: &ParsedClass,
    path: &Path,
    is_canonical_component: bool,
    component: &SemanticId,
) -> Vec<AuthoredValidationRuleDeclarationFact> {
    let owner = is_canonical_component.then_some(component);
    let mut facts = Vec::new();

    retain_validation_rule_facts(
        &mut facts,
        owner,
        None,
        Some(class.name.as_str()),
        AuthoredDeclarationKind::Class,
        false,
        &class.decorators,
        None,
        class.span,
        path,
    );

    for property in &class.properties {
        let declaration_field = (owner.is_some() && property.is_identifier_name)
            .then(|| component.field(&property.name));
        let kind = if property.is_static {
            AuthoredDeclarationKind::StaticField
        } else {
            AuthoredDeclarationKind::InstanceField
        };
        retain_validation_rule_facts(
            &mut facts,
            owner,
            declaration_field.as_ref(),
            property
                .is_identifier_name
                .then_some(property.name.as_str()),
            kind,
            property.is_static,
            &property.decorators,
            property.is_identifier_name.then_some(property.name_span),
            property.span,
            path,
        );
    }

    for method in &class.methods {
        let kind = if method.is_getter {
            AuthoredDeclarationKind::Getter
        } else if method.is_setter {
            AuthoredDeclarationKind::Setter
        } else {
            AuthoredDeclarationKind::Method
        };
        retain_validation_rule_facts(
            &mut facts,
            owner,
            None,
            Some(method.name.as_str()),
            kind,
            false,
            &method.decorators,
            None,
            method.span,
            path,
        );
        for parameter in &method.parameters {
            retain_validation_rule_facts(
                &mut facts,
                owner,
                None,
                Some(parameter.name.as_str()),
                AuthoredDeclarationKind::Parameter,
                false,
                &parameter.decorators,
                None,
                parameter.span,
                path,
            );
        }
    }

    facts.sort_by(|left, right| {
        (
            left.provenance.path.as_path(),
            left.decorator_provenance.span.start,
            left.id.as_str(),
        )
            .cmp(&(
                right.provenance.path.as_path(),
                right.decorator_provenance.span.start,
                right.id.as_str(),
            ))
    });
    facts
}

fn submission_declaration_facts_from_class(
    class: &ParsedClass,
    path: &Path,
    is_canonical_component: bool,
    component: &SemanticId,
) -> Vec<AuthoredSubmissionDeclarationFact> {
    let mut facts = class
        .methods
        .iter()
        .flat_map(|method| {
            let action = method
                .decorators
                .iter()
                .find(|decorator| decorator.name == "action");
            method
                .decorators
                .iter()
                .filter(move |decorator| decorator.name == "submit")
                .map(move |decorator| AuthoredSubmissionDeclarationFact {
                    id: SubmissionDeclarationCandidateId::for_source_position(
                        path,
                        decorator.span.start,
                    ),
                    owner_component: is_canonical_component.then(|| component.clone()),
                    method: is_canonical_component.then(|| component.method(&method.name)),
                    method_name: Some(method.name.clone()),
                    is_static: method.is_static,
                    is_async: method.is_async,
                    parameter_count: method.parameters.len(),
                    return_type: method
                        .return_type_annotation
                        .as_ref()
                        .map(|annotation| annotation.text.clone()),
                    submit_invoked: decorator.is_invoked,
                    submit_argument_count: decorator.argument_count,
                    form_designator: normalized_submission_form_designator(decorator),
                    has_action: action.is_some(),
                    action_invoked: action.is_some_and(|decorator| decorator.is_invoked),
                    action_argument_count: action.map_or(0, |decorator| decorator.argument_count),
                    inherited: class.heritage.is_some(),
                    decorator_provenance: SourceProvenance::new(path, decorator.span),
                    form_designator_provenance: decorator
                        .argument_spans
                        .first()
                        .map(|span| SourceProvenance::new(path, *span)),
                    method_provenance: SourceProvenance::new(path, method.span),
                })
        })
        .collect::<Vec<_>>();
    facts.sort_by(|left, right| {
        (
            left.decorator_provenance.path.as_path(),
            left.decorator_provenance.span.start,
            left.id.as_str(),
        )
            .cmp(&(
                right.decorator_provenance.path.as_path(),
                right.decorator_provenance.span.start,
                right.id.as_str(),
            ))
    });
    facts
}

fn serialization_declaration_facts_from_class(
    class: &ParsedClass,
    path: &Path,
    is_canonical_component: bool,
    component: &SemanticId,
) -> Vec<AuthoredSerializationDeclarationFact> {
    let mut facts = class
        .properties
        .iter()
        .flat_map(|property| {
            property
                .decorators
                .iter()
                .filter(|decorator| decorator.name == "serialize")
                .map(|decorator| AuthoredSerializationDeclarationFact {
                    owner_component: is_canonical_component.then(|| component.clone()),
                    declaration_field: (is_canonical_component && property.is_identifier_name)
                        .then(|| component.form_field(&property.name)),
                    authored_name: property.is_identifier_name.then(|| property.name.clone()),
                    invoked: decorator.is_invoked,
                    argument_count: decorator.argument_count,
                    format: decorator.argument.clone(),
                    provenance: SourceProvenance::new(path, property.span),
                    decorator_provenance: SourceProvenance::new(path, decorator.span),
                })
        })
        .collect::<Vec<_>>();
    facts.sort_by(|left, right| {
        (
            left.decorator_provenance.path.as_path(),
            left.decorator_provenance.span.start,
        )
            .cmp(&(
                right.decorator_provenance.path.as_path(),
                right.decorator_provenance.span.start,
            ))
    });
    facts
}

#[allow(clippy::too_many_arguments)]
fn retain_validation_rule_facts(
    facts: &mut Vec<AuthoredValidationRuleDeclarationFact>,
    owner: Option<&SemanticId>,
    declaration_field: Option<&SemanticId>,
    authored_name: Option<&str>,
    declaration_kind: AuthoredDeclarationKind,
    is_static: bool,
    decorators: &[presolve_parser::ParsedDecorator],
    name_span: Option<SourceSpan>,
    declaration_span: SourceSpan,
    path: &Path,
) {
    let mut conflicting_decorators = decorators
        .iter()
        .filter(|decorator| {
            !matches!(decorator.name.as_str(), "field" | "validate")
                && is_presolve_semantic_decorator(&decorator.name)
        })
        .map(|decorator| decorator.name.clone())
        .collect::<Vec<_>>();
    conflicting_decorators.sort();
    conflicting_decorators.dedup();

    for (authored_ordinal, decorator) in decorators
        .iter()
        .filter(|decorator| decorator.name == "validate")
        .enumerate()
    {
        facts.push(AuthoredValidationRuleDeclarationFact {
            id: ValidationRuleCandidateId::for_source_position(path, decorator.span.start),
            owner_component: owner.cloned(),
            declaration_field: declaration_field.cloned(),
            authored_name: authored_name.map(str::to_string),
            declaration_kind,
            is_static,
            authored_ordinal,
            decorator_invoked: decorator.is_invoked,
            decorator_argument_count: decorator.argument_count,
            expression: decorator
                .validation_rule_expression
                .as_ref()
                .map(|expression| authored_validation_rule_expression(expression, path)),
            conflicting_decorators: conflicting_decorators.clone(),
            decorator_provenance: SourceProvenance::new(path, decorator.span),
            name_provenance: name_span.map(|span| SourceProvenance::new(path, span)),
            provenance: SourceProvenance::new(path, declaration_span),
        });
    }
}

fn authored_validation_rule_expression(
    expression: &presolve_parser::ParsedValidationRuleExpression,
    path: &Path,
) -> AuthoredValidationRuleExpression {
    let kind = match &expression.kind {
        ParsedValidationRuleExpressionKind::Call { callee, arguments } => {
            AuthoredValidationRuleExpressionKind::Call {
                callee: callee.clone(),
                arguments: arguments
                    .iter()
                    .map(|argument| AuthoredValidationRuleArgument {
                        kind: match &argument.kind {
                            ParsedValidationRuleArgumentKind::StringLiteral(value) => {
                                AuthoredValidationRuleArgumentKind::StringLiteral(value.clone())
                            }
                            ParsedValidationRuleArgumentKind::Constant(constant) => {
                                AuthoredValidationRuleArgumentKind::Constant(
                                    constant_expression_from_parsed(constant),
                                )
                            }
                            ParsedValidationRuleArgumentKind::ThisMember(designator) => {
                                AuthoredValidationRuleArgumentKind::ThisMember {
                                    name: designator.member.clone(),
                                    name_provenance: SourceProvenance::new(
                                        path,
                                        designator.member_span,
                                    ),
                                }
                            }
                            ParsedValidationRuleArgumentKind::Unsupported => {
                                AuthoredValidationRuleArgumentKind::Unsupported
                            }
                        },
                        provenance: SourceProvenance::new(path, argument.span),
                    })
                    .collect(),
            }
        }
        ParsedValidationRuleExpressionKind::Identifier(identifier) => {
            AuthoredValidationRuleExpressionKind::Identifier(identifier.clone())
        }
        ParsedValidationRuleExpressionKind::Unsupported => {
            AuthoredValidationRuleExpressionKind::Unsupported
        }
    };
    AuthoredValidationRuleExpression {
        kind,
        provenance: SourceProvenance::new(path, expression.span),
    }
}

fn retain_non_property_form_field_candidates(
    candidates: &mut Vec<FormFieldDeclarationCandidate>,
    owner: Option<&SemanticId>,
    path: &Path,
    authored_name: &str,
    declaration_kind: AuthoredDeclarationKind,
    decorators: &[presolve_parser::ParsedDecorator],
    span: SourceSpan,
) {
    let field_decorator_count = decorators
        .iter()
        .filter(|decorator| decorator.name == "field")
        .count();
    for decorator in decorators
        .iter()
        .filter(|decorator| decorator.name == "field")
    {
        let (form_designator, unsupported_form_designator) =
            normalized_form_designator(decorator, path);
        let mut violations = vec![FormFieldDeclarationViolation::InvalidTarget {
            actual: declaration_kind,
        }];
        if owner.is_none() {
            violations.push(FormFieldDeclarationViolation::InvalidOwner);
        }
        if !decorator.is_invoked {
            violations.push(FormFieldDeclarationViolation::InvalidDecoratorInvocation);
        }
        if decorator.argument_count != 1 {
            violations.push(FormFieldDeclarationViolation::InvalidDecoratorArity {
                actual: decorator.argument_count,
                expected: 1,
            });
        }
        if field_decorator_count != 1 {
            violations.push(FormFieldDeclarationViolation::DuplicateFieldDecorator);
        }
        if form_designator.is_none() {
            violations.push(FormFieldDeclarationViolation::InvalidFormDesignator);
        }
        canonicalize_form_field_violations(&mut violations);
        candidates.push(FormFieldDeclarationCandidate {
            id: FormFieldDeclarationCandidateId::for_source_position(path, decorator.span.start),
            owner_component: owner.cloned(),
            declaration_field: None,
            authored_name: Some(authored_name.to_string()),
            field_id: None,
            decorator_invoked: decorator.is_invoked,
            decorator_argument_count: decorator.argument_count,
            decorator_argument_provenance: decorator
                .argument_spans
                .iter()
                .copied()
                .map(|argument| SourceProvenance::new(path, argument))
                .collect(),
            form_designator,
            unsupported_form_designator,
            resolved_form: None,
            declaration_kind,
            is_static: false,
            declared_type: None,
            semantic_type: None,
            type_assignment: None,
            initializer: None,
            conflicting_decorators: Vec::new(),
            decorator_provenance: SourceProvenance::new(path, decorator.span),
            name_provenance: None,
            initializer_provenance: None,
            provenance: SourceProvenance::new(path, span),
            violations,
        });
    }
}

fn normalized_form_designator(
    decorator: &presolve_parser::ParsedDecorator,
    path: &Path,
) -> (
    Option<FormDesignatorFact>,
    Option<UnsupportedFormDesignatorFact>,
) {
    let designator = decorator
        .this_member_argument
        .as_ref()
        .map(|designator| FormDesignatorFact {
            authored_name: designator.member.clone(),
            provenance: SourceProvenance::new(path, designator.span),
            name_provenance: SourceProvenance::new(path, designator.member_span),
        })
        .or_else(|| {
            let span = *decorator.argument_spans.first()?;
            decorator
                .argument
                .as_ref()
                .filter(|name| form_designator_name_is_valid(name))
                .map(|name| FormDesignatorFact {
                    authored_name: name.clone(),
                    provenance: SourceProvenance::new(path, span),
                    name_provenance: SourceProvenance::new(path, span),
                })
        });
    let unsupported =
        decorator
            .static_member_argument
            .as_ref()
            .map(|designator| UnsupportedFormDesignatorFact {
                object: designator.object.clone(),
                member: designator.member.clone(),
                provenance: SourceProvenance::new(path, designator.span),
            });
    (designator, unsupported)
}

fn normalized_submission_form_designator(
    decorator: &presolve_parser::ParsedDecorator,
) -> Option<String> {
    decorator
        .this_member_argument
        .as_ref()
        .map(|designator| designator.member.clone())
        .or_else(|| {
            decorator
                .argument
                .as_ref()
                .filter(|name| form_designator_name_is_valid(name))
                .cloned()
        })
}

fn form_designator_name_is_valid(name: &str) -> bool {
    let mut characters = name.chars();
    matches!(characters.next(), Some(character) if character == '_' || character == '$' || character.is_ascii_alphabetic())
        && characters.all(|character| {
            character == '_' || character == '$' || character.is_ascii_alphanumeric()
        })
}

fn canonicalize_form_field_violations(violations: &mut Vec<FormFieldDeclarationViolation>) {
    violations.sort_by_key(form_field_declaration_violation_rank);
    violations.dedup();
}

fn form_field_declaration_violation_rank(violation: &FormFieldDeclarationViolation) -> u8 {
    match violation {
        FormFieldDeclarationViolation::InvalidOwner => 0,
        FormFieldDeclarationViolation::InvalidTarget { .. }
        | FormFieldDeclarationViolation::StaticField
        | FormFieldDeclarationViolation::InheritedDeclaration
        | FormFieldDeclarationViolation::UnsupportedFieldName => 1,
        FormFieldDeclarationViolation::InvalidDecoratorInvocation
        | FormFieldDeclarationViolation::InvalidDecoratorArity { .. }
        | FormFieldDeclarationViolation::DuplicateFieldDecorator => 2,
        FormFieldDeclarationViolation::InvalidFormDesignator
        | FormFieldDeclarationViolation::UnresolvedForm
        | FormFieldDeclarationViolation::InvalidForm
        | FormFieldDeclarationViolation::CrossComponentForm => 3,
        FormFieldDeclarationViolation::MissingInitializer
        | FormFieldDeclarationViolation::UnsupportedInitializer => 4,
        FormFieldDeclarationViolation::InvalidDeclaredType
        | FormFieldDeclarationViolation::InitializerTypeMismatch
        | FormFieldDeclarationViolation::NonSerializableType => 5,
        FormFieldDeclarationViolation::ConflictingSemanticDecorator => 6,
        FormFieldDeclarationViolation::DuplicateName => 7,
    }
}

fn form_field_candidate_source_order(
    left: &FormFieldDeclarationCandidate,
    right: &FormFieldDeclarationCandidate,
) -> std::cmp::Ordering {
    (
        left.provenance.path.as_path(),
        left.provenance.span.start,
        left.provenance.span.end,
        left.id.as_str(),
    )
        .cmp(&(
            right.provenance.path.as_path(),
            right.provenance.span.start,
            right.provenance.span.end,
            right.id.as_str(),
        ))
}

fn is_presolve_semantic_decorator(name: &str) -> bool {
    matches!(
        name,
        "form"
            | "state"
            | "context"
            | "provide"
            | "provider"
            | "consume"
            | "consumer"
            | "computed"
            | "effect"
            | "action"
            | "resource"
            | "slot"
            | "field"
            | "submit"
            | "validate"
    )
}

fn is_validation_intrinsic_name(name: &str) -> bool {
    matches!(
        name,
        "required"
            | "min"
            | "max"
            | "minLength"
            | "maxLength"
            | "pattern"
            | "email"
            | "equals"
            | "notEquals"
    )
}

fn form_declaration_status(violations: Vec<FormDeclarationViolation>) -> FormDeclarationStatus {
    if violations.is_empty() {
        FormDeclarationStatus::Valid
    } else {
        FormDeclarationStatus::Invalid(violations)
    }
}

fn canonicalize_form_violations(violations: &mut Vec<FormDeclarationViolation>) {
    violations.sort_by_key(form_declaration_violation_rank);
    violations.dedup();
}

fn form_declaration_violation_rank(violation: &FormDeclarationViolation) -> u8 {
    match violation {
        FormDeclarationViolation::InvalidOwner => 0,
        FormDeclarationViolation::InvalidTarget { .. }
        | FormDeclarationViolation::UnsupportedFieldName
        | FormDeclarationViolation::InheritedDeclaration => 1,
        FormDeclarationViolation::InvalidDecoratorInvocation
        | FormDeclarationViolation::InvalidDecoratorArity { .. }
        | FormDeclarationViolation::DuplicateFormDecorator => 2,
        FormDeclarationViolation::StaticField => 3,
        FormDeclarationViolation::InitializedField
        | FormDeclarationViolation::DeclarationOnlyRequired => 4,
        FormDeclarationViolation::MissingType | FormDeclarationViolation::InvalidType { .. } => 5,
        FormDeclarationViolation::ConflictingSemanticDecorator => 6,
        FormDeclarationViolation::DuplicateName => 7,
    }
}

#[allow(clippy::too_many_lines)]
fn context_declaration_candidates_from_class(
    class: &ParsedClass,
    path: &Path,
    component: &SemanticId,
) -> Vec<AuthoredContextDeclarationCandidate> {
    let mut candidates = Vec::new();
    for property in &class.properties {
        for decorator in property.decorators.iter().filter(|decorator| {
            matches!(decorator.name.as_str(), "context" | "provide" | "consume")
        }) {
            let kind = match decorator.name.as_str() {
                "context" => ContextDeclarationCandidateKind::Context,
                "provide" => ContextDeclarationCandidateKind::Provider,
                "consume" => ContextDeclarationCandidateKind::Consumer,
                _ => unreachable!("filtered Context decorator"),
            };
            let declaration_kind = if property.is_static {
                AuthoredDeclarationKind::StaticField
            } else {
                AuthoredDeclarationKind::InstanceField
            };
            let mut violations = Vec::new();
            let expected_arity = usize::from(kind != ContextDeclarationCandidateKind::Context);
            if decorator.argument_count != expected_arity {
                violations.push(ContextDeclarationViolation::DecoratorArity {
                    actual: decorator.argument_count,
                    expected: expected_arity,
                });
            }
            if property.is_static && kind != ContextDeclarationCandidateKind::Context {
                violations.push(ContextDeclarationViolation::StaticDeclarationUnsupported);
            }
            let has_conflict = property.decorators.iter().any(|other| {
                other.name != decorator.name
                    && matches!(
                        other.name.as_str(),
                        "state"
                            | "context"
                            | "provide"
                            | "consume"
                            | "computed"
                            | "effect"
                            | "action"
                            | "resource"
                            | "slot"
                            | "form"
                            | "field"
                    )
            });
            if has_conflict {
                violations.push(ContextDeclarationViolation::ConflictingSemanticDecorators);
            }
            let declared_type =
                property
                    .type_annotation
                    .as_ref()
                    .map(|annotation| DeclaredStateType {
                        kind: declared_state_type_kind(&annotation.text),
                        text: annotation.text.clone(),
                        provenance: SourceProvenance::new(path, annotation.span),
                    });
            if declared_type.is_none() {
                violations.push(ContextDeclarationViolation::MissingDeclaredType);
            }
            let designator = context_designator_from_decorator(decorator, path);
            match kind {
                ContextDeclarationCandidateKind::Context => {
                    if property.initializer.is_some() && property.initializer_literal.is_none() {
                        violations.push(ContextDeclarationViolation::UnsupportedInitializer);
                    }
                }
                ContextDeclarationCandidateKind::Provider => {
                    if decorator.argument_count == 1 && designator.is_none() {
                        violations.push(ContextDeclarationViolation::ContextDesignatorUnsupported);
                    }
                    if property.initializer.is_none() {
                        violations.push(ContextDeclarationViolation::MissingInitializer);
                    } else if property.initializer_expression.is_none() {
                        violations.push(ContextDeclarationViolation::UnsupportedInitializer);
                    }
                }
                ContextDeclarationCandidateKind::Consumer => {
                    if decorator.argument_count == 1 && designator.is_none() {
                        violations.push(ContextDeclarationViolation::ContextDesignatorUnsupported);
                    }
                    if property.initializer.is_some() {
                        violations.push(ContextDeclarationViolation::ForbiddenInitializer);
                    }
                    if !property.is_definite_assignment {
                        violations.push(ContextDeclarationViolation::DefiniteAssignmentRequired);
                    }
                }
            }
            candidates.push(AuthoredContextDeclarationCandidate {
                id: ContextDeclarationCandidateId::for_component_position(
                    component,
                    decorator.span.start,
                ),
                kind,
                owner_component: component.clone(),
                authored_declaration: component
                    .context_declaration_candidate(property.name_span.start),
                declaration_kind,
                field_name: Some(property.name.clone()),
                declared_type,
                context_designator: designator,
                decorator_argument_count: decorator.argument_count,
                decorator_provenance: SourceProvenance::new(path, decorator.span),
                provenance: SourceProvenance::new(path, property.span),
                static_modifier_provenance: property
                    .is_static
                    .then(|| SourceProvenance::new(path, property.span)),
                initializer_provenance: property
                    .initializer_span
                    .map(|span| SourceProvenance::new(path, span)),
                violations,
            });
        }
    }
    for method in &class.methods {
        for decorator in method.decorators.iter().filter(|decorator| {
            matches!(decorator.name.as_str(), "context" | "provide" | "consume")
        }) {
            let kind = match decorator.name.as_str() {
                "context" => ContextDeclarationCandidateKind::Context,
                "provide" => ContextDeclarationCandidateKind::Provider,
                "consume" => ContextDeclarationCandidateKind::Consumer,
                _ => unreachable!("filtered Context decorator"),
            };
            let expected_arity = usize::from(kind != ContextDeclarationCandidateKind::Context);
            let mut violations = vec![ContextDeclarationViolation::InvalidDeclarationKind {
                actual: if method.is_getter {
                    AuthoredDeclarationKind::Getter
                } else {
                    AuthoredDeclarationKind::Method
                },
            }];
            if decorator.argument_count != expected_arity {
                violations.push(ContextDeclarationViolation::DecoratorArity {
                    actual: decorator.argument_count,
                    expected: expected_arity,
                });
            }
            candidates.push(AuthoredContextDeclarationCandidate {
                id: ContextDeclarationCandidateId::for_component_position(
                    component,
                    decorator.span.start,
                ),
                kind,
                owner_component: component.clone(),
                authored_declaration: component.method(&method.name),
                declaration_kind: if method.is_getter {
                    AuthoredDeclarationKind::Getter
                } else {
                    AuthoredDeclarationKind::Method
                },
                field_name: None,
                declared_type: None,
                context_designator: decorator
                    .static_member_argument
                    .as_ref()
                    .map(|value| context_designator_from_parsed(value, path)),
                decorator_argument_count: decorator.argument_count,
                decorator_provenance: SourceProvenance::new(path, decorator.span),
                provenance: SourceProvenance::new(path, method.span),
                static_modifier_provenance: None,
                initializer_provenance: None,
                violations,
            });
        }
    }
    candidates.sort_by(|left, right| left.id.cmp(&right.id));
    candidates
}

#[allow(clippy::too_many_lines)]
fn slot_declaration_candidates_from_class(
    class: &ParsedClass,
    path: &Path,
    component: &SemanticId,
) -> Vec<AuthoredSlotDeclarationCandidate> {
    let mut candidates = Vec::new();

    for property in &class.properties {
        for decorator in property
            .decorators
            .iter()
            .filter(|decorator| decorator.name == "slot")
        {
            let declaration_kind = if property.is_static {
                AuthoredDeclarationKind::StaticField
            } else {
                AuthoredDeclarationKind::InstanceField
            };
            let mut violations = Vec::new();
            if decorator.argument_count != 0 {
                violations.push(SlotDeclarationViolation::DecoratorArity {
                    actual: decorator.argument_count,
                    expected: 0,
                });
            }
            if property.is_static {
                violations.push(SlotDeclarationViolation::StaticDeclarationUnsupported);
            }
            let has_conflict = property.initializer.as_deref() == Some("state(...)")
                || property.decorators.iter().any(|other| {
                    other.name != "slot"
                        && matches!(
                            other.name.as_str(),
                            "context"
                                | "provide"
                                | "consume"
                                | "computed"
                                | "effect"
                                | "action"
                                | "resource"
                                | "form"
                                | "field"
                        )
                });
            if has_conflict {
                violations.push(SlotDeclarationViolation::ConflictingSemanticDecorators);
            }
            let declared_type =
                property
                    .type_annotation
                    .as_ref()
                    .map(|annotation| DeclaredStateType {
                        kind: declared_state_type_kind(&annotation.text),
                        text: annotation.text.clone(),
                        provenance: SourceProvenance::new(path, annotation.span),
                    });
            if declared_type
                .as_ref()
                .is_none_or(|declared| declared.text != "SlotContent")
            {
                violations.push(SlotDeclarationViolation::InvalidDeclaredType {
                    actual: declared_type.as_ref().map(|declared| declared.text.clone()),
                });
            }
            if property.initializer.is_some() {
                violations.push(SlotDeclarationViolation::ForbiddenInitializer);
            }
            if !property.is_definite_assignment {
                violations.push(SlotDeclarationViolation::DefiniteAssignmentRequired);
            }
            candidates.push(AuthoredSlotDeclarationCandidate {
                id: SlotDeclarationCandidateId::for_component_position(
                    component,
                    decorator.span.start,
                ),
                owner_component: component.clone(),
                authored_declaration: component.slot_field(&property.name),
                declaration_kind,
                field_name: Some(property.name.clone()),
                declared_type,
                decorator_argument_count: decorator.argument_count,
                decorator_provenance: SourceProvenance::new(path, decorator.span),
                name_provenance: Some(SourceProvenance::new(path, property.name_span)),
                provenance: SourceProvenance::new(path, property.span),
                static_modifier_provenance: property
                    .is_static
                    .then(|| SourceProvenance::new(path, property.span)),
                initializer_provenance: property
                    .initializer_span
                    .map(|span| SourceProvenance::new(path, span)),
                violations,
            });
        }
    }

    for method in &class.methods {
        retain_invalid_slot_candidate(
            &mut candidates,
            component,
            &component.method(&method.name),
            if method.is_getter {
                AuthoredDeclarationKind::Getter
            } else if method.is_setter {
                AuthoredDeclarationKind::Setter
            } else {
                AuthoredDeclarationKind::Method
            },
            None,
            &method.decorators,
            path,
            method.span,
        );
        for (index, parameter) in method.parameters.iter().enumerate() {
            retain_invalid_slot_candidate(
                &mut candidates,
                component,
                &component
                    .method(&method.name)
                    .parameter(&parameter.name, index),
                AuthoredDeclarationKind::Parameter,
                Some(&parameter.name),
                &parameter.decorators,
                path,
                parameter.span,
            );
        }
    }

    let mut counts = BTreeMap::<String, usize>::new();
    for name in candidates
        .iter()
        .filter_map(|candidate| candidate.field_name.as_ref())
    {
        *counts.entry(name.clone()).or_default() += 1;
    }
    for candidate in &mut candidates {
        if candidate
            .field_name
            .as_ref()
            .is_some_and(|name| counts[name] > 1)
        {
            candidate
                .violations
                .push(SlotDeclarationViolation::DuplicateSlot);
        }
    }
    candidates.sort_by(|left, right| left.id.cmp(&right.id));
    candidates
}

#[allow(clippy::too_many_arguments)]
fn retain_invalid_slot_candidate(
    candidates: &mut Vec<AuthoredSlotDeclarationCandidate>,
    component: &SemanticId,
    authored_declaration: &SemanticId,
    declaration_kind: AuthoredDeclarationKind,
    field_name: Option<&str>,
    decorators: &[presolve_parser::ParsedDecorator],
    path: &Path,
    span: SourceSpan,
) {
    for decorator in decorators
        .iter()
        .filter(|decorator| decorator.name == "slot")
    {
        let mut violations = vec![SlotDeclarationViolation::InvalidDeclarationKind {
            actual: declaration_kind,
        }];
        if decorator.argument_count != 0 {
            violations.push(SlotDeclarationViolation::DecoratorArity {
                actual: decorator.argument_count,
                expected: 0,
            });
        }
        candidates.push(AuthoredSlotDeclarationCandidate {
            id: SlotDeclarationCandidateId::for_component_position(component, decorator.span.start),
            owner_component: component.clone(),
            authored_declaration: authored_declaration.clone(),
            declaration_kind,
            field_name: field_name.map(str::to_string),
            declared_type: None,
            decorator_argument_count: decorator.argument_count,
            decorator_provenance: SourceProvenance::new(path, decorator.span),
            name_provenance: field_name.map(|_| SourceProvenance::new(path, span)),
            provenance: SourceProvenance::new(path, span),
            static_modifier_provenance: None,
            initializer_provenance: None,
            violations,
        });
    }
}

fn slot_declarations_from_candidates(
    candidates: &[AuthoredSlotDeclarationCandidate],
) -> Vec<SlotDeclaration> {
    candidates
        .iter()
        .filter(|candidate| candidate.violations.is_empty())
        .map(|candidate| {
            let name = candidate
                .field_name
                .clone()
                .expect("valid Slot candidates are fields");
            SlotDeclaration {
                authored_field: candidate.authored_declaration.clone(),
                kind: if name == "children" {
                    SlotKind::Default
                } else {
                    SlotKind::Named
                },
                name,
                declared_type: candidate
                    .declared_type
                    .clone()
                    .expect("valid Slot candidates have a declared type"),
                decorator_provenance: candidate.decorator_provenance.clone(),
                name_provenance: candidate
                    .name_provenance
                    .clone()
                    .expect("valid Slot candidates retain field-name provenance"),
                provenance: candidate.provenance.clone(),
            }
        })
        .collect()
}

fn component_method_from_parsed(
    method: &ParsedMethod,
    path: &Path,
    component_id: &SemanticId,
) -> ComponentMethod {
    let id = component_id.method(&method.name);
    let semantic_role = method_semantic_role(method);
    let computed_expression = match semantic_role {
        MethodSemanticRole::Computed => method
            .computed_expression
            .as_ref()
            .map(computed_expression_from_parsed),
        MethodSemanticRole::Standard | MethodSemanticRole::Effect | MethodSemanticRole::Action => {
            None
        }
    };
    ComponentMethod {
        id: id.clone(),
        owner: SemanticOwner::entity(component_id.clone()),
        name: method.name.clone(),
        is_getter: method.is_getter,
        is_async: method.is_async,
        is_static: method.is_static,
        decorators: method
            .decorators
            .iter()
            .map(|decorator| decorator.name.clone())
            .collect(),
        semantic_role,
        local_variables: method
            .local_variables
            .iter()
            .enumerate()
            .map(|(index, local)| MethodLocalVariable {
                id: id.local_variable(&local.name, index),
                owner: SemanticOwner::entity(id.clone()),
                name: local.name.clone(),
                value: serializable_value_from_parsed(&local.value),
                span: local.span,
            })
            .collect(),
        parameters: method
            .parameters
            .iter()
            .enumerate()
            .map(|(index, parameter)| MethodParameter {
                id: id.parameter(&parameter.name, index),
                owner: SemanticOwner::entity(id.clone()),
                name: parameter.name.clone(),
                span: parameter.span,
                declared_type: parameter.type_annotation.as_ref().map(|annotation| {
                    DeclaredStateType {
                        kind: declared_state_type_kind(&annotation.text),
                        text: annotation.text.clone(),
                        provenance: SourceProvenance::new(path, annotation.span),
                    }
                }),
            })
            .collect(),
        declared_return_type: method.return_type_annotation.as_ref().map(|annotation| {
            DeclaredStateType {
                kind: declared_state_type_kind(&annotation.text),
                text: annotation.text.clone(),
                provenance: SourceProvenance::new(path, annotation.span),
            }
        }),
        return_values: method
            .return_values
            .iter()
            .map(serializable_value_from_parsed)
            .collect(),
        computed_expression,
        effect_body: method.effect_body.as_ref().map(effect_body_from_parsed),
        calls: method.calls.iter().map(method_call_from_parsed).collect(),
    }
}

fn method_call_from_parsed(call: &ParsedMethodCall) -> MethodCall {
    MethodCall {
        callee: call.callee.clone(),
        span: call.span,
    }
}

fn method_semantic_role(method: &ParsedMethod) -> MethodSemanticRole {
    if method.is_getter
        && method
            .decorators
            .iter()
            .any(|decorator| decorator.name == "computed")
    {
        MethodSemanticRole::Computed
    } else if method
        .decorators
        .iter()
        .any(|decorator| decorator.name == "effect")
    {
        MethodSemanticRole::Effect
    } else if method
        .decorators
        .iter()
        .any(|decorator| decorator.name == "action")
    {
        MethodSemanticRole::Action
    } else {
        MethodSemanticRole::Standard
    }
}

fn state_fields_from_class(class: &ParsedClass, path: &Path, id: &SemanticId) -> Vec<StateField> {
    class
        .properties
        .iter()
        .filter(|property| {
            property.initializer.as_deref() == Some("state(...)")
                && !property.decorators.iter().any(|decorator| {
                    matches!(
                        decorator.name.as_str(),
                        "context" | "provide" | "consume" | "slot" | "form" | "field"
                    )
                })
        })
        .map(|property| {
            let initial_expression = property
                .state_initial_expression
                .as_ref()
                .map(constant_expression_from_parsed);
            let initial_value = property
                .state_initial_value
                .as_ref()
                .map(serializable_value_from_parsed);

            StateField {
                id: id.state_field(&property.name),
                owner: SemanticOwner::entity(id.clone()),
                name: property.name.clone(),
                initial_value,
                initial_expression,
                declared_type: property.state_type_annotation.as_ref().map(|annotation| {
                    DeclaredStateType {
                        text: annotation.text.clone(),
                        provenance: SourceProvenance::new(path, annotation.span),
                        kind: declared_state_type_kind(&annotation.text),
                    }
                }),
            }
        })
        .collect()
}

fn context_declarations_from_class(
    class: &ParsedClass,
    path: &Path,
    id: &SemanticId,
) -> Vec<ContextDeclaration> {
    class
        .properties
        .iter()
        .filter_map(|property| {
            let decorator = property
                .decorators
                .iter()
                .find(|decorator| decorator.name == "context")?;
            let declared_type = property.type_annotation.as_ref()?;

            (decorator.argument_count == 0
                && !property.decorators.iter().any(|decorator| {
                    matches!(
                        decorator.name.as_str(),
                        "provide" | "consume" | "slot" | "form" | "field"
                    )
                })
                && property
                    .initializer
                    .as_ref()
                    .is_none_or(|_| property.initializer_literal.is_some()))
            .then(|| ContextDeclaration {
                authored_field: id.context_field(&property.name),
                name: property.name.clone(),
                declared_type: DeclaredStateType {
                    kind: declared_state_type_kind(&declared_type.text),
                    text: declared_type.text.clone(),
                    provenance: SourceProvenance::new(path, declared_type.span),
                },
                default_expression: property.initializer_literal.as_ref().map(|value| {
                    ConstantExpression {
                        kind: ConstantExpressionKind::Literal(serializable_value_from_parsed(
                            value,
                        )),
                        span: property
                            .initializer_span
                            .expect("literal context defaults should retain a source span"),
                    }
                }),
                decorator_provenance: SourceProvenance::new(path, decorator.span),
                name_provenance: SourceProvenance::new(path, property.name_span),
                provenance: SourceProvenance::new(path, property.span),
            })
        })
        .collect()
}

fn provider_declarations_from_class(
    class: &ParsedClass,
    path: &Path,
    id: &SemanticId,
) -> Vec<ProviderDeclaration> {
    class
        .properties
        .iter()
        .filter_map(|property| {
            let decorator = property
                .decorators
                .iter()
                .find(|decorator| decorator.name == "provide")?;
            let designator = context_designator_from_decorator(decorator, path)?;
            let declared_type = property.type_annotation.as_ref()?;
            let value_expression = property.initializer_expression.as_ref()?;

            (decorator.argument_count == 1
                && !property.is_static
                && !property.decorators.iter().any(|decorator| {
                    matches!(
                        decorator.name.as_str(),
                        "context" | "consume" | "slot" | "form" | "field"
                    )
                }))
            .then(|| ProviderDeclaration {
                authored_field: id.provider_field(&property.name),
                name: property.name.clone(),
                context_designator: designator,
                declared_type: DeclaredStateType {
                    kind: declared_state_type_kind(&declared_type.text),
                    text: declared_type.text.clone(),
                    provenance: SourceProvenance::new(path, declared_type.span),
                },
                value_expression: computed_expression_from_parsed(value_expression),
                decorator_provenance: SourceProvenance::new(path, decorator.span),
                name_provenance: SourceProvenance::new(path, property.name_span),
                provenance: SourceProvenance::new(path, property.span),
            })
        })
        .collect()
}

fn consumer_declarations_from_class(
    class: &ParsedClass,
    path: &Path,
    id: &SemanticId,
) -> Vec<ConsumerDeclaration> {
    class
        .properties
        .iter()
        .filter_map(|property| {
            let decorator = property
                .decorators
                .iter()
                .find(|decorator| decorator.name == "consume")?;
            let designator = context_designator_from_decorator(decorator, path)?;
            let requested_type = property.type_annotation.as_ref()?;
            let has_conflicting_decorator = property.decorators.iter().any(|decorator| {
                matches!(
                    decorator.name.as_str(),
                    "state"
                        | "context"
                        | "provide"
                        | "computed"
                        | "effect"
                        | "action"
                        | "resource"
                        | "slot"
                        | "form"
                        | "field"
                )
            });

            (decorator.argument_count == 1
                && !property.is_static
                && property.is_definite_assignment
                && property.initializer.is_none()
                && !has_conflicting_decorator)
                .then(|| ConsumerDeclaration {
                    authored_field: id.consumer_field(&property.name),
                    name: property.name.clone(),
                    context_designator: designator,
                    requested_type: DeclaredStateType {
                        kind: declared_state_type_kind(&requested_type.text),
                        text: requested_type.text.clone(),
                        provenance: SourceProvenance::new(path, requested_type.span),
                    },
                    decorator_provenance: SourceProvenance::new(path, decorator.span),
                    name_provenance: SourceProvenance::new(path, property.name_span),
                    provenance: SourceProvenance::new(path, property.span),
                })
        })
        .collect()
}

fn context_designator_from_parsed(
    designator: &ParsedStaticMemberDesignator,
    path: &Path,
) -> ContextDesignator {
    ContextDesignator {
        component_symbol: designator.object.clone(),
        context_member: designator.member.clone(),
        provenance: SourceProvenance::new(path, designator.span),
        component_provenance: SourceProvenance::new(path, designator.object_span),
        member_provenance: SourceProvenance::new(path, designator.member_span),
    }
}

fn context_designator_from_decorator(
    decorator: &ParsedDecorator,
    path: &Path,
) -> Option<ContextDesignator> {
    decorator
        .static_member_argument
        .as_ref()
        .map(|value| context_designator_from_parsed(value, path))
        .or_else(|| {
            let value = decorator.argument.as_deref()?;
            let (component_symbol, context_member) = value.split_once('.')?;
            if component_symbol.is_empty()
                || context_member.is_empty()
                || context_member.contains('.')
                || !context_designator_segment(component_symbol)
                || !context_designator_segment(context_member)
            {
                return None;
            }
            let provenance = SourceProvenance::new(path, decorator.argument_spans.first()?.clone());
            Some(ContextDesignator {
                component_symbol: component_symbol.to_string(),
                context_member: context_member.to_string(),
                provenance: provenance.clone(),
                component_provenance: provenance.clone(),
                member_provenance: provenance,
            })
        })
}

fn context_designator_segment(value: &str) -> bool {
    let mut characters = value.chars();
    matches!(characters.next(), Some(character) if character == '_' || character == '$' || character.is_ascii_alphabetic())
        && characters.all(|character| {
            character == '_' || character == '$' || character.is_ascii_alphanumeric()
        })
}

fn arithmetic_expression_from_parsed(
    expression: &ParsedArithmeticExpression,
) -> ArithmeticExpression {
    let kind = match &expression.kind {
        ParsedArithmeticExpressionKind::Number(value) => {
            ArithmeticExpressionKind::Number(value.clone())
        }
        ParsedArithmeticExpressionKind::Binary {
            operator,
            left,
            right,
        } => ArithmeticExpressionKind::Binary {
            operator: arithmetic_operator_from_parsed(*operator),
            left: Box::new(arithmetic_expression_from_parsed(left)),
            right: Box::new(arithmetic_expression_from_parsed(right)),
        },
    };

    ArithmeticExpression {
        kind,
        span: expression.span,
    }
}

fn constant_expression_from_parsed(expression: &ParsedConstantExpression) -> ConstantExpression {
    let kind = match &expression.kind {
        ParsedConstantExpressionKind::Primitive(value) => {
            ConstantExpressionKind::Literal(serializable_value_from_parsed(value))
        }
        ParsedConstantExpressionKind::Boolean(value) => ConstantExpressionKind::Boolean(*value),
        ParsedConstantExpressionKind::Arithmetic(expression) => {
            ConstantExpressionKind::Arithmetic(arithmetic_expression_from_parsed(expression))
        }
        ParsedConstantExpressionKind::Comparison {
            operator,
            left,
            right,
        } => ConstantExpressionKind::Comparison {
            operator: comparison_operator_from_parsed(*operator),
            left: arithmetic_expression_from_parsed(left),
            right: arithmetic_expression_from_parsed(right),
        },
        ParsedConstantExpressionKind::Logical {
            operator,
            left,
            right,
        } => ConstantExpressionKind::Logical {
            operator: logical_operator_from_parsed(*operator),
            left: Box::new(constant_expression_from_parsed(left)),
            right: Box::new(constant_expression_from_parsed(right)),
        },
        ParsedConstantExpressionKind::NullishCoalescing { left, right } => {
            ConstantExpressionKind::NullishCoalescing {
                left: Box::new(constant_expression_from_parsed(left)),
                right: Box::new(constant_expression_from_parsed(right)),
            }
        }
        ParsedConstantExpressionKind::Unary { operator, operand } => {
            ConstantExpressionKind::Unary {
                operator: match operator {
                    ParsedUnaryOperator::Not => UnaryOperator::Not,
                    ParsedUnaryOperator::Plus => UnaryOperator::Plus,
                    ParsedUnaryOperator::Minus => UnaryOperator::Minus,
                },
                operand: Box::new(constant_expression_from_parsed(operand)),
            }
        }
    };

    ConstantExpression {
        kind,
        span: expression.span,
    }
}

fn computed_expression_from_parsed(expression: &ParsedComputedExpression) -> ComputedExpression {
    let kind = match &expression.kind {
        ParsedComputedExpressionKind::Literal(value) => {
            ComputedExpressionKind::Literal(serializable_value_from_parsed(value))
        }
        ParsedComputedExpressionKind::ThisMember(name) => {
            ComputedExpressionKind::ThisMember(name.clone())
        }
        ParsedComputedExpressionKind::MemberAccess {
            object,
            property,
            optional,
        } => ComputedExpressionKind::MemberAccess {
            object: Box::new(computed_expression_from_parsed(object)),
            property: property.clone(),
            optional: *optional,
        },
        ParsedComputedExpressionKind::IndexAccess { object, index } => {
            ComputedExpressionKind::IndexAccess {
                object: Box::new(computed_expression_from_parsed(object)),
                index: Box::new(computed_expression_from_parsed(index)),
            }
        }
        ParsedComputedExpressionKind::Conditional {
            condition,
            when_true,
            when_false,
        } => ComputedExpressionKind::Conditional {
            condition: Box::new(computed_expression_from_parsed(condition)),
            when_true: Box::new(computed_expression_from_parsed(when_true)),
            when_false: Box::new(computed_expression_from_parsed(when_false)),
        },
        ParsedComputedExpressionKind::Template {
            quasis,
            expressions,
        } => ComputedExpressionKind::Template {
            quasis: quasis.clone(),
            expressions: expressions
                .iter()
                .map(computed_expression_from_parsed)
                .collect(),
        },
        ParsedComputedExpressionKind::Call { callee, arguments } => ComputedExpressionKind::Call {
            callee: callee.clone(),
            arguments: arguments
                .iter()
                .map(computed_expression_from_parsed)
                .collect(),
        },
        ParsedComputedExpressionKind::Arithmetic {
            left,
            right,
            operator,
        } => ComputedExpressionKind::Arithmetic {
            left: Box::new(computed_expression_from_parsed(left)),
            right: Box::new(computed_expression_from_parsed(right)),
            operator: arithmetic_operator_from_parsed(*operator),
        },
        ParsedComputedExpressionKind::Comparison {
            left,
            right,
            operator,
        } => ComputedExpressionKind::Comparison {
            left: Box::new(computed_expression_from_parsed(left)),
            right: Box::new(computed_expression_from_parsed(right)),
            operator: comparison_operator_from_parsed(*operator),
        },
        ParsedComputedExpressionKind::Logical {
            left,
            right,
            operator,
        } => ComputedExpressionKind::Logical {
            left: Box::new(computed_expression_from_parsed(left)),
            right: Box::new(computed_expression_from_parsed(right)),
            operator: logical_operator_from_parsed(*operator),
        },
        ParsedComputedExpressionKind::NullishCoalescing { left, right } => {
            ComputedExpressionKind::NullishCoalescing {
                left: Box::new(computed_expression_from_parsed(left)),
                right: Box::new(computed_expression_from_parsed(right)),
            }
        }
        ParsedComputedExpressionKind::Unary { operand, operator } => {
            ComputedExpressionKind::Unary {
                operand: Box::new(computed_expression_from_parsed(operand)),
                operator: match operator {
                    ParsedUnaryOperator::Not => UnaryOperator::Not,
                    ParsedUnaryOperator::Plus => UnaryOperator::Plus,
                    ParsedUnaryOperator::Minus => UnaryOperator::Minus,
                },
            }
        }
    };

    ComputedExpression {
        kind,
        span: expression.span,
    }
}

fn effect_body_from_parsed(body: &ParsedEffectBody) -> EffectBodySyntax {
    EffectBodySyntax {
        statements: body
            .statements
            .iter()
            .map(|statement| EffectStatementSyntax {
                kind: effect_statement_syntax_kind_from_parsed(&statement.kind),
                span: statement.span,
            })
            .collect(),
    }
}

fn effect_statement_syntax_kind_from_parsed(
    kind: &ParsedEffectStatementKind,
) -> EffectStatementSyntaxKind {
    match kind {
        ParsedEffectStatementKind::StaticMemberAssignment { target, value } => {
            EffectStatementSyntaxKind::StaticMemberAssignment {
                target: effect_expression_from_parsed(target),
                value: effect_expression_from_parsed(value),
            }
        }
        ParsedEffectStatementKind::CapabilityCall { callee, arguments } => {
            EffectStatementSyntaxKind::CapabilityCall {
                callee: effect_expression_from_parsed(callee),
                arguments: arguments
                    .iter()
                    .map(effect_expression_from_parsed)
                    .collect(),
            }
        }
        ParsedEffectStatementKind::EffectReturn { value } => {
            EffectStatementSyntaxKind::EffectReturn {
                value: value.as_ref().map(effect_expression_from_parsed),
            }
        }
        ParsedEffectStatementKind::Empty => EffectStatementSyntaxKind::Empty,
        ParsedEffectStatementKind::Unsupported(kind) => EffectStatementSyntaxKind::Unsupported(
            unsupported_effect_statement_kind_from_parsed(*kind),
        ),
    }
}

fn unsupported_effect_statement_kind_from_parsed(
    kind: ParsedUnsupportedEffectStatementKind,
) -> UnsupportedEffectStatementKind {
    match kind {
        ParsedUnsupportedEffectStatementKind::LocalDeclaration => {
            UnsupportedEffectStatementKind::LocalDeclaration
        }
        ParsedUnsupportedEffectStatementKind::Branch => UnsupportedEffectStatementKind::Branch,
        ParsedUnsupportedEffectStatementKind::Loop => UnsupportedEffectStatementKind::Loop,
        ParsedUnsupportedEffectStatementKind::NestedBlock => {
            UnsupportedEffectStatementKind::NestedBlock
        }
        ParsedUnsupportedEffectStatementKind::ExceptionHandling => {
            UnsupportedEffectStatementKind::ExceptionHandling
        }
        ParsedUnsupportedEffectStatementKind::AsyncOperation => {
            UnsupportedEffectStatementKind::AsyncOperation
        }
        ParsedUnsupportedEffectStatementKind::CompoundAssignment => {
            UnsupportedEffectStatementKind::CompoundAssignment
        }
        ParsedUnsupportedEffectStatementKind::CleanupReturnCandidate => {
            UnsupportedEffectStatementKind::CleanupReturnCandidate
        }
        ParsedUnsupportedEffectStatementKind::UnsupportedExpression => {
            UnsupportedEffectStatementKind::UnsupportedExpression
        }
    }
}

fn effect_expression_from_parsed(expression: &ParsedEffectExpression) -> EffectExpression {
    let kind = match &expression.kind {
        ParsedEffectExpressionKind::Literal(value) => {
            EffectExpressionKind::Literal(serializable_value_from_parsed(value))
        }
        ParsedEffectExpressionKind::Identifier(name) => {
            EffectExpressionKind::Identifier(name.clone())
        }
        ParsedEffectExpressionKind::ThisMember(name) => {
            EffectExpressionKind::ThisMember(name.clone())
        }
        ParsedEffectExpressionKind::MemberAccess { object, property } => {
            EffectExpressionKind::MemberAccess {
                object: Box::new(effect_expression_from_parsed(object)),
                property: property.clone(),
            }
        }
        ParsedEffectExpressionKind::Arithmetic {
            left,
            right,
            operator,
        } => EffectExpressionKind::Arithmetic {
            left: Box::new(effect_expression_from_parsed(left)),
            right: Box::new(effect_expression_from_parsed(right)),
            operator: arithmetic_operator_from_parsed(*operator),
        },
        ParsedEffectExpressionKind::Comparison {
            left,
            right,
            operator,
        } => EffectExpressionKind::Comparison {
            left: Box::new(effect_expression_from_parsed(left)),
            right: Box::new(effect_expression_from_parsed(right)),
            operator: comparison_operator_from_parsed(*operator),
        },
        ParsedEffectExpressionKind::Logical {
            left,
            right,
            operator,
        } => EffectExpressionKind::Logical {
            left: Box::new(effect_expression_from_parsed(left)),
            right: Box::new(effect_expression_from_parsed(right)),
            operator: logical_operator_from_parsed(*operator),
        },
        ParsedEffectExpressionKind::NullishCoalescing { left, right } => {
            EffectExpressionKind::NullishCoalescing {
                left: Box::new(effect_expression_from_parsed(left)),
                right: Box::new(effect_expression_from_parsed(right)),
            }
        }
        ParsedEffectExpressionKind::Unary { operand, operator } => EffectExpressionKind::Unary {
            operand: Box::new(effect_expression_from_parsed(operand)),
            operator: match operator {
                ParsedUnaryOperator::Not => UnaryOperator::Not,
                ParsedUnaryOperator::Plus => UnaryOperator::Plus,
                ParsedUnaryOperator::Minus => UnaryOperator::Minus,
            },
        },
    };
    EffectExpression {
        kind,
        span: expression.span,
    }
}

fn arithmetic_operator_from_parsed(operator: ParsedArithmeticOperator) -> ArithmeticOperator {
    match operator {
        ParsedArithmeticOperator::Add => ArithmeticOperator::Add,
        ParsedArithmeticOperator::Subtract => ArithmeticOperator::Subtract,
        ParsedArithmeticOperator::Multiply => ArithmeticOperator::Multiply,
        ParsedArithmeticOperator::Divide => ArithmeticOperator::Divide,
        ParsedArithmeticOperator::Remainder => ArithmeticOperator::Remainder,
    }
}

fn comparison_operator_from_parsed(operator: ParsedComparisonOperator) -> ComparisonOperator {
    match operator {
        ParsedComparisonOperator::Equal => ComparisonOperator::Equal,
        ParsedComparisonOperator::NotEqual => ComparisonOperator::NotEqual,
        ParsedComparisonOperator::LessThan => ComparisonOperator::LessThan,
        ParsedComparisonOperator::LessThanOrEqual => ComparisonOperator::LessThanOrEqual,
        ParsedComparisonOperator::GreaterThan => ComparisonOperator::GreaterThan,
        ParsedComparisonOperator::GreaterThanOrEqual => ComparisonOperator::GreaterThanOrEqual,
    }
}

fn logical_operator_from_parsed(operator: ParsedLogicalOperator) -> LogicalOperator {
    match operator {
        ParsedLogicalOperator::And => LogicalOperator::And,
        ParsedLogicalOperator::Or => LogicalOperator::Or,
    }
}

fn declared_state_type_kind(text: &str) -> Option<DeclaredStateTypeKind> {
    match text {
        "string" => Some(DeclaredStateTypeKind::String),
        "number" => Some(DeclaredStateTypeKind::Number),
        "boolean" => Some(DeclaredStateTypeKind::Boolean),
        "null" => Some(DeclaredStateTypeKind::Null),
        _ => None,
    }
}

fn render_model_from_parsed_method(
    method: &presolve_parser::ParsedMethod,
    component_id: &SemanticId,
) -> RenderModel {
    let root = method.jsx_roots.first();
    let root_element = root.and_then(parsed_root_element);
    let root_fragment = root.and_then(parsed_root_fragment);
    let mut event_ids = EventIdAllocator::default();

    RenderModel {
        root_element: root_element.map(|element| element.name.clone()),
        root_element_name_span: root_element.map(|element| element.name_span),
        root_span: root_element.map(|element| element.span),
        root_fragment: root_fragment
            .map(|fragment| render_fragment_from_parsed(fragment, component_id, &mut event_ids)),
        attributes: root_element.map_or_else(Vec::new, |element| {
            element
                .attributes
                .iter()
                .map(render_attribute_from_parsed)
                .collect()
        }),
        event_handlers: root_element.map_or_else(Vec::new, |element| {
            element
                .event_handlers
                .iter()
                .map(|handler| {
                    render_event_handler_from_parsed(handler, component_id, &mut event_ids)
                })
                .collect()
        }),
        children: root_element.map_or_else(Vec::new, |element| {
            element
                .children
                .iter()
                .map(|child| render_child_from_parsed(child, component_id, &mut event_ids))
                .collect()
        }),
        bindings: method.bindings.clone(),
    }
}

fn parsed_root_element(root: &ParsedJsxNode) -> Option<&presolve_parser::ParsedJsxElement> {
    match root {
        ParsedJsxNode::Element(element) => Some(element),
        ParsedJsxNode::Fragment(_) => None,
    }
}

fn parsed_root_fragment(root: &ParsedJsxNode) -> Option<&ParsedJsxFragment> {
    match root {
        ParsedJsxNode::Element(_) => None,
        ParsedJsxNode::Fragment(fragment) => Some(fragment),
    }
}

fn collect_render_binding_diagnostics(
    class: &ParsedClass,
    render: &RenderModel,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    let property_names = class
        .properties
        .iter()
        .map(|property| property.name.as_str())
        .collect::<Vec<_>>();

    for binding in &render.bindings {
        if let Some(name) = this_member_name(binding) {
            if !property_names.contains(&name) {
                diagnostics.push(ComponentDiagnostic {
            severity: ComponentDiagnosticSeverity::Error,
            effect_id: None,
            statement_id: None,
            context_declaration_candidate_id: None,
            context_id: None,
            provider_id: None,
            consumer_id: None,
            slot_id: None,
            invocation_id: None,
            component_instance_id: None,
            slot_binding_id: None,
            structural_region_id: None,
            component_id: None,
            provider_instance_id: None,
            consumer_instance_id: None,
            secondary_labels: Vec::new(),
                    provenance: None,
                    code: "PSC1003".to_string(),
                    message: format!(
                        "render binding `{binding}` references unknown field `{name}` in class `{}`",
                        class.name
                    ),
                });
            }
        }
    }
}

fn collect_render_event_diagnostics(
    class: &ParsedClass,
    render: &RenderModel,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    let method_names = class
        .methods
        .iter()
        .map(|method| method.name.as_str())
        .collect::<Vec<_>>();

    for event_handler in render_event_handlers(render) {
        if event_handler.event != "click" {
            diagnostics.push(ComponentDiagnostic {
                severity: ComponentDiagnosticSeverity::Error,
                effect_id: None,
                statement_id: None,
                context_declaration_candidate_id: None,
                context_id: None,
                provider_id: None,
                consumer_id: None,
                slot_id: None,
                invocation_id: None,
                component_instance_id: None,
                slot_binding_id: None,
                structural_region_id: None,
                component_id: None,
                provider_instance_id: None,
                consumer_instance_id: None,
                secondary_labels: Vec::new(),
                provenance: None,
                code: "PSC1005".to_string(),
                message: format!(
                    "event `{}` is not supported yet in class `{}`",
                    event_handler.event, class.name
                ),
            });
        }

        if let Some(name) = this_member_name(&event_handler.handler) {
            if !method_names.contains(&name) {
                diagnostics.push(ComponentDiagnostic {
                    severity: ComponentDiagnosticSeverity::Error,
                    effect_id: None,
                    statement_id: None,
                    context_declaration_candidate_id: None,
                    context_id: None,
                    provider_id: None,
                    consumer_id: None,
                    slot_id: None,
                    invocation_id: None,
                    component_instance_id: None,
                    slot_binding_id: None,
                    structural_region_id: None,
                    component_id: None,
                    provider_instance_id: None,
                    consumer_instance_id: None,
                    secondary_labels: Vec::new(),
                    provenance: None,
                    code: "PSC1004".to_string(),
                    message: format!(
                        "event handler `{}` references unknown method `{name}` in class `{}`",
                        event_handler.handler, class.name
                    ),
                });
            }
        }
    }
}

fn state_operation_from_parsed(operation: &ParsedStateOperation) -> StateOperation {
    match operation {
        ParsedStateOperation::Increment => StateOperation::Increment,
        ParsedStateOperation::Decrement => StateOperation::Decrement,
        ParsedStateOperation::AddAssign(value) => {
            StateOperation::AddAssign(serializable_value_from_parsed(value))
        }
        ParsedStateOperation::SubtractAssign(value) => {
            StateOperation::SubtractAssign(serializable_value_from_parsed(value))
        }
        ParsedStateOperation::Assign(value) => {
            StateOperation::Assign(serializable_value_from_parsed(value))
        }
        ParsedStateOperation::Toggle => StateOperation::Toggle,
    }
}

fn serializable_value_from_parsed(value: &ParsedSerializableValue) -> SerializableValue {
    match value {
        ParsedSerializableValue::Null => SerializableValue::Null,
        ParsedSerializableValue::Number(value) => SerializableValue::Number(value.clone()),
        ParsedSerializableValue::String(value) => SerializableValue::String(value.clone()),
        ParsedSerializableValue::Boolean(value) => SerializableValue::Boolean(*value),
        ParsedSerializableValue::Array(values) => {
            SerializableValue::Array(values.iter().map(serializable_value_from_parsed).collect())
        }
        ParsedSerializableValue::Object(values) => SerializableValue::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), serializable_value_from_parsed(value)))
                .collect(),
        ),
    }
}

fn decorator_argument(class: &ParsedClass, name: &str) -> Option<String> {
    class
        .decorators
        .iter()
        .find(|decorator| decorator.name == name)
        .and_then(|decorator| decorator.argument.clone())
}

fn render_child_from_parsed(
    child: &ParsedJsxChild,
    component_id: &SemanticId,
    event_ids: &mut EventIdAllocator,
) -> RenderChild {
    match child {
        ParsedJsxChild::Text { value, span } => RenderChild::Text {
            value: value.clone(),
            span: *span,
        },
        ParsedJsxChild::Binding { expression, span } => RenderChild::Binding {
            expression: expression.clone(),
            span: *span,
        },
        ParsedJsxChild::Element(element) => RenderChild::Element(RenderElement {
            tag_name: element.name.clone(),
            tag_name_span: element.name_span,
            span: element.span,
            attributes: element
                .attributes
                .iter()
                .map(render_attribute_from_parsed)
                .collect(),
            event_handlers: element
                .event_handlers
                .iter()
                .map(|handler| render_event_handler_from_parsed(handler, component_id, event_ids))
                .collect(),
            children: element
                .children
                .iter()
                .map(|child| render_child_from_parsed(child, component_id, event_ids))
                .collect::<Vec<_>>(),
        }),
        ParsedJsxChild::Fragment(fragment) => RenderChild::Fragment(render_fragment_from_parsed(
            fragment,
            component_id,
            event_ids,
        )),
        ParsedJsxChild::Conditional(conditional) => RenderChild::Conditional(
            render_conditional_from_parsed(conditional, component_id, event_ids),
        ),
        ParsedJsxChild::List(list) => {
            RenderChild::List(render_list_from_parsed(list, component_id, event_ids))
        }
    }
}

fn render_list_from_parsed(
    list: &ParsedJsxList,
    component_id: &SemanticId,
    event_ids: &mut EventIdAllocator,
) -> RenderList {
    RenderList {
        iterable: list.iterable.clone(),
        item_variable: list.item_variable.clone(),
        index_variable: list.index_variable.clone(),
        key_expression: list.key_expression.clone(),
        span: list.span,
        item_template: render_children_from_parsed_node(
            &list.item_template,
            component_id,
            event_ids,
        ),
    }
}

fn render_conditional_from_parsed(
    conditional: &ParsedJsxConditional,
    component_id: &SemanticId,
    event_ids: &mut EventIdAllocator,
) -> RenderConditional {
    RenderConditional {
        condition: conditional.condition.clone(),
        span: conditional.span,
        when_true: render_children_from_parsed_node(
            &conditional.when_true,
            component_id,
            event_ids,
        ),
        when_false: conditional
            .when_false
            .as_ref()
            .map(|node| render_children_from_parsed_node(node, component_id, event_ids))
            .unwrap_or_default(),
    }
}

fn render_children_from_parsed_node(
    node: &ParsedJsxNode,
    component_id: &SemanticId,
    event_ids: &mut EventIdAllocator,
) -> Vec<RenderChild> {
    match node {
        ParsedJsxNode::Element(element) => vec![RenderChild::Element(RenderElement {
            tag_name: element.name.clone(),
            tag_name_span: element.name_span,
            span: element.span,
            attributes: element
                .attributes
                .iter()
                .map(render_attribute_from_parsed)
                .collect(),
            event_handlers: element
                .event_handlers
                .iter()
                .map(|handler| render_event_handler_from_parsed(handler, component_id, event_ids))
                .collect(),
            children: element
                .children
                .iter()
                .map(|child| render_child_from_parsed(child, component_id, event_ids))
                .collect::<Vec<_>>(),
        })],
        ParsedJsxNode::Fragment(fragment) => fragment
            .children
            .iter()
            .map(|child| render_child_from_parsed(child, component_id, event_ids))
            .collect(),
    }
}

fn render_fragment_from_parsed(
    fragment: &ParsedJsxFragment,
    component_id: &SemanticId,
    event_ids: &mut EventIdAllocator,
) -> RenderFragment {
    RenderFragment {
        span: fragment.span,
        children: fragment
            .children
            .iter()
            .map(|child| render_child_from_parsed(child, component_id, event_ids))
            .collect(),
    }
}

fn render_attribute_from_parsed(attribute: &ParsedJsxAttribute) -> RenderAttribute {
    RenderAttribute {
        name: attribute.name.clone(),
        value: match &attribute.value {
            ParsedJsxAttributeValue::Boolean => RenderAttributeValue::Boolean,
            ParsedJsxAttributeValue::Static(value) => RenderAttributeValue::Static(value.clone()),
            ParsedJsxAttributeValue::Expression(expression) => {
                RenderAttributeValue::Expression(expression.clone())
            }
            ParsedJsxAttributeValue::Spread(expression) => {
                RenderAttributeValue::Spread(expression.clone())
            }
            ParsedJsxAttributeValue::Unsupported => RenderAttributeValue::Unsupported,
        },
        name_span: attribute.name_span,
        value_span: attribute.value_span,
        expression_span: attribute.expression_span,
        this_member: attribute.this_member.clone(),
        constant_value: attribute
            .constant_value
            .as_ref()
            .map(serializable_value_from_parsed),
        span: attribute.span,
    }
}

fn render_event_handler_from_parsed(
    event_handler: &ParsedEventHandler,
    component_id: &SemanticId,
    event_ids: &mut EventIdAllocator,
) -> RenderEventHandler {
    RenderEventHandler {
        id: component_id.event_handler(&event_handler.event, event_ids.next()),
        owner: SemanticOwner::entity(component_id.template()),
        event: event_handler.event.clone(),
        handler: event_handler.handler.clone(),
        span: event_handler.span,
    }
}

#[derive(Debug, Default)]
struct EventIdAllocator {
    next: usize,
}

impl EventIdAllocator {
    fn next(&mut self) -> usize {
        let current = self.next;
        self.next += 1;
        current
    }
}

fn collect_component_provenance(
    class: &ParsedClass,
    component: &ComponentNode,
    path: &Path,
) -> BTreeMap<SemanticId, SourceProvenance> {
    let mut provenance = BTreeMap::new();
    provenance.insert(
        component.id.clone(),
        SourceProvenance::new(path, class.span),
    );

    for property in &class.properties {
        if property.initializer.as_deref() != Some("state(...)") {
            continue;
        }

        if let Some(field) = component
            .state_fields
            .iter()
            .find(|field| field.name == property.name)
        {
            provenance.insert(field.id.clone(), SourceProvenance::new(path, property.span));
        }
    }

    for method in &class.methods {
        if let Some(component_method) = component
            .methods
            .iter()
            .find(|component_method| component_method.name == method.name)
        {
            provenance.insert(
                component_method.id.clone(),
                SourceProvenance::new(path, method.span),
            );
        }

        if method.name == "render" {
            provenance.insert(
                component.id.template(),
                SourceProvenance::new(path, method.span),
            );
        }

        for (index, update) in method.state_updates.iter().enumerate() {
            provenance.insert(
                component.id.action(&method.name, index),
                SourceProvenance::new(path, update.span),
            );
        }

        if let Some(component_method) = component
            .methods
            .iter()
            .find(|component_method| component_method.name == method.name)
        {
            for parameter in &component_method.parameters {
                provenance.insert(
                    parameter.id.clone(),
                    SourceProvenance::new(path, parameter.span),
                );
            }
            for local in &component_method.local_variables {
                provenance.insert(local.id.clone(), SourceProvenance::new(path, local.span));
            }
        }
    }

    if let Some(render) = &component.render {
        for handler in render_event_handlers(render) {
            provenance.insert(
                handler.id.clone(),
                SourceProvenance::new(path, handler.span),
            );
        }
    }

    provenance
}

fn collect_semantic_references(
    component: &ComponentNode,
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> Vec<SemanticReference> {
    let mut references = component
        .actions
        .iter()
        .filter_map(|action| {
            component
                .state_fields
                .iter()
                .find(|field| field.name == action.field)
                .map(|field| SemanticReference {
                    kind: SemanticReferenceKind::ActionState,
                    source: action.id.clone(),
                    target: field.id.clone(),
                    provenance: provenance
                        .get(&action.id)
                        .expect("action semantic provenance should exist")
                        .clone(),
                })
        })
        .collect::<Vec<_>>();

    if let Some(render) = &component.render {
        references.extend(
            render_event_handlers(render)
                .into_iter()
                .filter_map(|handler| {
                    let method_name = this_member_name(&handler.handler)?;
                    component
                        .methods
                        .iter()
                        .find(|method| method.name == method_name)
                        .map(|method| SemanticReference {
                            kind: SemanticReferenceKind::EventMethod,
                            source: handler.id.clone(),
                            target: method.id.clone(),
                            provenance: provenance
                                .get(&handler.id)
                                .expect("event semantic provenance should exist")
                                .clone(),
                        })
                }),
        );
    }

    references
}

pub(crate) fn render_event_handlers(render: &RenderModel) -> Vec<&RenderEventHandler> {
    let mut event_handlers = render.event_handlers.iter().collect::<Vec<_>>();

    for child in &render.children {
        collect_child_event_handlers(child, &mut event_handlers);
    }
    if let Some(fragment) = &render.root_fragment {
        for child in &fragment.children {
            collect_child_event_handlers(child, &mut event_handlers);
        }
    }

    event_handlers
}

fn collect_child_event_handlers<'a>(
    child: &'a RenderChild,
    event_handlers: &mut Vec<&'a RenderEventHandler>,
) {
    match child {
        RenderChild::Element(element) => {
            event_handlers.extend(element.event_handlers.iter());

            for child in &element.children {
                collect_child_event_handlers(child, event_handlers);
            }
        }
        RenderChild::Fragment(fragment) => {
            for child in &fragment.children {
                collect_child_event_handlers(child, event_handlers);
            }
        }
        RenderChild::Conditional(conditional) => {
            for child in &conditional.when_true {
                collect_child_event_handlers(child, event_handlers);
            }
            for child in &conditional.when_false {
                collect_child_event_handlers(child, event_handlers);
            }
        }
        RenderChild::List(list) => {
            for child in &list.item_template {
                collect_child_event_handlers(child, event_handlers);
            }
        }
        RenderChild::Text { .. } | RenderChild::Binding { .. } => {}
    }
}

fn collect_duplicate_event_diagnostics(
    render: &RenderModel,
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    collect_duplicate_events_for_handlers(&render.event_handlers, class_name, diagnostics);

    for child in &render.children {
        collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
    }
    if let Some(fragment) = &render.root_fragment {
        for child in &fragment.children {
            collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
        }
    }
}

fn collect_render_attribute_diagnostics(
    render: &RenderModel,
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    collect_attribute_diagnostics_for_attributes(
        &render.attributes,
        state_fields,
        class_name,
        diagnostics,
        None,
    );

    for child in &render.children {
        collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
    }
    if let Some(fragment) = &render.root_fragment {
        for child in &fragment.children {
            collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
        }
    }
}

fn collect_render_list_diagnostics(
    render: &RenderModel,
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    for child in &render.children {
        collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
    }
    if let Some(fragment) = &render.root_fragment {
        for child in &fragment.children {
            collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
        }
    }
}

fn collect_child_list_diagnostics(
    child: &RenderChild,
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    match child {
        RenderChild::Element(element) => {
            for child in &element.children {
                collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::Fragment(fragment) => {
            for child in &fragment.children {
                collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::Conditional(conditional) => {
            for child in &conditional.when_true {
                collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
            }
            for child in &conditional.when_false {
                collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::List(list) => {
            collect_list_diagnostics(list, state_fields, class_name, diagnostics);

            for child in &list.item_template {
                collect_child_list_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::Text { .. } | RenderChild::Binding { .. } => {}
    }
}

#[allow(clippy::too_many_lines)]
fn collect_list_diagnostics(
    list: &RenderList,
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    if list.key_expression.is_empty() {
        diagnostics.push(ComponentDiagnostic {
            severity: ComponentDiagnosticSeverity::Error,
            effect_id: None,
            statement_id: None,
            context_declaration_candidate_id: None,
            context_id: None,
            provider_id: None,
            consumer_id: None,
            slot_id: None,
            invocation_id: None,
            component_instance_id: None,
            slot_binding_id: None,
            structural_region_id: None,
            component_id: None,
            provider_instance_id: None,
            consumer_instance_id: None,
            secondary_labels: Vec::new(),
            provenance: None,
            code: "PSC1011".to_string(),
            message: format!(
                "list over `{}` in class `{class_name}` is missing a `key={{...}}` attribute; stable keys are required for retained-node reconciliation",
                list.iterable
            ),
        });
        return;
    }

    if list.index_variable.as_deref() == Some(list.key_expression.as_str()) {
        diagnostics.push(ComponentDiagnostic {
            severity: ComponentDiagnosticSeverity::Error,
            effect_id: None,
            statement_id: None,
            context_declaration_candidate_id: None,
            context_id: None,
            provider_id: None,
            consumer_id: None,
            slot_id: None,
            invocation_id: None,
            component_instance_id: None,
            slot_binding_id: None,
            structural_region_id: None,
            component_id: None,
            provider_instance_id: None,
            consumer_instance_id: None,
            secondary_labels: Vec::new(),
            provenance: None,
            code: "PSC1012".to_string(),
            message: format!(
                "list key `{}` in class `{class_name}` uses the iteration index; index keys are unstable when items move",
                list.key_expression
            ),
        });
        return;
    }

    let member_path = list_member_key_path(list);
    if list.key_expression != list.item_variable && member_path.is_none() {
        diagnostics.push(ComponentDiagnostic {
            severity: ComponentDiagnosticSeverity::Error,
            effect_id: None,
            statement_id: None,
            context_declaration_candidate_id: None,
            context_id: None,
            provider_id: None,
            consumer_id: None,
            slot_id: None,
            invocation_id: None,
            component_instance_id: None,
            slot_binding_id: None,
            structural_region_id: None,
            component_id: None,
            provider_instance_id: None,
            consumer_instance_id: None,
            secondary_labels: Vec::new(),
            provenance: None,
            code: "PSC1013".to_string(),
            message: format!(
                "list key `{}` in class `{class_name}` is not supported yet; use the item variable `{}` or one of its object members",
                list.key_expression, list.item_variable
            ),
        });
        return;
    }

    let Some(field_name) = this_member_name(&list.iterable) else {
        return;
    };
    let Some(SerializableValue::Array(values)) = state_fields
        .iter()
        .find(|field| field.name == field_name)
        .and_then(|field| field.initial_value.as_ref())
    else {
        return;
    };

    let mut keys = Vec::new();
    for (index, value) in values.iter().enumerate() {
        let key_value = member_path.map_or(Some(value), |path| value.member_path_value(path));
        let Some(key) = key_value.and_then(list_key_from_static_value) else {
            diagnostics.push(ComponentDiagnostic {
            severity: ComponentDiagnosticSeverity::Error,
            effect_id: None,
            statement_id: None,
            context_declaration_candidate_id: None,
            context_id: None,
            provider_id: None,
            consumer_id: None,
            slot_id: None,
            invocation_id: None,
            component_instance_id: None,
            slot_binding_id: None,
            structural_region_id: None,
            component_id: None,
            provider_instance_id: None,
            consumer_instance_id: None,
            secondary_labels: Vec::new(),
                provenance: None,
                code: "PSC1015".to_string(),
                message: member_path.map_or_else(
                    || format!(
                        "list key `{}` resolves to a non-primitive initial item at index {index} in class `{class_name}`; keyed reconciliation requires primitive keys",
                        list.key_expression
                    ),
                    |_| format!(
                        "list key `{}` cannot resolve a primitive member value for initial item at index {index} in class `{class_name}`; every item must provide that member",
                        list.key_expression
                    ),
                ),
            });
            return;
        };

        if keys.contains(&key) {
            diagnostics.push(ComponentDiagnostic {
            severity: ComponentDiagnosticSeverity::Error,
            effect_id: None,
            statement_id: None,
            context_declaration_candidate_id: None,
            context_id: None,
            provider_id: None,
            consumer_id: None,
            slot_id: None,
            invocation_id: None,
            component_instance_id: None,
            slot_binding_id: None,
            structural_region_id: None,
            component_id: None,
            provider_instance_id: None,
            consumer_instance_id: None,
            secondary_labels: Vec::new(),
                provenance: None,
                code: "PSC1014".to_string(),
                message: format!(
                    "list key `{}` resolves to duplicate initial value `{key}` in class `{class_name}`; keyed reconciliation requires unique keys",
                    list.key_expression
                ),
            });
            return;
        }

        keys.push(key);
    }
}

fn list_member_key_path(list: &RenderList) -> Option<&str> {
    list.key_expression
        .strip_prefix(&list.item_variable)
        .and_then(|suffix| suffix.strip_prefix('.'))
        .filter(|path| !path.is_empty() && !path.split('.').any(str::is_empty))
}

fn list_key_from_static_value(value: &SerializableValue) -> Option<String> {
    match value {
        SerializableValue::Null => Some("null".to_string()),
        SerializableValue::Number(value) | SerializableValue::String(value) => Some(value.clone()),
        SerializableValue::Boolean(value) => Some(value.to_string()),
        SerializableValue::Array(_) | SerializableValue::Object(_) => None,
    }
}

fn collect_child_attribute_diagnostics(
    child: &RenderChild,
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    match child {
        RenderChild::Element(element) => {
            collect_attribute_diagnostics_for_attributes(
                &element.attributes,
                state_fields,
                class_name,
                diagnostics,
                None,
            );

            for child in &element.children {
                collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::Fragment(fragment) => {
            for child in &fragment.children {
                collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::Conditional(conditional) => {
            for child in &conditional.when_true {
                collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
            }
            for child in &conditional.when_false {
                collect_child_attribute_diagnostics(child, state_fields, class_name, diagnostics);
            }
        }
        RenderChild::List(list) => {
            for child in &list.item_template {
                collect_list_item_attribute_diagnostics(
                    child,
                    state_fields,
                    class_name,
                    diagnostics,
                    &list.item_variable,
                    list.index_variable.as_deref(),
                );
            }
        }
        RenderChild::Text { .. } | RenderChild::Binding { .. } => {}
    }
}

#[allow(clippy::too_many_lines)]
fn collect_attribute_diagnostics_for_attributes(
    attributes: &[RenderAttribute],
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
    list_scope: Option<(&str, Option<&str>)>,
) {
    let mut seen = Vec::<&str>::new();

    for attribute in attributes {
        // Phase I Form control bindings are compiler-owned candidates. Their
        // validity and later diagnostics consume I4 products rather than the
        // legacy ordinary-state attribute checker.
        if attribute.name == "field" {
            continue;
        }
        if !attribute.name.starts_with("on") {
            if seen.contains(&attribute.name.as_str()) {
                diagnostics.push(ComponentDiagnostic {
            severity: ComponentDiagnosticSeverity::Error,
            effect_id: None,
            statement_id: None,
            context_declaration_candidate_id: None,
            context_id: None,
            provider_id: None,
            consumer_id: None,
            slot_id: None,
            invocation_id: None,
            component_instance_id: None,
            slot_binding_id: None,
            structural_region_id: None,
            component_id: None,
            provider_instance_id: None,
            consumer_instance_id: None,
            secondary_labels: Vec::new(),
                    provenance: None,
                    code: "PSC1007".to_string(),
                    message: format!(
                        "attribute `{}` is declared more than once on the same element in class `{}`",
                        attribute.name, class_name
                    ),
                });
            } else if attribute.name != "{...}" {
                seen.push(&attribute.name);
            }
        }

        match &attribute.value {
            RenderAttributeValue::Expression(_) if attribute.name == "key" => {}
            RenderAttributeValue::Expression(expression)
                if !is_event_attribute(&attribute.name) =>
            {
                if expression.as_deref().is_some_and(|expression| {
                    list_scope
                        .is_some_and(|scope| list_item_attribute_expression(expression, scope))
                }) {
                    continue;
                }

                match expression.as_deref().and_then(this_member_name) {
                    Some(field_name)
                        if state_fields.iter().any(|field| field.name == field_name) => {}
                    Some(field_name) => diagnostics.push(ComponentDiagnostic {
            severity: ComponentDiagnosticSeverity::Error,
            effect_id: None,
            statement_id: None,
            context_declaration_candidate_id: None,
            context_id: None,
            provider_id: None,
            consumer_id: None,
            slot_id: None,
            invocation_id: None,
            component_instance_id: None,
            slot_binding_id: None,
            structural_region_id: None,
            component_id: None,
            provider_instance_id: None,
            consumer_instance_id: None,
            secondary_labels: Vec::new(),
                        provenance: None,
                        code: "PSC1003".to_string(),
                        message: format!(
                            "attribute binding `{}` references unknown state field `{field_name}` in class `{}`",
                            attribute.name, class_name
                        ),
                    }),
                    None => diagnostics.push(ComponentDiagnostic {
            severity: ComponentDiagnosticSeverity::Error,
            effect_id: None,
            statement_id: None,
            context_declaration_candidate_id: None,
            context_id: None,
            provider_id: None,
            consumer_id: None,
            slot_id: None,
            invocation_id: None,
            component_instance_id: None,
            slot_binding_id: None,
            structural_region_id: None,
            component_id: None,
            provider_instance_id: None,
            consumer_instance_id: None,
            secondary_labels: Vec::new(),
                        provenance: None,
                        code: "PSC1008".to_string(),
                        message: format!(
                            "attribute `{}` uses an unsupported expression value in class `{}`",
                            attribute.name, class_name
                        ),
                    }),
                }
            }
            RenderAttributeValue::Spread(_) => {
                diagnostics.push(ComponentDiagnostic {
                    severity: ComponentDiagnosticSeverity::Error,
                    effect_id: None,
                    statement_id: None,
                    context_declaration_candidate_id: None,
                    context_id: None,
                    provider_id: None,
                    consumer_id: None,
                    slot_id: None,
                    invocation_id: None,
                    component_instance_id: None,
                    slot_binding_id: None,
                    structural_region_id: None,
                    component_id: None,
                    provider_instance_id: None,
                    consumer_instance_id: None,
                    secondary_labels: Vec::new(),
                    provenance: None,
                    code: "PSC1009".to_string(),
                    message: format!(
                        "JSX spread attributes are not supported yet in class `{class_name}`"
                    ),
                });
            }
            RenderAttributeValue::Unsupported if !is_event_attribute(&attribute.name) => {
                diagnostics.push(ComponentDiagnostic {
                    severity: ComponentDiagnosticSeverity::Error,
                    effect_id: None,
                    statement_id: None,
                    context_declaration_candidate_id: None,
                    context_id: None,
                    provider_id: None,
                    consumer_id: None,
                    slot_id: None,
                    invocation_id: None,
                    component_instance_id: None,
                    slot_binding_id: None,
                    structural_region_id: None,
                    component_id: None,
                    provider_instance_id: None,
                    consumer_instance_id: None,
                    secondary_labels: Vec::new(),
                    provenance: None,
                    code: "PSC1010".to_string(),
                    message: format!(
                        "attribute `{}` uses an unsupported JSX value in class `{}`",
                        attribute.name, class_name
                    ),
                });
            }
            _ => {}
        }
    }
}

fn collect_list_item_attribute_diagnostics(
    child: &RenderChild,
    state_fields: &[StateField],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
    item_variable: &str,
    index_variable: Option<&str>,
) {
    match child {
        RenderChild::Element(element) => {
            collect_attribute_diagnostics_for_attributes(
                &element.attributes,
                state_fields,
                class_name,
                diagnostics,
                Some((item_variable, index_variable)),
            );

            for child in &element.children {
                collect_list_item_attribute_diagnostics(
                    child,
                    state_fields,
                    class_name,
                    diagnostics,
                    item_variable,
                    index_variable,
                );
            }
        }
        RenderChild::Fragment(fragment) => {
            for child in &fragment.children {
                collect_list_item_attribute_diagnostics(
                    child,
                    state_fields,
                    class_name,
                    diagnostics,
                    item_variable,
                    index_variable,
                );
            }
        }
        RenderChild::Conditional(conditional) => {
            for child in &conditional.when_true {
                collect_list_item_attribute_diagnostics(
                    child,
                    state_fields,
                    class_name,
                    diagnostics,
                    item_variable,
                    index_variable,
                );
            }
            for child in &conditional.when_false {
                collect_list_item_attribute_diagnostics(
                    child,
                    state_fields,
                    class_name,
                    diagnostics,
                    item_variable,
                    index_variable,
                );
            }
        }
        RenderChild::List(list) => {
            for child in &list.item_template {
                collect_list_item_attribute_diagnostics(
                    child,
                    state_fields,
                    class_name,
                    diagnostics,
                    &list.item_variable,
                    list.index_variable.as_deref(),
                );
            }
        }
        RenderChild::Text { .. } | RenderChild::Binding { .. } => {}
    }
}

fn list_item_attribute_expression(expression: &str, scope: (&str, Option<&str>)) -> bool {
    expression == scope.0
        || scope.1 == Some(expression)
        || expression
            .strip_prefix(scope.0)
            .and_then(|suffix| suffix.strip_prefix('.'))
            .is_some_and(|path| !path.is_empty() && !path.split('.').any(str::is_empty))
}

fn is_event_attribute(name: &str) -> bool {
    name.strip_prefix("on")
        .and_then(|event| event.chars().next())
        .is_some_and(char::is_uppercase)
}

fn collect_duplicate_child_event_diagnostics(
    child: &RenderChild,
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    match child {
        RenderChild::Element(element) => {
            collect_duplicate_events_for_handlers(&element.event_handlers, class_name, diagnostics);

            for child in &element.children {
                collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
            }
        }
        RenderChild::Fragment(fragment) => {
            for child in &fragment.children {
                collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
            }
        }
        RenderChild::Conditional(conditional) => {
            for child in &conditional.when_true {
                collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
            }
            for child in &conditional.when_false {
                collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
            }
        }
        RenderChild::List(list) => {
            for child in &list.item_template {
                collect_duplicate_child_event_diagnostics(child, class_name, diagnostics);
            }
        }
        RenderChild::Text { .. } | RenderChild::Binding { .. } => {}
    }
}

fn collect_duplicate_events_for_handlers(
    event_handlers: &[RenderEventHandler],
    class_name: &str,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    let mut seen = Vec::<&str>::new();

    for event_handler in event_handlers {
        if seen.contains(&event_handler.event.as_str()) {
            diagnostics.push(ComponentDiagnostic {
                severity: ComponentDiagnosticSeverity::Error,
                effect_id: None,
                statement_id: None,
                context_declaration_candidate_id: None,
                context_id: None,
                provider_id: None,
                consumer_id: None,
                slot_id: None,
                invocation_id: None,
                component_instance_id: None,
                slot_binding_id: None,
                structural_region_id: None,
                component_id: None,
                provider_instance_id: None,
                consumer_instance_id: None,
                secondary_labels: Vec::new(),
                provenance: None,
                code: "PSC1006".to_string(),
                message: format!(
                    "event `{}` is declared more than once on the same element in class `{}`",
                    event_handler.event, class_name
                ),
            });
        } else {
            seen.push(&event_handler.event);
        }
    }
}

fn this_member_name(reference: &str) -> Option<&str> {
    reference.strip_prefix("this.")
}
