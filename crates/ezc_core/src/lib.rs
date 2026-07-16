//! Core compiler data structures for the first `EdgeZero` learning slice.
//!
//! This crate deliberately does **not** parse TSX yet. It records a source summary,
//! spans, obvious declarations, and diagnostics. That gives the project a stable
//! place to learn compiler fundamentals before choosing a real parser backend.

pub mod application_semantic_model;
pub mod asm_validation;
pub mod binding_table;
pub mod compilation_unit;
pub mod compiler_pass;
pub mod component_composition;
pub mod component_diagnostics;
pub mod component_graph;
pub mod component_initialization;
pub mod component_instance;
pub mod component_instance_scope;
pub mod component_invocation;
pub mod component_ir;
pub mod component_ir_optimization;
pub mod component_scope;
pub mod composition_typing;
pub mod computed_instance_slots;
pub mod computed_value;
pub mod consumer;
pub mod context;
pub mod context_declaration_candidate;
pub mod context_dependency;
mod context_designator;
pub mod context_diagnostics;
pub mod context_evaluation;
pub mod context_inspection;
pub mod context_lifetime;
pub mod context_ownership;
pub mod context_resolution;
pub mod context_resume;
pub mod context_typing;
pub mod context_update;
pub mod effect;
pub mod effect_capability;
pub mod effect_diagnostics;
pub mod effect_inspection;
pub mod effect_resume;
pub mod explain;
pub mod expression_graph;
pub mod form;
pub mod form_binding;
pub mod form_diagnostics;
pub mod form_field;
pub mod form_inspection;
pub mod form_ir;
pub mod form_ir_optimization;
pub mod form_ownership;
pub mod form_reset;
pub mod form_serialization;
pub mod form_submission;
pub mod form_submission_host;
pub mod form_tracking;
pub mod form_validation;
pub mod form_validation_plan;
pub mod html_codegen;
pub mod instance_context;
pub mod intermediate_representation;
pub mod layout_graph;
pub mod lazy_action_chunks;
pub mod model;
pub mod module_graph;
pub mod ordinary_html_codegen;
pub mod ordinary_template_instance;
pub mod ordinary_template_integrity;
pub mod page_codegen;
pub mod provider;
pub mod resume_activation;
pub mod resume_boot;
pub mod resume_boundary;
pub mod resume_capture;
pub mod resume_chunk;
pub mod resume_diagnostics;
pub mod resume_explain;
pub mod resume_identity;
pub mod resume_instance;
pub mod resume_liveness;
pub mod resume_manifest;
pub mod resume_plan;
pub mod resume_schema;
pub mod route_graph;
pub mod runtime_codegen;
pub mod runtime_component;
pub mod runtime_component_artifact;
pub mod runtime_computed;
pub mod runtime_computed_artifact;
pub mod runtime_context;
pub mod runtime_context_artifact;
pub mod runtime_effect;
pub mod runtime_effect_artifact;
pub mod runtime_form_artifact;
pub mod runtime_form_registry;
pub mod semantic_graph;
pub mod semantic_id;
pub mod semantic_provenance;
pub mod semantic_reference;
pub mod semantic_type;
pub mod slot;
pub mod slot_binding;
pub mod slot_content;
pub mod state_instance_storage;
pub mod summarize;
pub mod symbol_table;
pub mod template_graph;
pub mod template_manifest;
pub mod template_semantics;

pub use application_semantic_model::{
    build_application_semantic_model, build_application_semantic_model_for_unit,
    build_application_semantic_model_from_component_graph, ApplicationSemanticModel,
    SemanticEntity, SemanticEntityKind,
};
pub use asm_validation::{validate_application_semantic_model, AsmValidationDiagnostic};
pub use binding_table::{
    build_binding_table, BindingDiagnostic, BindingTable, ExportBinding, ImportBinding,
    ImportBindingTarget, ModuleBindingTable,
};
pub use compilation_unit::CompilationUnit;
pub use compiler_pass::{
    fold_component_graph, AnalysisPass, ConstantEvaluation, ConstantEvaluationPass,
    ConstantFoldingPass, DependencyAnalysis, DependencyAnalysisPass, ImmutableAsmPass,
};
pub use component_composition::{
    analyze_component_composition, ComponentCompositionAnalysis, ComponentCompositionCycle,
};
pub use component_diagnostics::{
    collect_component_diagnostics, ComponentDiagnosticContract, COMPONENT_DIAGNOSTIC_CONTRACTS,
};
pub use component_graph::{
    build_component_graph, build_component_graph_for_module, ArithmeticEvaluationError,
    ArithmeticExpression, ArithmeticExpressionKind, ArithmeticOperator, AuthoredComponentHeritage,
    AuthoredContextDeclarationCandidate, AuthoredDeclarationKind, AuthoredSlotDeclarationCandidate,
    AuthoredSubmissionDeclarationFact, AuthoredValidationRuleArgument,
    AuthoredValidationRuleArgumentKind, AuthoredValidationRuleDeclarationFact,
    AuthoredValidationRuleExpression, AuthoredValidationRuleExpressionKind, ComparisonOperator,
    ComponentAction, ComponentDiagnostic, ComponentDiagnosticSeverity, ComponentGraph,
    ComponentMethod, ComponentNode, ComputedExpression, ComputedExpressionKind,
    ConstantEvaluationError, ConstantExpression, ConstantExpressionKind, ConsumerDeclaration,
    ContextDeclaration, ContextDeclarationCandidateKind, ContextDeclarationViolation,
    ContextDesignator, DeclaredStateType, DeclaredStateTypeKind, DiagnosticSecondaryLabel,
    EffectBodySyntax, EffectExpression, EffectExpressionKind, EffectStatementSyntax,
    EffectStatementSyntaxKind, FormDeclarationCandidate, FormDeclarationStatus,
    FormDeclarationViolation, FormDesignatorFact, FormFieldDeclarationCandidate,
    FormFieldDeclarationViolation, LogicalOperator, MethodCall, MethodLocalVariable,
    MethodParameter, RenderAttribute, RenderAttributeValue, RenderChild, RenderEventHandler,
    RenderFragment, RenderList, RenderModel, SerializableValue, SlotDeclaration,
    SlotDeclarationViolation, SlotKind, StateField, StateOperation, UnsupportedEffectStatementKind,
    UnsupportedFormDesignatorFact,
};
pub use component_initialization::{
    plan_component_initialization, ComponentInitializationPlan, ComponentInstanceBatch,
    SlotBindingBatch,
};
pub use component_instance::{
    plan_component_instances, BlockedComponentInstancePlan, BlockedComponentInstanceReason,
    ComponentBuildRoot, ComponentBuildRootKind, ComponentInstance, ComponentInstancePlan,
    ComponentInstanceStatus,
};
pub use component_instance_scope::{
    build_component_instance_scope_graph, validate_component_instance_scope_graph,
    ComponentInstanceScopeDiagnostic, ComponentInstanceScopeGraph, ComponentInstanceScopeNode,
    ComponentInstanceScopeViolation,
};
pub use component_invocation::{
    collect_component_invocations, ComponentInvocationEntity, ComponentInvocationResolutionStatus,
};
pub use component_ir::{
    lower_component_ir, validate_component_ir, ComponentIrInstruction, ComponentIrOperation,
    ComponentIrReport,
};
pub use component_ir_optimization::{
    optimize_component_ir, validate_optimized_component_ir, OptimizedComponentIrReport,
};
pub use component_scope::{ComponentScopeDiagnostic, ComponentScopeGraph};
pub use composition_typing::{
    collect_composition_type_products, ComponentInvocationTypeRecord, CompositionCompatibility,
    CompositionTypeProducts, InstanceContextBindingTypeRecord, SlotBindingTypeRecord,
};
pub use computed_instance_slots::{
    build_computed_instance_slot_registry, validate_computed_instance_slot_registry,
    ComputedInstanceSlotRecord, ComputedInstanceSlotRegistry,
    COMPUTED_INSTANCE_SLOT_REGISTRY_VERSION,
};
pub use computed_value::{
    collect_computed_values, ComputedCachePolicy, ComputedDiagnosticCode, ComputedPurity,
    ComputedPurityViolation, ComputedPurityViolationKind, ComputedValue,
};
pub use consumer::{collect_consumer_entities, ConsumerEntity, ContextResolutionState};
pub use context::{collect_context_entities, ContextEntity};
pub use context_declaration_candidate::{
    collect_context_declaration_candidates, ContextDeclarationCandidate,
    ContextDeclarationCandidateRegistry, ContextDeclarationStatus, ContextSemanticEntityId,
};
pub use context_dependency::{
    collect_context_dependency_graph, ContextDependencyCompatibility, ContextDependencyEdge,
    ContextDependencyEdgeKind, ContextDependencyGraph, ContextDependencyNode,
    ContextDependencyNodeId, ContextDependencyNodeKind,
};
pub use context_diagnostics::collect_context_diagnostics;
pub use context_evaluation::{
    collect_context_evaluation_plan, ContextConsumerAvailabilityEntry,
    ContextConsumerAvailabilityStatus, ContextEvaluationBatch, ContextEvaluationBatchId,
    ContextEvaluationPlan, ContextEvaluationPlanId, ContextSourceBlockReason,
    ContextSourcePlanEntry, ContextSourcePlanStatus, ContextValueSourceId,
};
pub use context_inspection::{
    build_context_inspection_registry, ContextInspection, ContextInspectionRegistry,
};
pub use context_lifetime::{
    collect_context_lifetime_analysis, ContextBindingLifetimeRecord, ContextBindingLifetimeSource,
    ContextBindingLifetimeStatus, ContextDefaultLifetimeRecord, ContextDependencyLifetimeRecord,
    ContextEntityLifetimeRecord, ContextLifetimeAnalysis, ContextLifetimeEntityId,
    ContextLifetimeId, LifetimeCompatibilityStatus, ProviderLifetimeRecord,
};
pub use context_ownership::{
    collect_context_ownership_graph, ContextOwnedEntities, ContextOwnershipEdge,
    ContextOwnershipEdgeKind, ContextOwnershipGraph, ContextOwnershipNode, ContextOwnershipNodeId,
    ContextOwnershipNodeKind, ContextOwnershipOwnerId, ContextOwnershipTargetId,
};
pub use context_resolution::{
    collect_context_resolutions, ContextResolution, ContextResolutionResult,
};
pub use context_resume::{
    build_context_resume_plan, ContextResumePlan, ContextResumeRecord, ContextResumeSlotId,
    ContextSlotResumeStatus,
};
pub use context_typing::{
    collect_context_type_products, CompatibilityStatus, ConsumerTypeRecord,
    ContextBindingCompatibility, ContextBindingTypeRecord, ContextSerializationCompatibility,
    ContextTypeProducts, ContextTypeRecord, ProviderTypeRecord,
};
pub use context_update::{build_context_update_plan, ContextActionUpdatePlan, ContextUpdatePlan};
pub use effect::{
    analyze_effect_reactivity, collect_effects, derive_effect_trigger_plan, lower_effect_bodies,
    plan_effect_execution, validate_effects, ActionBatch, ActionBatchEffectTrigger,
    ActionEffectExecutionPlan, Effect, EffectBody, EffectComputedPrerequisiteBatch,
    EffectExecutionBatch, EffectExecutionPlan, EffectExecutionPolicy, EffectReactiveAnalysis,
    EffectRenderBoundary, EffectSemanticViolation, EffectSemanticViolationKind, EffectStatement,
    EffectStatementKind, EffectTriggerPlan, EffectValidation, InitialEffectExecutionPlan,
    UnplannedEffect, UnplannedEffectReason,
};
pub use effect_capability::{
    ArgumentSerializationPolicy, BuiltinCapabilityProvenance, CapabilityDefinition, CapabilityId,
    CapabilityOperation, CapabilityOperationId, CapabilityOperationKind, CapabilityParameters,
    CapabilityResultPolicy, CapabilitySignature, CapabilityValueContract, EffectCapabilityRegistry,
    RuntimeCapabilityLowering, StaticCapabilityPath, EFFECT_CAPABILITY_REGISTRY,
    EFFECT_CAPABILITY_REGISTRY_VERSION,
};
pub use effect_diagnostics::{collect_effect_diagnostics, EffectDiagnosticCode};
pub use effect_inspection::{
    build_effect_inspection_registry, validate_effect_inspection_registry, EffectInspection,
    EffectInspectionActionTrigger, EffectInspectionCapability, EffectInspectionDependencies,
    EffectInspectionInitialTrigger, EffectInspectionIr, EffectInspectionPrerequisiteBatch,
    EffectInspectionProvenance, EffectInspectionRegistry, EffectInspectionResumability,
    EffectInspectionRuntime, EffectInspectionSchedule, EffectInspectionScheduledAction,
    EffectInspectionUnplanned, EffectInspectionValidation, EffectInspectionValidationDiagnostic,
    EffectInspectionViolation,
};
pub use effect_resume::{
    build_effect_resume_plan, validate_effect_resume_plan, EffectActivationSlotId,
    EffectActivationStatus, EffectInitialResumeMembership, EffectResumePlan, EffectResumeRecord,
    EffectResumeValidationDiagnostic,
};
pub use explain::{explain_json, explain_text};
pub use expression_graph::{ExpressionGraph, ExpressionNode, ExpressionNodeKind};
pub use form::{collect_form_entities, FormEntity};
pub use form_binding::{
    collect_form_field_binding_products, FormControlChannel, FormControlCompatibility,
    FormControlNormalization, FormFieldBinding, FormFieldBindingCandidate,
    FormFieldBindingEvidence, FormFieldBindingEvidenceKind, FormFieldBindingExpressionFact,
    FormFieldBindingProducts, FormFieldBindingViolation, FormInputKind,
};
pub use form_diagnostics::{
    collect_form_diagnostics, FormDiagnosticReservation, FORM_DIAGNOSTIC_RESERVATIONS,
};
pub use form_field::{collect_form_field_products, FormFieldEntity, FormFieldProducts};
pub use form_inspection::{build_form_inspection_registry, FormInspection, FormInspectionRegistry};
pub use form_ir::{
    lower_form_ir, FormInstanceIr, FormIrOperation, FormIrReport, FormRuntimeStorage,
};
pub use form_ir_optimization::{
    optimize_form_ir, FormIrOptimizationMetrics, OptimizedFormIrReport,
};
pub use form_ownership::{
    collect_form_ownership_graph, validate_form_ownership_graph, FormOwnershipEdge,
    FormOwnershipEdgeKind, FormOwnershipGraph, FormOwnershipIntegrityDiagnostic,
    FormOwnershipIntegrityKind, FormOwnershipNode, FormOwnershipNodeKey, FormOwnershipValidation,
    FormReferenceEdge, FormReferenceKind,
};
pub use form_reset::{
    collect_reset_products, FieldResetOperation, FieldResetStep, FormResetPlan, ResetProducts,
};
pub use form_serialization::{
    collect_serialization_products, FormFieldSerializationConversion, FormSerializationFormat,
    FormSerializationPlan, SerializationDeclarationFact, SerializationPlanStatus,
    SerializationProducts, SerializedFieldPlan,
};
pub use form_submission::{
    collect_submission_products, FormSubmissionPlan, SubmissionDeclarationCandidate,
    SubmissionDeclarationViolation, SubmissionProducts, SubmitResetPolicy,
};
pub use form_submission_host::{
    collect_submission_host_products, SubmissionHost, SubmissionHostCandidate,
    SubmissionHostProducts, SubmissionHostViolation,
};
pub use form_tracking::{
    collect_form_tracking_products, structurally_equal_serializable_values,
    validate_dirty_tracking_graph, validate_touched_tracking_graph, DirtyTrackingGraph,
    DirtyTrackingPlan, DirtyTransitionCause, DirtyTransitionPlan, FieldDirtyTracking,
    FieldTouchedTracking, FormTrackingIntegrityDiagnostic, FormTrackingIntegrityKind,
    FormTrackingProducts, FormTrackingValidation, TouchedTrackingGraph, TouchedTrackingPlan,
    TouchedTransitionCause, TouchedTransitionPlan,
};
pub use form_validation::{
    collect_validation_graph, collect_validation_products, validate_validation_graph,
    ValidationCompatibility, ValidationDependencyCycle, ValidationDependencyDesignator,
    ValidationGraph, ValidationGraphEdge, ValidationGraphEdgeKind,
    ValidationGraphIntegrityDiagnostic, ValidationGraphIntegrityKind, ValidationGraphNode,
    ValidationGraphNodeKey, ValidationGraphValidation, ValidationProducts, ValidationRule,
    ValidationRuleArgument, ValidationRuleCandidate, ValidationRuleKind, ValidationRuleViolation,
};
pub use form_validation_plan::{
    collect_validation_dependency_plans, validate_validation_dependency_plans,
    BlockedFieldValidationDependency, FieldChangeSet, FieldChangeValidationSchedule,
    FieldDependencyBlockReason, FieldValidationChangePlan, FieldValidationDependency,
    FieldValidationSourceEntry, FieldValidationTargetEntry, FormValidationDependencyPlan,
    ValidationDependencyPlanIntegrityDiagnostic, ValidationDependencyPlanIntegrityKind,
    ValidationDependencyPlanValidation, ValidationDependencyPlans, ValidationPlanningStatus,
};
pub use html_codegen::generate_static_html;
pub use instance_context::{
    collect_instance_context_registry, ConsumerInstanceId, ConsumerInstanceRecord,
    ContextDefaultSourceInstanceId, ContextSourceInstanceId, ContextSourceInstanceOwner,
    InstanceContextRegistry, InstanceContextResolution, InstanceContextResolutionStatus,
    InstanceContextValueSlotId, ProviderInstanceId, ProviderInstanceRecord,
};
pub use intermediate_representation::{
    analyze_constant_propagation, analyze_dead_assignments, analyze_definition_uses,
    analyze_liveness, analyze_reachability, analyze_reactive_cycles,
    analyze_reactive_transitive_graph, analyze_use_definitions, build_reactive_graph,
    compute_dominators, compute_post_dominators, computed_optimization_pipeline, inspect_dom_nodes,
    lower_components_to_ir, optimize_computed_ir, optimize_context_ir, optimize_effect_ir,
    plan_computed_evaluation, validate_context_ir, validate_effect_ir,
    validate_intermediate_representation, validate_optimized_context_ir, ContextConsumerLoadId,
    ContextIrReport, ContextSourceFunctionId, ContextValueSlotId, IntermediateRepresentation,
    IrBinaryOperation, IrBlock, IrBlockId, IrBranchArm, IrBranchEdge, IrCfgCleanupPass,
    IrCommonSubexpressionEliminationPass, IrComputedEvaluation, IrComputedEvaluationPlan,
    IrConstant, IrConstantFoldingPass, IrConstantPropagationAnalysis, IrContextConsumerBinding,
    IrContextLoad, IrContextSourceEvaluation, IrCopyPropagationPass, IrDeadAssignmentAnalysis,
    IrDeadCodeEliminationPass, IrDefinitionUseAnalysis, IrDomAttribute, IrDomAttributeValue,
    IrDomBinding, IrDomConditional, IrDomEvent, IrDomInspection, IrDomList, IrDomNode, IrDomNodeId,
    IrDomNodeKind, IrDomText, IrDominatorTree, IrEffectCompletion, IrEffectExecution, IrFunction,
    IrInstruction, IrInstructionId, IrInstructionKind, IrInstructionSimplificationPass,
    IrLivenessAnalysis, IrLoop, IrLoopId, IrModule, IrOperand, IrOptimizationMetrics,
    IrOptimizationPass, IrOptimizationPassReport, IrOptimizationPipeline, IrOptimizationReport,
    IrPassManager, IrPostDominatorTree, IrReachabilityAnalysis, IrReactiveCycle,
    IrReactiveCycleAnalysis, IrReactiveEdge, IrReactiveEdgeKind, IrReactiveGraph, IrReactiveNode,
    IrReactiveNodeKind, IrReactiveTransitiveAnalysis, IrSchedulerInspection, IrStorage,
    IrStorageId, IrTemplateEntrypoint, IrUnaryOperation, IrUpdateScheduler, IrUse, IrUseDefinition,
    IrValidationDiagnostic, IrValue, IrValueDefinition, IrValueId, OptimizedContextIrReport,
    OptimizedIrContextSourceEvaluation,
};
pub use model::{
    ClassSummary, DecoratorSummary, Diagnostic, RenderMethodSummary, Severity, SourceSummary, Span,
};
pub use module_graph::{
    build_module_graph, ModuleEdge, ModuleEdgeKind, ModuleGraph, ModuleNode, ModuleTarget,
};
pub use ordinary_html_codegen::generate_ordinary_instance_html;
pub use ordinary_template_instance::{
    build_ordinary_template_instance_registry, validate_ordinary_template_instance_registry,
    OrdinaryTemplateBindingKind, OrdinaryTemplateInstanceBindingRecord,
    OrdinaryTemplateInstanceEventRecord, OrdinaryTemplateInstanceRegistry,
    OrdinaryTemplateInstanceTargetRecord, OrdinaryTemplateTargetKind,
    ORDINARY_TEMPLATE_INSTANCE_REGISTRY_VERSION,
};
pub use ordinary_template_integrity::{
    ComputedInstanceSlotIntegrityCode, OrdinaryTemplateIntegrityCode,
    StateInstanceStorageIntegrityCode,
};
pub use page_codegen::{
    generate_standalone_page, generate_standalone_page_with_component_runtime,
    generate_standalone_page_with_component_runtime_and_forms,
    generate_standalone_page_with_computed_runtime, generate_standalone_page_with_context_runtime,
    generate_standalone_page_with_effect_runtime,
};
pub use provider::{collect_provider_entities, DuplicateProviderDeclaration, ProviderEntity};
pub use resume_activation::{
    build_resume_activation_plan, validate_resume_activation_plan, ResumeActivationBlock,
    ResumeActivationBlockReason, ResumeActivationIntegrityCode,
    ResumeActivationIntegrityDiagnostic, ResumeActivationPlan, ResumeActivationPolicy,
    ResumeActivationPolicyDecision, ResumeActivationPrerequisite, RESUME_ACTIVATION_PLAN_VERSION,
};
pub use resume_boundary::{
    build_resume_boundary_graph, validate_resume_boundary_graph, ResumeBoundary,
    ResumeBoundaryActivationIdentity, ResumeBoundaryActivationProgram,
    ResumeBoundaryActivationReference, ResumeBoundaryBlock, ResumeBoundaryBlockSource,
    ResumeBoundaryGraph, ResumeBoundaryIntegrityCode, ResumeBoundaryIntegrityDiagnostic,
    ResumeBoundaryOwner, ResumeBoundaryOwnershipEdge, RESUME_BOUNDARY_GRAPH_VERSION,
};
pub use resume_capture::{
    build_resume_capture_plan, capture_resume_snapshot, encode_resume_value,
    resume_snapshot_artifact_json, resume_snapshot_json, validate_resume_capture_plan,
    ResumeCaptureBlock, ResumeCaptureBlockReason, ResumeCaptureError, ResumeCaptureErrorKind,
    ResumeCaptureInstruction, ResumeCaptureIntegrityCode, ResumeCaptureIntegrityDiagnostic,
    ResumeCapturePlan, ResumeCaptureProgram, ResumeEncodedValue, ResumeEnvelopeWriterPlan,
    ResumeSnapshotBoundaryV1, ResumeSnapshotV1, ResumeSnapshotValueRecordV1,
    RuntimeQuiescenceState, RESUME_CAPTURE_MANIFEST_VERSION, RESUME_CAPTURE_PLAN_VERSION,
    RESUME_SNAPSHOT_SCHEMA_VERSION,
};
pub use resume_chunk::{
    build_resume_chunk_graph, validate_resume_chunk_graph, ResumeChunk, ResumeChunkBlock,
    ResumeChunkBlockReason, ResumeChunkGraph, ResumeChunkIntegrityCode,
    ResumeChunkIntegrityDiagnostic, ResumeChunkModulePlan, ResumeChunkProgram,
    ResumeChunkProgramInclusion, ResumeChunkRootKind, RESUME_CHUNK_GRAPH_VERSION,
};
pub use resume_diagnostics::{
    ResumeDiagnosticReservation, RESUME_DIAGNOSTIC_RESERVATIONS, RESUME_INTEGRITY_RESERVATION_END,
    RESUME_INTEGRITY_RESERVATION_START,
};
pub use resume_identity::{
    ComputedInstanceCacheSlotId, ComputedInstanceDirtySlotId, ResumeActivationId,
    ResumeActivationRootKind, ResumeAnchorId, ResumeBoundaryId, ResumeBoundaryKind, ResumeBuildId,
    ResumeCaptureProgramId, ResumeChunkGroupId, ResumeChunkId, ResumeEventId,
    ResumeIdentityParseError, ResumeRestoreProgramId, ResumeSchemaId, ResumeSlotId,
    ResumeSnapshotId, ResumeValueRecordId, StateInstanceSlotId, TemplateInstanceBindingId,
    TemplateInstanceTargetId,
};
pub use resume_liveness::{
    build_resume_liveness_plan, validate_resume_liveness_plan, ResumeExcludedSlot,
    ResumeExistingSlot, ResumeLivenessBlock, ResumeLivenessBlockReason,
    ResumeLivenessClassificationKind, ResumeLivenessClassificationRef, ResumeLivenessIntegrityCode,
    ResumeLivenessIntegrityDiagnostic, ResumeLivenessOwner, ResumeLivenessPlan, ResumeLivenessSlot,
    ResumeRecomputableSlot, ResumeRecomputationProof, ResumeRetainedSlot, ResumeRetentionReason,
    RESUME_LIVENESS_PLAN_VERSION,
};
pub use resume_manifest::{
    build_resume_manifest, resume_manifest_json, validate_resume_manifest, ResumeManifest,
    ResumeManifestContextSlotRecord, ResumeManifestEffectRecord,
    ResumeManifestValidationDiagnostic, RESUME_MANIFEST_SCHEMA_VERSION,
};
pub use resume_plan::{
    build_resume_plan, ComponentInstanceResumePlan, FormFieldResumePlan, FormInstanceResumePlan,
    ResumeComponentPlan, ResumeComputedPlan, ResumePlan, SlotBindingResumePlan,
    StructuralRegionResumePlan,
};
pub use resume_schema::{
    build_resume_schema_registry, resume_value_codec, validate_resume_schema_registry,
    ResumeBoundarySchema, ResumeObjectPropertyCodec, ResumeSchemaBlock, ResumeSchemaBlockReason,
    ResumeSchemaIntegrityCode, ResumeSchemaIntegrityDiagnostic, ResumeSchemaRegistry,
    ResumeSlotSchema, ResumeValueCodec, RESUME_SCHEMA_REGISTRY_VERSION,
};
pub use runtime_codegen::generate_runtime_stub;
pub use runtime_component::{
    build_runtime_component_registry, RuntimeComponentContextBindingRecord,
    RuntimeComponentDefinitionRecord, RuntimeComponentInitializationBatch,
    RuntimeComponentInstanceRecord, RuntimeComponentRegistry, RuntimeComponentSlotBindingRecord,
    RUNTIME_COMPONENT_REGISTRY_SCHEMA_CONTRACT_VERSION,
};
pub use runtime_component_artifact::{
    build_runtime_component_artifact, runtime_component_artifact_json,
    validate_runtime_component_artifact, RuntimeComponentArtifact,
    RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION,
};
pub use runtime_computed::{
    build_runtime_computed_registry, ComputedCacheSlotId, ComputedDirtyFlagId,
    RuntimeComputedCacheSlot, RuntimeComputedDirtyFlag, RuntimeComputedRecord,
    RuntimeComputedRegistry,
};
pub use runtime_computed_artifact::{
    build_runtime_computed_artifact, runtime_computed_artifact_json, RuntimeComputedArtifact,
    RuntimeComputedArtifactDirtyFlag, RuntimeComputedArtifactEvaluation,
    RuntimeComputedArtifactInvalidation, RuntimeComputedArtifactSerialization,
    RUNTIME_COMPUTED_ARTIFACT_SCHEMA_VERSION,
};
pub use runtime_context::{
    build_runtime_context_registry, validate_runtime_context_registry,
    RuntimeContextConsumerRecord, RuntimeContextEvaluationBatch, RuntimeContextRegistry,
    RuntimeContextRegistryValidationDiagnostic, RuntimeContextSourceKind,
    RuntimeContextSourceRecord, RUNTIME_CONTEXT_REGISTRY_SCHEMA_CONTRACT_VERSION,
};
pub use runtime_context_artifact::{
    build_runtime_context_artifact, runtime_context_artifact_json, RuntimeContextArtifact,
    SerializedContextActionUpdatePlan, SerializedContextBatchId, SerializedContextConsumerBinding,
    SerializedContextEvaluationBatch, SerializedContextExecutionBoundary,
    SerializedContextInstruction, SerializedContextInstructionKind, SerializedContextProgram,
    SerializedContextSource, SerializedContextSourceKind, RUNTIME_CONTEXT_ARTIFACT_SCHEMA_VERSION,
};
pub use runtime_effect::{
    build_runtime_effect_registry, RuntimeActionBatchEffectTrigger, RuntimeEffectRecord,
    RuntimeEffectRegistry, RuntimeInitialEffectTrigger,
};
pub use runtime_effect_artifact::{
    build_runtime_effect_artifact, runtime_effect_artifact_json, RuntimeEffectArtifact,
    RuntimeEffectArtifactActionTrigger, RuntimeEffectArtifactCapabilityInstructionKind,
    RuntimeEffectArtifactCapabilityOperation, RuntimeEffectArtifactEffect,
    RuntimeEffectArtifactExecutionBoundary, RuntimeEffectArtifactExecutionPolicy,
    RuntimeEffectArtifactInitialTrigger, RuntimeEffectArtifactInstruction,
    RuntimeEffectArtifactPrerequisiteBatch, RuntimeEffectArtifactProgram,
    RuntimeEffectArtifactRenderBoundary, RUNTIME_EFFECT_ARTIFACT_SCHEMA_VERSION,
};
pub use runtime_form_artifact::{
    build_runtime_forms_artifact, runtime_forms_artifact_json, validate_runtime_forms_artifact,
    RuntimeFormsArtifact, RuntimeFormsArtifactBinding, RuntimeFormsArtifactDependency,
    RuntimeFormsArtifactField, RuntimeFormsArtifactFieldProgram, RuntimeFormsArtifactFieldSlots,
    RuntimeFormsArtifactForm, RuntimeFormsArtifactInstance, RuntimeFormsArtifactPrograms,
    RuntimeFormsArtifactReset, RuntimeFormsArtifactRule, RuntimeFormsArtifactSerialization,
    RuntimeFormsArtifactSubmission, RuntimeFormsArtifactValidation,
    RUNTIME_FORM_ARTIFACT_SCHEMA_VERSION,
};
pub use runtime_form_registry::{
    build_runtime_form_registry, RuntimeFormInstanceRecord, RuntimeFormRecord, RuntimeFormRegistry,
    RUNTIME_FORM_REGISTRY_VERSION,
};
pub use semantic_graph::{
    build_semantic_graph, semantic_graph_json, SemanticGraph, SemanticGraphConsumer,
    SemanticGraphContext, SemanticGraphEdge, SemanticGraphEdgeKind, SemanticGraphNode,
    SemanticGraphNodeKind, SemanticGraphProvenance, SemanticGraphProvider,
    SEMANTIC_GRAPH_SCHEMA_VERSION,
};
pub use semantic_id::{
    ComponentInstanceId, ComponentInvocationId, ComponentRootId, ComponentStructuralRegionId,
    ConsumerId, ContextDeclarationCandidateId, ContextId, DirtyTrackingPlanId, EffectId,
    EffectStatementId, FieldBindingId, FieldDependencyId, FieldId, FieldResetOperationId,
    FieldTrackingId, FormDeclarationCandidateId, FormFieldBindingCandidateId,
    FormFieldDeclarationCandidateId, FormFieldDirtySlotId, FormFieldTouchedSlotId,
    FormFieldValidationSlotId, FormFieldValueSlotId, FormId, FormInstanceId, FormOwnershipGraphId,
    FormSubmissionStateSlotId, FormValidationAggregateSlotId, ProviderId, ResetPlanId, SemanticId,
    SemanticOwner, SerializationPlanId, SlotBindingId, SlotContentFragmentId,
    SlotDeclarationCandidateId, SlotId, SlotOutletId, SubmissionDeclarationCandidateId,
    SubmissionHostCandidateId, SubmissionHostId, SubmissionPlanId, TemplatePositionId,
    TouchedTrackingPlanId, ValidationDependencyCycleId, ValidationGraphId, ValidationPlanId,
    ValidationRuleCandidateId, ValidationRuleId,
};
pub use semantic_provenance::SourceProvenance;
pub use semantic_reference::{SemanticReference, SemanticReferenceKind};
pub use semantic_type::{
    boundary_compatibility, dom_binding_contract, infer_serializable_value_type, is_assignable,
    is_state_initializer_assignable, operator_result_type, semantic_type_text,
    serialization_compatibility, state_initializer_value_type, BoundaryCompatibility,
    BuiltinTypeAuthority, ComputedValueType, DomBindingContract, DomBindingKind,
    EffectCompatibility, EffectOperationClassification, EffectStatementTypeRecord,
    ExecutionBoundary, ObjectType, ResolvedDeclaredSemanticType, ResourceExecutionBoundary,
    ResourceType, SemanticOperator, SemanticType, SemanticTypeAlias, SemanticTypeAssignment,
    SemanticTypeId, SemanticTypeModel, SemanticTypeStatus, SerializationCompatibility,
    TypeDiagnosticCode, TypeDiagnosticFamily,
};
pub use slot::{collect_slot_entities, SlotEntity};
pub use slot_binding::{
    collect_slot_bindings, SlotBinding, SlotBindingRegistry, SlotBindingStatus,
};
pub use slot_content::{
    collect_slot_composition, SlotCompositionRegistry, SlotContentFragment,
    SlotContentFragmentStatus, SlotContentFragmentViolation, SlotOutlet, SlotOutletStatus,
    SlotOutletViolation,
};
pub use state_instance_storage::{
    build_state_instance_storage_registry, validate_state_instance_storage_registry,
    StateInstanceStorageRecord, StateInstanceStorageRegistry,
    STATE_INSTANCE_STORAGE_REGISTRY_VERSION,
};
pub use summarize::summarize_source;
pub use symbol_table::{
    build_symbol_table, ModuleSymbol, ModuleSymbolTable, SymbolDiagnostic, SymbolKind, SymbolTable,
};
pub use template_graph::{
    build_template_graph, AttributeValue, ConditionalNode, ElementNode, FragmentNode, ListNode,
    TemplateAttribute, TemplateChild, TemplateGraph, TemplateNode, TemplateNodeId,
};
pub use template_manifest::{
    build_template_manifest, build_template_manifest_from_asm, template_manifest_json,
    validate_template_manifest, ManifestAction, ManifestBindingTarget, ManifestComponent,
    ManifestEvent, ManifestEventKind, ManifestFormBinding, ManifestFormHost, ManifestNode,
    ManifestOperation, ManifestOrdinaryBinding, ManifestOrdinaryEvent, ManifestOrdinaryTarget,
    ManifestTemplate, TemplateManifest, TEMPLATE_MANIFEST_SCHEMA_VERSION,
};
pub use template_semantics::{
    build_template_semantic_entities, TemplateSemanticEntity, TemplateSemanticKind,
    TemplateSemanticScope,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeMap, path::Path};

    #[test]
    fn summarizes_component_decorator_class_and_render_method() {
        let source = r#"
@component("x-counter")
class Counter extends Component {
  render() {
    return <button>Count</button>;
  }
}
"#;

        let summary = summarize_source("Counter.tsx", source);

        assert_eq!(summary.component_decorators.len(), 1);
        assert_eq!(
            summary.component_decorators[0].argument.as_deref(),
            Some("x-counter")
        );
        assert_eq!(summary.class_declarations.len(), 1);
        assert_eq!(summary.class_declarations[0].name, "Counter");
        assert_eq!(summary.render_methods.len(), 1);
        assert!(summary.has_tsx_like_syntax);
    }

    #[test]
    fn emits_diagnostics_for_empty_source() {
        let summary = summarize_source("Empty.tsx", "");
        assert!(summary
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "EZ0001"));
    }

    #[test]
    fn fixture_0001_source_summary_explain_text_matches_expected() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("fixtures/0001-source-summary");

        let input_path = fixture_root.join("input/Counter.tsx");
        let expected_path = fixture_root.join("expected/explain.txt");

        let source = std::fs::read_to_string(&input_path).expect("failed to read fixture input");
        let expected = std::fs::read_to_string(&expected_path)
            .expect("failed to read expected explain output");

        let summary = summarize_source("fixtures/0001-source-summary/input/Counter.tsx", &source);

        let actual = explain_text(&summary);

        assert_eq!(actual, expected);
    }

    #[test]
    fn fixture_0001_source_summary_explain_json_matches_expected() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("fixtures/0001-source-summary");

        let input_path = fixture_root.join("input/Counter.tsx");
        let expected_path = fixture_root.join("expected/explain.json");

        let source = std::fs::read_to_string(&input_path).expect("failed to read fixture input");
        let expected = std::fs::read_to_string(&expected_path)
            .expect("failed to read expected JSON explain output");

        let summary = summarize_source("fixtures/0001-source-summary/input/Counter.tsx", &source);

        let actual = explain_json(&summary);

        let actual_json: serde_json::Value =
            serde_json::from_str(&actual).expect("actual explain JSON is invalid");
        let expected_json: serde_json::Value =
            serde_json::from_str(&expected).expect("expected explain JSON fixture is invalid");

        assert_eq!(actual_json, expected_json);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn builds_component_graph_from_parsed_counter() {
        let source = include_str!("../../../fixtures/0001-source-summary/input/Counter.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0001-source-summary/input/Counter.tsx", source);

        let graph = build_component_graph(&parsed);

        assert!(graph.diagnostics.is_empty());

        let component = graph.components.first().expect("expected component");

        assert_eq!(component.class_name, "Counter");
        assert_eq!(component.id.as_str(), "component:x-counter");
        assert_eq!(component.owner, SemanticOwner::Application);
        assert_eq!(component.element_name.as_deref(), Some("x-counter"));
        assert_eq!(component.route_path.as_deref(), Some("/counter"));

        assert_eq!(component.state_fields.len(), 1);
        assert_eq!(component.state_fields[0].name, "count");
        assert_eq!(
            component.state_fields[0].id.as_str(),
            "component:x-counter/state:count"
        );
        assert_eq!(
            component.state_fields[0].owner,
            SemanticOwner::entity(component.id.clone())
        );
        assert_eq!(
            component.state_fields[0].initial_value,
            Some(SerializableValue::Number("0".to_string()))
        );

        let method_names = component
            .methods
            .iter()
            .map(|method| method.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(method_names, vec!["increment", "render"]);
        assert_eq!(
            component.methods[0].id.as_str(),
            "component:x-counter/method:increment"
        );
        assert_eq!(
            component.methods[0].owner,
            SemanticOwner::entity(component.id.clone())
        );
        assert_eq!(
            component.actions,
            vec![ComponentAction {
                id: SemanticId::component(Some("x-counter"), "Counter").action("increment", 0),
                owner: SemanticOwner::entity(
                    SemanticId::component(Some("x-counter"), "Counter").method("increment"),
                ),
                method: "increment".to_string(),
                operation: StateOperation::AddAssign(SerializableValue::Number("1".to_string())),
                field: "count".to_string(),
            }]
        );

        let render = component.render.as_ref().expect("expected render model");

        assert_eq!(render.root_element.as_deref(), Some("button"));
        assert_eq!(render.attributes.len(), 1);
        assert_eq!(render.attributes[0].name, "onClick");
        assert!(matches!(
            render.attributes[0].value,
            RenderAttributeValue::Expression(_)
        ));
        assert_eq!(render.bindings, vec!["this.count"]);
        assert_eq!(render.root_span.expect("expected root span").line, 12);
        assert_eq!(render.root_span.expect("expected root span").column, 7);
        assert_eq!(render.event_handlers.len(), 1);
        assert_eq!(
            render.event_handlers[0].id.as_str(),
            "component:x-counter/event:click:0"
        );
        assert_eq!(
            render.event_handlers[0].owner,
            SemanticOwner::entity(component.id.template())
        );
        assert_eq!(render.event_handlers[0].event, "click");
        assert_eq!(render.event_handlers[0].handler, "this.increment");
        assert_eq!(render.event_handlers[0].span.line, 12);
        assert_eq!(render.event_handlers[0].span.column, 15);
        assert_eq!(render.children.len(), 2);

        let RenderChild::Text { value, span } = &render.children[0] else {
            panic!("expected text child");
        };
        assert_eq!(value, "Count:");
        assert_eq!(span.line, 13);
        assert_eq!(span.column, 9);

        let RenderChild::Binding { expression, span } = &render.children[1] else {
            panic!("expected binding child");
        };
        assert_eq!(expression, "this.count");
        assert_eq!(span.line, 13);
        assert_eq!(span.column, 16);

        assert_eq!(graph.references.len(), 2);
        assert_eq!(graph.references[0].kind, SemanticReferenceKind::ActionState);
        assert_eq!(
            graph.references[0].source,
            SemanticId::component(Some("x-counter"), "Counter").action("increment", 0)
        );
        assert_eq!(
            graph.references[0].target,
            SemanticId::component(Some("x-counter"), "Counter").state_field("count")
        );
        assert_eq!(
            graph.references[0].provenance.path,
            Path::new("fixtures/0001-source-summary/input/Counter.tsx")
        );
        assert_eq!(graph.references[0].provenance.span.line, 7);

        assert_eq!(graph.references[1].kind, SemanticReferenceKind::EventMethod);
        assert_eq!(
            graph.references[1].source,
            SemanticId::component(Some("x-counter"), "Counter").event_handler("click", 0)
        );
        assert_eq!(
            graph.references[1].target,
            SemanticId::component(Some("x-counter"), "Counter").method("increment")
        );
        assert_eq!(
            graph.references[1].provenance.path,
            Path::new("fixtures/0001-source-summary/input/Counter.tsx")
        );
        assert_eq!(graph.references[1].provenance.span.line, 12);

        assert_eq!(graph.provenance[&component.id].span.line, 1);
        assert_eq!(graph.provenance[&component.state_fields[0].id].span.line, 4);
        assert_eq!(graph.provenance[&component.methods[0].id].span.line, 6);
        assert_eq!(graph.provenance[&component.actions[0].id].span.line, 7);
        assert_eq!(graph.provenance[&component.id.template()].span.line, 10);
        assert_eq!(graph.provenance[&render.event_handlers[0].id].span.line, 12);
    }

    #[test]
    fn lowers_method_parameters_into_canonical_method_metadata() {
        let parsed = ezc_parser::parse_file(
            "src/Parameters.tsx",
            r#"
@component("x-parameters")
class Parameters extends Component {
  save(title: string, retries?: number) {}
}
"#,
        );

        let graph = fold_component_graph(&build_component_graph_for_module(&parsed));
        let method = &graph.components[0].methods[0];

        assert_eq!(method.name, "save");
        assert_eq!(
            method
                .parameters
                .iter()
                .map(|parameter| parameter.name.as_str())
                .collect::<Vec<_>>(),
            vec!["title", "retries"]
        );
        assert_eq!(method.parameters[0].span.line, 4);
        assert_eq!(method.parameters[1].span.line, 4);
    }

    #[test]
    fn assembles_application_semantic_model_from_existing_graphs() {
        let source = include_str!("../../../fixtures/0001-source-summary/input/Counter.tsx");
        let parsed =
            ezc_parser::parse_file("fixtures/0001-source-summary/input/Counter.tsx", source);

        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];

        assert!(asm.diagnostics.is_empty());
        assert_eq!(asm.templates.len(), 1);
        assert!(asm.template_entities.len() >= 3);
        assert_eq!(asm.references.len(), 4);
        assert_eq!(asm.ownership.len(), asm.provenance.len());
        assert_eq!(asm.ownership[&component.id], SemanticOwner::Application);
        assert_eq!(
            asm.ownership[&component.state_fields[0].id],
            SemanticOwner::entity(component.id.clone())
        );
        assert_eq!(
            asm.ownership[&component.actions[0].id],
            SemanticOwner::entity(component.methods[0].id.clone())
        );
        assert_eq!(
            asm.ownership[&asm.templates[0].id],
            SemanticOwner::entity(component.id.clone())
        );
        assert_eq!(asm.provenance[&asm.templates[0].id].span.line, 10);

        assert!(matches!(
            asm.entity(&component.id),
            Some(SemanticEntity::Component(_))
        ));
        assert!(matches!(
            asm.entity(&component.state_fields[0].id),
            Some(SemanticEntity::StateField(_))
        ));
        assert_eq!(asm.component(&component.id), Some(component));
        assert_eq!(asm.template(&asm.templates[0].id), Some(&asm.templates[0]));
        assert_eq!(
            asm.owner(&component.actions[0].id),
            Some(&component.actions[0].owner)
        );
        assert_eq!(asm.provenance(&asm.templates[0].id).unwrap().span.line, 10);
        let template_binding = asm
            .template_entities
            .iter()
            .find(|entity| entity.kind == TemplateSemanticKind::Binding)
            .expect("template binding entity");
        assert!(matches!(
            asm.entity(&template_binding.id),
            Some(SemanticEntity::TemplateEntity(_))
        ));
        assert_eq!(
            asm.template_entities_for(&asm.templates[0].id).len(),
            asm.template_entities.len()
        );
        assert_eq!(asm.references_from(&component.actions[0].id).len(), 1);
        assert_eq!(asm.references_to(&component.state_fields[0].id).len(), 2);
        assert_eq!(asm.references_to(&component.methods[0].id).len(), 2);
        assert!(validate_application_semantic_model(&asm).is_empty());

        let mut invalid = asm.clone();
        invalid.provenance.remove(&component.actions[0].id);
        let diagnostics = validate_application_semantic_model(&invalid);
        let codes = diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>();
        assert!(codes.contains(&"EZASM1002"));
        assert!(codes.contains(&"EZASM1006"));

        let mut invalid_type = asm.clone();
        let state_id = component.state_fields[0].id.clone();
        invalid_type
            .semantic_types
            .assignments
            .get_mut(&state_id)
            .expect("state type")
            .id = SemanticTypeId::for_subject(&component.id);
        let diagnostics = validate_application_semantic_model(&invalid_type);
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "EZASM1102"));

        let dependencies = DependencyAnalysisPass.analyze(&asm);
        assert_eq!(dependencies.dependencies[&component.actions[0].id].len(), 1);
        assert_eq!(
            dependencies.dependents[&component.state_fields[0].id].len(),
            2
        );
    }

    #[test]
    fn carries_declared_state_types_into_component_and_asm_data() {
        let parsed = ezc_parser::parse_file(
            "src/Panel.tsx",
            r#"
@component("x-panel")
class Panel extends Component {
  count: number = state(0);
}
"#,
        );

        let graph = fold_component_graph(&build_component_graph_for_module(&parsed));
        let graph_type = graph.components[0].state_fields[0]
            .declared_type
            .as_ref()
            .expect("declared state type");

        assert_eq!(graph_type.text, "number");
        assert_eq!(graph_type.provenance.path, Path::new("src/Panel.tsx"));
        assert_eq!(graph_type.provenance.span.line, 4);
        assert_eq!(graph_type.provenance.span.column, 8);
        assert_eq!(graph_type.kind, Some(DeclaredStateTypeKind::Number));

        let asm = build_application_semantic_model(&parsed);
        assert_eq!(
            asm.components[0].state_fields[0].declared_type,
            Some(graph_type.clone())
        );
    }

    #[test]
    fn lowers_and_evaluates_constant_arithmetic_state_initializers() {
        let parsed = ezc_parser::parse_file(
            "src/ArithmeticState.tsx",
            r#"
@component("x-arithmetic-state")
class ArithmeticState extends Component {
  total: number = state((1 + 2) * 3);
  difference: number = state(10 - 3);
  quotient: number = state(10 / 2);
  remainder: number = state(10 % 3);

  render() {
    return <output>{this.total}</output>;
  }
}
"#,
        );
        let graph = fold_component_graph(&build_component_graph_for_module(&parsed));
        let field = &graph.components[0].state_fields[0];

        assert_eq!(
            field
                .initial_expression
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("((1 + 2) * 3)")
        );
        assert_eq!(
            field.initial_value,
            Some(SerializableValue::Number("9".to_string()))
        );
        assert_eq!(
            graph.components[0].state_fields[1].initial_value,
            Some(SerializableValue::Number("7".to_string()))
        );
        assert_eq!(
            graph.components[0].state_fields[2].initial_value,
            Some(SerializableValue::Number("5".to_string()))
        );
        assert_eq!(
            graph.components[0].state_fields[3].initial_value,
            Some(SerializableValue::Number("1".to_string()))
        );
        assert!(graph.diagnostics.is_empty());
    }

    #[test]
    fn reports_invalid_constant_arithmetic_state_initializers() {
        let parsed = ezc_parser::parse_file(
            "src/ArithmeticState.tsx",
            r#"
@component("x-arithmetic-state")
class ArithmeticState extends Component {
  total: number = state(10 / 0);
}
"#,
        );
        let graph = fold_component_graph(&build_component_graph_for_module(&parsed));
        let diagnostic = graph
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "EZC1022")
            .expect("arithmetic diagnostic");

        assert!(diagnostic.message.contains("division or remainder by zero"));
        assert_eq!(
            diagnostic
                .provenance
                .as_ref()
                .map(|provenance| provenance.span.line),
            Some(4)
        );
    }

    #[test]
    fn lowers_and_evaluates_constant_comparison_state_initializers() {
        let parsed = ezc_parser::parse_file(
            "src/ComparisonState.tsx",
            r#"
@component("x-comparison-state")
class ComparisonState extends Component {
  equal: boolean = state(3 === 3);
  notEqual: boolean = state(3 !== 4);
  lessThan: boolean = state(2 < 3);
  lessThanOrEqual: boolean = state(3 <= 3);
  greaterThan: boolean = state(4 > 3);
  ready: boolean = state(((1 + 2) * 3) >= 9);

  render() {
    return <output>{this.ready}</output>;
  }
}
"#,
        );
        let graph = fold_component_graph(&build_component_graph_for_module(&parsed));
        let fields = &graph.components[0].state_fields;

        assert_eq!(
            fields[0]
                .initial_expression
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("(3 === 3)")
        );
        for field in fields {
            assert_eq!(field.initial_value, Some(SerializableValue::Boolean(true)));
        }
        assert_eq!(
            fields[5]
                .initial_expression
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("(((1 + 2) * 3) >= 9)")
        );
        assert!(graph.diagnostics.is_empty());
    }

    #[test]
    fn reports_invalid_constant_comparison_state_initializers() {
        let parsed = ezc_parser::parse_file(
            "src/ComparisonState.tsx",
            r#"
@component("x-comparison-state")
class ComparisonState extends Component {
  ready: boolean = state((10 / 0) >= 1);
}
"#,
        );
        let graph = fold_component_graph(&build_component_graph_for_module(&parsed));
        let diagnostic = graph
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "EZC1023")
            .expect("comparison diagnostic");

        assert!(diagnostic.message.contains("division or remainder by zero"));
        assert_eq!(
            diagnostic
                .provenance
                .as_ref()
                .map(|provenance| provenance.span.line),
            Some(4)
        );
    }

    #[test]
    fn lowers_and_evaluates_constant_logical_state_initializers() {
        let parsed = ezc_parser::parse_file(
            "src/LogicalState.tsx",
            r#"
@component("x-logical-state")
class LogicalState extends Component {
  both: boolean = state((1 < 2) && (3 >= 3));
  either: boolean = state(false || (10 !== 4));
  shortAnd: boolean = state(false && ((10 / 0) > 1));
  shortOr: boolean = state(true || ((10 / 0) > 1));

  render() {
    return <output>{this.both}</output>;
  }
}
"#,
        );
        let graph = fold_component_graph(&build_component_graph_for_module(&parsed));
        let fields = &graph.components[0].state_fields;

        assert_eq!(
            fields[0]
                .initial_expression
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("((1 < 2) && (3 >= 3))")
        );
        assert_eq!(
            fields
                .iter()
                .map(|field| field.initial_value.clone())
                .collect::<Vec<_>>(),
            vec![
                Some(SerializableValue::Boolean(true)),
                Some(SerializableValue::Boolean(true)),
                Some(SerializableValue::Boolean(false)),
                Some(SerializableValue::Boolean(true)),
            ]
        );
        assert!(graph.diagnostics.is_empty());
    }

    #[test]
    fn reports_evaluated_invalid_constant_logical_state_initializers() {
        let parsed = ezc_parser::parse_file(
            "src/LogicalState.tsx",
            r#"
@component("x-logical-state")
class LogicalState extends Component {
  ready: boolean = state(true && ((10 / 0) > 1));
}
"#,
        );
        let graph = fold_component_graph(&build_component_graph_for_module(&parsed));
        let diagnostic = graph
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "EZC1024")
            .expect("logical diagnostic");

        assert!(diagnostic.message.contains("division or remainder by zero"));
        assert_eq!(
            diagnostic
                .provenance
                .as_ref()
                .map(|provenance| provenance.span.line),
            Some(4)
        );
    }

    #[test]
    fn lowers_and_evaluates_constant_nullish_state_initializers() {
        let parsed = ezc_parser::parse_file(
            "src/NullishState.tsx",
            r#"
@component("x-nullish-state")
class NullishState extends Component {
  label: string = state(null ?? "fallback");
  total: number = state(5 ?? (10 / 0));

  render() {
    return <output>{this.label}</output>;
  }
}
"#,
        );
        let graph = fold_component_graph(&build_component_graph_for_module(&parsed));
        let fields = &graph.components[0].state_fields;

        assert_eq!(
            fields[0]
                .initial_expression
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("(null ?? \"fallback\")")
        );
        assert_eq!(
            fields[0].initial_value,
            Some(SerializableValue::String("fallback".to_string()))
        );
        assert_eq!(
            fields[1].initial_value,
            Some(SerializableValue::Number("5".to_string()))
        );
        assert!(graph.diagnostics.is_empty());
    }

    #[test]
    fn lowers_and_evaluates_constant_unary_state_initializers() {
        let parsed = ezc_parser::parse_file(
            "src/UnaryState.tsx",
            r#"
@component("x-unary-state")
class UnaryState extends Component {
  negated: boolean = state(!(1 < 2));
  signed: number = state(-(1 + 2));
  render() { return <output>{this.signed}</output>; }
}
"#,
        );
        let graph = fold_component_graph(&build_component_graph_for_module(&parsed));
        assert_eq!(
            graph.components[0].state_fields[0].initial_value,
            Some(SerializableValue::Boolean(false))
        );
        assert_eq!(
            graph.components[0].state_fields[1].initial_value,
            Some(SerializableValue::Number("-3".to_string()))
        );
    }

    #[test]
    fn reports_reached_invalid_constant_nullish_state_initializers() {
        let parsed = ezc_parser::parse_file(
            "src/NullishState.tsx",
            r#"
@component("x-nullish-state")
class NullishState extends Component {
  total: number = state(null ?? (10 / 0));
}
"#,
        );
        let graph = fold_component_graph(&build_component_graph_for_module(&parsed));
        let diagnostic = graph
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "EZC1025")
            .expect("nullish diagnostic");
        assert!(diagnostic.message.contains("division or remainder by zero"));
    }

    #[test]
    fn classifies_exact_primitive_declared_state_types() {
        let source =
            include_str!("../../../fixtures/0025-typed-state-annotations/input/TypedState.tsx");
        let parsed = ezc_parser::parse_file(
            "fixtures/0025-typed-state-annotations/input/TypedState.tsx",
            source,
        );
        let graph = build_component_graph_for_module(&parsed);
        let kinds = graph.components[0]
            .state_fields
            .iter()
            .map(|field| {
                (
                    field.name.as_str(),
                    field
                        .declared_type
                        .as_ref()
                        .and_then(|declared_type| declared_type.kind),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            kinds,
            vec![
                ("count", Some(DeclaredStateTypeKind::Number)),
                ("status", None),
                ("title", Some(DeclaredStateTypeKind::String)),
                ("enabled", Some(DeclaredStateTypeKind::Boolean)),
                ("empty", Some(DeclaredStateTypeKind::Null)),
            ]
        );
    }

    #[test]
    fn reports_primitive_declared_state_initializer_mismatches() {
        let source = include_str!(
            "../../../fixtures/0027-declared-state-type-diagnostics/input/InvalidTypedState.tsx"
        );
        let parsed = ezc_parser::parse_file(
            "fixtures/0027-declared-state-type-diagnostics/input/InvalidTypedState.tsx",
            source,
        );
        let graph = build_component_graph_for_module(&parsed);
        let folded = ConstantFoldingPass.transform(
            &build_application_semantic_model_from_component_graph(&graph),
        );
        let diagnostics = folded
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.code.as_str(), diagnostic.message.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(diagnostics.len(), 6);
        assert!(diagnostics.iter().all(|(code, _)| *code == "EZC1016"));

        let provenance = folded.diagnostics[0]
            .provenance
            .as_ref()
            .expect("mismatch diagnostic provenance");
        assert_eq!(
            provenance.path,
            Path::new("fixtures/0027-declared-state-type-diagnostics/input/InvalidTypedState.tsx")
        );
        assert_eq!(provenance.span.line, 3);
        assert_eq!(provenance.span.column, 8);
    }

    #[test]
    fn reports_primitive_declared_state_action_assignment_mismatches() {
        let source = include_str!(
            "../../../fixtures/0028-primitive-action-type-diagnostics/input/InvalidTypedActions.tsx"
        );
        let parsed = ezc_parser::parse_file(
            "fixtures/0028-primitive-action-type-diagnostics/input/InvalidTypedActions.tsx",
            source,
        );
        let graph = build_component_graph_for_module(&parsed);
        let folded = ConstantFoldingPass.transform(
            &build_application_semantic_model_from_component_graph(&graph),
        );
        let diagnostics = folded
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "EZC1017")
            .map(|diagnostic| (diagnostic.code.as_str(), diagnostic.message.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(diagnostics.len(), 6);
        assert!(diagnostics.iter().all(|(code, _)| *code == "EZC1017"));
        assert!(diagnostics.iter().any(|(_, message)| {
            message.contains("state field `status`") && message.contains("assigns `number`")
        }));
        assert!(diagnostics.iter().any(|(_, message)| {
            message.contains("state field `collection`") && message.contains("assigns `tuple`")
        }));

        let provenance = folded
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "EZC1017")
            .expect("action mismatch diagnostic")
            .provenance
            .as_ref()
            .expect("action mismatch diagnostic provenance");
        assert_eq!(
            provenance.path,
            Path::new(
                "fixtures/0028-primitive-action-type-diagnostics/input/InvalidTypedActions.tsx"
            )
        );
        assert_eq!(provenance.span.line, 11);
        assert_eq!(provenance.span.column, 5);
    }

    #[test]
    fn reports_non_boolean_primitive_toggle_actions() {
        let source = include_str!(
            "../../../fixtures/0029-primitive-toggle-type-diagnostics/input/InvalidTypedToggles.tsx"
        );
        let parsed = ezc_parser::parse_file(
            "fixtures/0029-primitive-toggle-type-diagnostics/input/InvalidTypedToggles.tsx",
            source,
        );
        let graph = build_component_graph_for_module(&parsed);
        let folded = ConstantFoldingPass.transform(
            &build_application_semantic_model_from_component_graph(&graph),
        );
        let diagnostics = folded
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.code.as_str(), diagnostic.message.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(diagnostics.len(), 4);
        assert!(diagnostics.iter().all(|(code, _)| *code == "EZC1018"));

        let provenance = folded.diagnostics[0]
            .provenance
            .as_ref()
            .expect("toggle diagnostic provenance");
        assert_eq!(
            provenance.path,
            Path::new(
                "fixtures/0029-primitive-toggle-type-diagnostics/input/InvalidTypedToggles.tsx"
            )
        );
        assert_eq!(provenance.span.line, 10);
        assert_eq!(provenance.span.column, 5);
    }

    #[test]
    fn reports_non_numeric_primitive_increment_and_decrement_actions() {
        let source = include_str!(
            "../../../fixtures/0030-primitive-numeric-action-type-diagnostics/input/InvalidTypedNumericActions.tsx"
        );
        let parsed = ezc_parser::parse_file(
            "fixtures/0030-primitive-numeric-action-type-diagnostics/input/InvalidTypedNumericActions.tsx",
            source,
        );
        let graph = build_component_graph_for_module(&parsed);
        let folded = ConstantFoldingPass.transform(
            &build_application_semantic_model_from_component_graph(&graph),
        );
        let diagnostics = folded
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.code.as_str(), diagnostic.message.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(diagnostics.len(), 4);
        assert!(diagnostics.iter().all(|(code, _)| *code == "EZC1019"));

        let provenance = folded.diagnostics[0]
            .provenance
            .as_ref()
            .expect("numeric action diagnostic provenance");
        assert_eq!(
            provenance.path,
            Path::new(
                "fixtures/0030-primitive-numeric-action-type-diagnostics/input/InvalidTypedNumericActions.tsx"
            )
        );
        assert_eq!(provenance.span.line, 10);
        assert_eq!(provenance.span.column, 5);
    }

    #[test]
    fn reports_compound_numeric_action_target_and_operand_mismatches() {
        let source = include_str!(
            "../../../fixtures/0031-primitive-compound-action-type-diagnostics/input/InvalidTypedCompoundActions.tsx"
        );
        let parsed = ezc_parser::parse_file(
            "fixtures/0031-primitive-compound-action-type-diagnostics/input/InvalidTypedCompoundActions.tsx",
            source,
        );
        let graph = build_component_graph_for_module(&parsed);
        let folded = ConstantFoldingPass.transform(
            &build_application_semantic_model_from_component_graph(&graph),
        );
        let diagnostics = folded
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.code.as_str(), diagnostic.message.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(diagnostics.len(), 7);
        assert_eq!(
            diagnostics
                .iter()
                .filter(|(code, _)| *code == "EZC1020")
                .count(),
            3
        );
        assert_eq!(
            diagnostics
                .iter()
                .filter(|(code, _)| *code == "EZC1021")
                .count(),
            4
        );

        let provenance = folded.diagnostics[0]
            .provenance
            .as_ref()
            .expect("compound action diagnostic provenance");
        assert_eq!(
            provenance.path,
            Path::new(
                "fixtures/0031-primitive-compound-action-type-diagnostics/input/InvalidTypedCompoundActions.tsx"
            )
        );
        assert_eq!(provenance.span.line, 9);
        assert_eq!(provenance.span.column, 5);
    }

    #[test]
    fn assembles_application_semantic_model_from_multiple_files() {
        let unit = CompilationUnit::parse_sources([
            (
                "src/Zeta.tsx",
                r#"
@component("x-zeta")
class Zeta extends Component {
  render() {
    return <div>Zeta</div>;
  }
}
"#,
            ),
            (
                "src/Alpha.tsx",
                r#"
@component("x-alpha")
class Alpha extends Component {
  render() {
    return <div>Alpha</div>;
  }
}
"#,
            ),
        ]);

        let asm = build_application_semantic_model_for_unit(&unit);

        assert_eq!(
            unit.files()
                .iter()
                .map(|file| file.path.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            vec!["src/Alpha.tsx", "src/Zeta.tsx"]
        );
        assert_eq!(
            asm.components
                .iter()
                .map(|component| component.id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "module:src/Alpha.tsx/component:x-alpha",
                "module:src/Zeta.tsx/component:x-zeta"
            ]
        );
        assert!(asm.diagnostics.is_empty());
        assert!(validate_application_semantic_model(&asm).is_empty());
    }

    #[test]
    fn component_graph_reports_semantic_errors() {
        let source =
            include_str!("../../../fixtures/0003-semantic-errors/input/BrokenSemantics.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0003-semantic-errors/input/BrokenSemantics.tsx",
            source,
        );

        let graph = build_component_graph(&parsed);

        let codes = graph
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>();

        assert!(codes.contains(&"EZC1001"));
        assert!(codes.contains(&"EZC1003"));
        assert!(codes.contains(&"EZC1004"));
        assert!(graph.references.is_empty());
    }

    #[test]
    fn component_graph_reports_unsupported_event_errors() {
        let source = r#"
@component("x-counter")
class Counter extends Component {
  count = state(0);

  increment() {
    this.count++;
  }

  render() {
    return <button onMouseover={() => this.increment()}>Count: {this.count}</button>;
  }
}
"#;

        let parsed = ezc_parser::parse_file("UnsupportedEvent.tsx", source);

        let graph = build_component_graph(&parsed);

        assert!(graph
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "EZC1005"));
    }

    #[test]
    fn component_graph_reports_duplicate_event_errors() {
        let parsed = ezc_parser::ParsedFile {
            path: "DuplicateEvent.tsx".into(),
            diagnostics: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            type_aliases: Vec::new(),
            local_type_bindings: Vec::new(),
            local_value_bindings: Vec::new(),
            classes: vec![ezc_parser::ParsedClass {
                name: "DuplicateEvent".to_string(),
                span: test_span(),
                heritage: None,
                decorators: vec![ezc_parser::ParsedDecorator {
                    name: "component".to_string(),
                    is_invoked: true,
                    argument: Some("x-duplicate-event".to_string()),
                    argument_count: 1,
                    argument_spans: vec![test_span()],
                    static_member_argument: None,
                    this_member_argument: None,
                    validation_rule_expression: None,
                    span: test_span(),
                }],
                properties: Vec::new(),
                methods: vec![ezc_parser::ParsedMethod {
                    name: "render".to_string(),
                    span: test_span(),
                    decorators: Vec::new(),
                    is_getter: false,
                    is_setter: false,
                    is_async: false,
                    is_static: false,
                    jsx_roots: vec![ezc_parser::ParsedJsxNode::Element(
                        ezc_parser::ParsedJsxElement {
                            name: "button".to_string(),
                            name_span: test_span(),
                            span: test_span(),
                            attributes: Vec::new(),
                            event_handlers: vec![
                                ezc_parser::ParsedEventHandler {
                                    event: "click".to_string(),
                                    handler: "this.render".to_string(),
                                    span: test_span(),
                                },
                                ezc_parser::ParsedEventHandler {
                                    event: "click".to_string(),
                                    handler: "this.render".to_string(),
                                    span: test_span(),
                                },
                            ],
                            children: Vec::new(),
                        },
                    )],
                    bindings: Vec::new(),
                    state_updates: Vec::new(),
                    local_variables: Vec::new(),
                    parameters: Vec::new(),
                    return_type_annotation: None,
                    return_values: Vec::new(),
                    computed_expression: None,
                    effect_body: None,
                    calls: Vec::new(),
                }],
            }],
        };

        let graph = build_component_graph(&parsed);

        assert!(graph
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "EZC1006"));
    }

    fn test_span() -> ezc_parser::SourceSpan {
        ezc_parser::SourceSpan {
            start: 0,
            end: 0,
            line: 1,
            column: 1,
        }
    }

    #[test]
    fn builds_increment_action_from_parsed_method_update() {
        let source = include_str!("../../../fixtures/0004-nested-jsx/input/NestedCounter.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0004-nested-jsx/input/NestedCounter.tsx", source);

        let graph = build_component_graph(&parsed);
        let component = graph.components.first().expect("expected component");

        assert_eq!(
            component.actions,
            vec![ComponentAction {
                id: SemanticId::component(Some("x-nested-counter"), "NestedCounter")
                    .action("increment", 0),
                owner: SemanticOwner::entity(
                    SemanticId::component(Some("x-nested-counter"), "NestedCounter")
                        .method("increment"),
                ),
                method: "increment".to_string(),
                operation: StateOperation::Increment,
                field: "count".to_string(),
            }]
        );
    }

    #[test]
    fn builds_decrement_action_from_parsed_method_update() {
        let source =
            include_str!("../../../fixtures/0009-decrement-counter/input/DecrementCounter.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0009-decrement-counter/input/DecrementCounter.tsx",
            source,
        );

        let graph = build_component_graph(&parsed);
        let component = graph.components.first().expect("expected component");

        assert_eq!(
            component.actions,
            vec![ComponentAction {
                id: SemanticId::component(Some("x-decrement-counter"), "DecrementCounter")
                    .action("decrement", 0),
                owner: SemanticOwner::entity(
                    SemanticId::component(Some("x-decrement-counter"), "DecrementCounter")
                        .method("decrement"),
                ),
                method: "decrement".to_string(),
                operation: StateOperation::Decrement,
                field: "count".to_string(),
            }]
        );
    }

    #[test]
    fn builds_add_and_subtract_assign_actions_from_parsed_method_updates() {
        let source =
            include_str!("../../../fixtures/0010-add-subtract-assign/input/StepCounter.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0010-add-subtract-assign/input/StepCounter.tsx",
            source,
        );

        let graph = build_component_graph(&parsed);
        let component = graph.components.first().expect("expected component");

        assert_eq!(
            component.actions,
            vec![
                ComponentAction {
                    id: SemanticId::component(Some("x-step-counter"), "StepCounter")
                        .action("addTwo", 0),
                    owner: SemanticOwner::entity(
                        SemanticId::component(Some("x-step-counter"), "StepCounter")
                            .method("addTwo"),
                    ),
                    method: "addTwo".to_string(),
                    operation: StateOperation::AddAssign(SerializableValue::Number(
                        "2".to_string()
                    )),
                    field: "count".to_string(),
                },
                ComponentAction {
                    id: SemanticId::component(Some("x-step-counter"), "StepCounter")
                        .action("subtractThree", 0),
                    owner: SemanticOwner::entity(
                        SemanticId::component(Some("x-step-counter"), "StepCounter")
                            .method("subtractThree"),
                    ),
                    method: "subtractThree".to_string(),
                    operation: StateOperation::SubtractAssign(SerializableValue::Number(
                        "3".to_string()
                    )),
                    field: "count".to_string(),
                }
            ]
        );
    }

    #[test]
    fn builds_direct_assignment_action_from_parsed_method_update() {
        let source =
            include_str!("../../../fixtures/0011-direct-assignment/input/ResetCounter.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0011-direct-assignment/input/ResetCounter.tsx",
            source,
        );

        let graph = build_component_graph(&parsed);
        let component = graph.components.first().expect("expected component");

        assert_eq!(
            component.actions,
            vec![ComponentAction {
                id: SemanticId::component(Some("x-reset-counter"), "ResetCounter")
                    .action("reset", 0),
                owner: SemanticOwner::entity(
                    SemanticId::component(Some("x-reset-counter"), "ResetCounter").method("reset"),
                ),
                method: "reset".to_string(),
                operation: StateOperation::Assign(SerializableValue::Number("0".to_string())),
                field: "count".to_string(),
            }]
        );
    }

    #[test]
    fn builds_boolean_toggle_action_from_parsed_method_update() {
        let source = include_str!("../../../fixtures/0012-boolean-toggle/input/ToggleFlag.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0012-boolean-toggle/input/ToggleFlag.tsx", source);

        let graph = build_component_graph(&parsed);
        let component = graph.components.first().expect("expected component");

        assert_eq!(
            component.actions,
            vec![ComponentAction {
                id: SemanticId::component(Some("x-toggle-flag"), "ToggleFlag").action("toggle", 0),
                owner: SemanticOwner::entity(
                    SemanticId::component(Some("x-toggle-flag"), "ToggleFlag").method("toggle"),
                ),
                method: "toggle".to_string(),
                operation: StateOperation::Toggle,
                field: "enabled".to_string(),
            }]
        );
    }

    #[test]
    fn builds_multi_step_actions_from_parsed_method_updates_in_source_order() {
        let source =
            include_str!("../../../fixtures/0013-multi-step-action/input/BatchActionCounter.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0013-multi-step-action/input/BatchActionCounter.tsx",
            source,
        );

        let graph = build_component_graph(&parsed);
        let component = graph.components.first().expect("expected component");

        assert_eq!(
            component.actions,
            vec![
                ComponentAction {
                    id:
                        SemanticId::component(Some("x-batch-action-counter"), "BatchActionCounter",)
                            .action("apply", 0),
                    owner:
                        SemanticOwner::entity(
                            SemanticId::component(
                                Some("x-batch-action-counter"),
                                "BatchActionCounter",
                            )
                            .method("apply"),
                        ),
                    method: "apply".to_string(),
                    operation: StateOperation::AddAssign(SerializableValue::Number(
                        "2".to_string()
                    )),
                    field: "count".to_string(),
                },
                ComponentAction {
                    id:
                        SemanticId::component(Some("x-batch-action-counter"), "BatchActionCounter",)
                            .action("apply", 1),
                    owner:
                        SemanticOwner::entity(
                            SemanticId::component(
                                Some("x-batch-action-counter"),
                                "BatchActionCounter",
                            )
                            .method("apply"),
                        ),
                    method: "apply".to_string(),
                    operation: StateOperation::Decrement,
                    field: "count".to_string(),
                },
                ComponentAction {
                    id:
                        SemanticId::component(Some("x-batch-action-counter"), "BatchActionCounter",)
                            .action("apply", 2),
                    owner:
                        SemanticOwner::entity(
                            SemanticId::component(
                                Some("x-batch-action-counter"),
                                "BatchActionCounter",
                            )
                            .method("apply"),
                        ),
                    method: "apply".to_string(),
                    operation: StateOperation::Assign(SerializableValue::Number("8".to_string())),
                    field: "count".to_string(),
                },
                ComponentAction {
                    id:
                        SemanticId::component(Some("x-batch-action-counter"), "BatchActionCounter",)
                            .action("apply", 3),
                    owner:
                        SemanticOwner::entity(
                            SemanticId::component(
                                Some("x-batch-action-counter"),
                                "BatchActionCounter",
                            )
                            .method("apply"),
                        ),
                    method: "apply".to_string(),
                    operation: StateOperation::Increment,
                    field: "count".to_string(),
                },
                ComponentAction {
                    id:
                        SemanticId::component(Some("x-batch-action-counter"), "BatchActionCounter",)
                            .action("apply", 4),
                    owner:
                        SemanticOwner::entity(
                            SemanticId::component(
                                Some("x-batch-action-counter"),
                                "BatchActionCounter",
                            )
                            .method("apply"),
                        ),
                    method: "apply".to_string(),
                    operation: StateOperation::Toggle,
                    field: "enabled".to_string(),
                }
            ]
        );
    }

    #[test]
    fn generates_static_html_from_template_graph() {
        let source = include_str!("../../../fixtures/0001-source-summary/input/Counter.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0001-source-summary/input/Counter.tsx", source);

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let html = generate_static_html(&template_graph);

        assert_eq!(
            html,
            "<button data-ez-node=\"n0\" data-ez-on-click=\"this.increment\" data-ez-bindings=\"this.count\">Count:<!-- ez-binding:n1:this.count -->0</button>\n"
        );
    }

    #[test]
    fn preserves_string_state_literals_in_template_outputs() {
        let source = include_str!("../../../fixtures/0006-string-state/input/StringGreeting.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0006-string-state/input/StringGreeting.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        let component = component_graph
            .components
            .first()
            .expect("expected component");

        assert_eq!(
            component.state_fields[0].initial_value,
            Some(SerializableValue::String("Austin & <Zero>".to_string()))
        );

        let template_graph = build_template_graph(&component_graph);
        let html = generate_static_html(&template_graph);

        assert_eq!(
            html,
            "<p data-ez-node=\"n0\" data-ez-bindings=\"this.name\">Name:<!-- ez-binding:n1:this.name -->Austin &amp; &lt;Zero&gt;</p>\n"
        );

        let manifest = build_template_manifest(&component_graph, &template_graph);

        assert_eq!(
            manifest.components[0].template.nodes,
            vec![
                ManifestNode::Element {
                    id: "n0".to_string(),
                    tag: "p".to_string(),
                },
                ManifestNode::Binding {
                    id: "n1".to_string(),
                    expression: "this.name".to_string(),
                    initial_value: Some(SerializableValue::String("Austin & <Zero>".to_string())),
                    target: None,
                    element: None,
                    attribute: None,
                }
            ]
        );

        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        assert_eq!(
            manifest_value["components"][0]["template"]["nodes"][1]["initial_value"],
            serde_json::json!("Austin & <Zero>")
        );
    }

    #[test]
    fn preserves_static_jsx_attributes_in_template_outputs() {
        let source =
            include_str!("../../../fixtures/0014-static-attributes/input/StaticAttributePanel.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0014-static-attributes/input/StaticAttributePanel.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        assert!(component_graph.diagnostics.is_empty());

        let template_graph = build_template_graph(&component_graph);
        let root = template_graph.templates[0]
            .root
            .as_ref()
            .expect("expected root");

        assert_eq!(root.attributes.len(), 3);
        assert_eq!(root.attributes[0].name, "id");
        assert_eq!(
            root.attributes[0].value,
            AttributeValue::Static("panel-root".to_string())
        );
        assert_eq!(root.attributes[1].name, "aria-label");
        assert_eq!(
            root.attributes[1].value,
            AttributeValue::Static("Status \"Panel\"".to_string())
        );
        assert_eq!(root.attributes[2].name, "hidden");
        assert_eq!(root.attributes[2].value, AttributeValue::Boolean);

        let TemplateChild::Element(button) = &root.children[0] else {
            panic!("expected button child");
        };

        assert_eq!(button.attributes.len(), 4);
        assert_eq!(button.attributes[0].name, "type");
        assert_eq!(
            button.attributes[0].value,
            AttributeValue::Static("button".to_string())
        );
        assert_eq!(button.attributes[1].name, "data-mode");
        assert_eq!(
            button.attributes[1].value,
            AttributeValue::Static("safe & sound".to_string())
        );
        assert_eq!(button.attributes[2].name, "title");
        assert_eq!(
            button.attributes[2].value,
            AttributeValue::Static("Use <carefully>".to_string())
        );
        assert_eq!(button.attributes[3].name, "data-ez-bindings");

        let html = generate_static_html(&template_graph);

        assert_eq!(
            html,
            "<section data-ez-node=\"n0\" id=\"panel-root\" aria-label=\"Status &quot;Panel&quot;\" hidden><button data-ez-node=\"n1\" type=\"button\" data-mode=\"safe &amp; sound\" title=\"Use &lt;carefully&gt;\" data-ez-bindings=\"this.label\">Label:<!-- ez-binding:n2:this.label -->Ready</button></section>\n"
        );
    }

    #[test]
    fn reports_static_attribute_semantic_errors() {
        let source = r#"
@component("x-bad-attrs")
class BadAttrs extends Component {
  disabled = state(false);

  render() {
    return <button type="button" type="submit" title={label} {...props}>Go</button>;
  }
}
"#;

        let parsed = ezc_parser::parse_file("BadAttrs.tsx", source);
        let graph = build_component_graph(&parsed);
        let codes = graph
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>();

        assert!(codes.contains(&"EZC1007"));
        assert!(codes.contains(&"EZC1008"));
        assert!(codes.contains(&"EZC1009"));
    }

    #[test]
    fn builds_dynamic_attribute_bindings_in_template_outputs() {
        let source = include_str!(
            "../../../fixtures/0015-dynamic-attributes/input/DynamicAttributeButton.tsx"
        );

        let parsed = ezc_parser::parse_file(
            "fixtures/0015-dynamic-attributes/input/DynamicAttributeButton.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        assert!(component_graph.diagnostics.is_empty());

        let template_graph = build_template_graph(&component_graph);
        let root = template_graph.templates[0]
            .root
            .as_ref()
            .expect("expected root");

        assert_eq!(root.attributes[0].name, "disabled");
        assert_eq!(
            root.attributes[0].value,
            AttributeValue::Binding {
                id: TemplateNodeId("n1".to_string()),
                expression: "this.disabled".to_string(),
                initial_value: Some(SerializableValue::Boolean(false)),
            }
        );
        assert_eq!(root.attributes[1].name, "title");
        assert_eq!(
            root.attributes[1].value,
            AttributeValue::Binding {
                id: TemplateNodeId("n2".to_string()),
                expression: "this.label".to_string(),
                initial_value: Some(SerializableValue::String("Ready".to_string())),
            }
        );

        let html = generate_static_html(&template_graph);

        assert_eq!(
            html,
            "<button data-ez-node=\"n0\" title=\"Ready\" data-ez-on-click=\"this.lock\" data-ez-bindings=\"this.label\">Status:<!-- ez-binding:n3:this.label -->Ready</button>\n"
        );
    }

    #[test]
    fn preserves_boolean_state_literals_in_template_outputs() {
        let source = include_str!("../../../fixtures/0007-boolean-state/input/BooleanFlags.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0007-boolean-state/input/BooleanFlags.tsx", source);

        let component_graph = build_component_graph(&parsed);
        let component = component_graph
            .components
            .first()
            .expect("expected component");

        assert_eq!(
            component.state_fields[0].initial_value,
            Some(SerializableValue::Boolean(true))
        );
        assert_eq!(
            component.state_fields[1].initial_value,
            Some(SerializableValue::Boolean(false))
        );

        let template_graph = build_template_graph(&component_graph);
        let html = generate_static_html(&template_graph);

        assert_eq!(
            html,
            "<section data-ez-node=\"n0\"><p data-ez-node=\"n1\" data-ez-bindings=\"this.enabled\">Enabled:<!-- ez-binding:n2:this.enabled -->true</p><p data-ez-node=\"n3\" data-ez-bindings=\"this.disabled\">Disabled:<!-- ez-binding:n4:this.disabled -->false</p></section>\n"
        );

        let manifest = build_template_manifest(&component_graph, &template_graph);

        assert_eq!(
            manifest.components[0].template.nodes,
            vec![
                ManifestNode::Element {
                    id: "n0".to_string(),
                    tag: "section".to_string(),
                },
                ManifestNode::Element {
                    id: "n1".to_string(),
                    tag: "p".to_string(),
                },
                ManifestNode::Binding {
                    id: "n2".to_string(),
                    expression: "this.enabled".to_string(),
                    initial_value: Some(SerializableValue::Boolean(true)),
                    target: None,
                    element: None,
                    attribute: None,
                },
                ManifestNode::Element {
                    id: "n3".to_string(),
                    tag: "p".to_string(),
                },
                ManifestNode::Binding {
                    id: "n4".to_string(),
                    expression: "this.disabled".to_string(),
                    initial_value: Some(SerializableValue::Boolean(false)),
                    target: None,
                    element: None,
                    attribute: None,
                }
            ]
        );

        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        assert_eq!(
            manifest_value["components"][0]["template"]["nodes"][2]["initial_value"],
            serde_json::json!(true)
        );
        assert_eq!(
            manifest_value["components"][0]["template"]["nodes"][4]["initial_value"],
            serde_json::json!(false)
        );
    }

    #[test]
    fn preserves_null_state_literals_in_template_outputs() {
        let source = include_str!("../../../fixtures/0008-null-state/input/NullSelection.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0008-null-state/input/NullSelection.tsx", source);

        let component_graph = build_component_graph(&parsed);
        let component = component_graph
            .components
            .first()
            .expect("expected component");

        assert_eq!(
            component.state_fields[0].initial_value,
            Some(SerializableValue::Null)
        );

        let template_graph = build_template_graph(&component_graph);
        let html = generate_static_html(&template_graph);

        assert_eq!(
            html,
            "<p data-ez-node=\"n0\" data-ez-bindings=\"this.selection\">Selection:<!-- ez-binding:n1:this.selection --></p>\n"
        );

        let manifest = build_template_manifest(&component_graph, &template_graph);

        assert_eq!(
            manifest.components[0].template.nodes,
            vec![
                ManifestNode::Element {
                    id: "n0".to_string(),
                    tag: "p".to_string(),
                },
                ManifestNode::Binding {
                    id: "n1".to_string(),
                    expression: "this.selection".to_string(),
                    initial_value: Some(SerializableValue::Null),
                    target: None,
                    element: None,
                    attribute: None,
                }
            ]
        );

        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        assert_eq!(
            manifest_value["components"][0]["template"]["nodes"][1]["initial_value"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn builds_template_graph_from_component_graph() {
        let source = include_str!("../../../fixtures/0001-source-summary/input/Counter.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0001-source-summary/input/Counter.tsx", source);

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);

        assert_eq!(template_graph.templates.len(), 1);

        let template = &template_graph.templates[0];
        assert_eq!(template.component_name, "Counter");
        assert_eq!(template.id.as_str(), "component:x-counter/template:render");
        assert_eq!(
            template.provenance.path,
            Path::new("fixtures/0001-source-summary/input/Counter.tsx")
        );
        assert_eq!(template.provenance.span.line, 10);
        assert_eq!(
            template.owner,
            SemanticOwner::entity(SemanticId::component(Some("x-counter"), "Counter"))
        );

        let root = template.root.as_ref().expect("expected template root");

        assert_eq!(root.id.0, "n0");
        assert_eq!(root.tag_name, "button");
        assert_eq!(root.span.line, 12);
        assert_eq!(root.span.column, 7);

        assert_eq!(root.attributes.len(), 2);
        assert_eq!(root.attributes[0].name, "data-ez-on-click");
        assert_eq!(
            root.attributes[0].span.expect("expected event span").line,
            12
        );
        assert_eq!(
            root.attributes[0].span.expect("expected event span").column,
            15
        );
        assert_eq!(
            root.attributes[0].value,
            AttributeValue::EventHandler {
                event: "click".to_string(),
                handler: "this.increment".to_string(),
            }
        );

        assert_eq!(root.attributes[1].name, "data-ez-bindings");
        assert_eq!(root.attributes[1].span, None);
        assert_eq!(
            root.attributes[1].value,
            AttributeValue::BindingList(vec!["this.count".to_string()])
        );

        assert_eq!(root.children.len(), 2);
        let TemplateChild::Text { value, span } = &root.children[0] else {
            panic!("expected text child");
        };
        assert_eq!(value, "Count:");
        assert_eq!(span.line, 13);
        assert_eq!(span.column, 9);

        let TemplateChild::Binding {
            id,
            expression,
            initial_value,
            span,
        } = &root.children[1]
        else {
            panic!("expected binding child");
        };
        assert_eq!(id.0, "n1");
        assert_eq!(expression, "this.count");
        assert_eq!(
            initial_value,
            &Some(SerializableValue::Number("0".to_string()))
        );
        assert_eq!(span.line, 13);
        assert_eq!(span.column, 16);
    }

    #[test]
    fn semantic_ids_are_stable_when_component_declaration_order_changes() {
        let alpha = r#"
@component("x-alpha")
class Alpha extends Component {
  count = state(0);

  increment() {
    this.count++;
  }

  render() {
    return <p>{this.count}</p>;
  }
}
"#;
        let beta = r#"
@component("x-beta")
class Beta extends Component {
  enabled = state(false);

  toggle() {
    this.enabled = !this.enabled;
  }

  render() {
    return <p>{this.enabled}</p>;
  }
}
"#;

        let ids_for = |source: &str| {
            let parsed = ezc_parser::parse_file("App.tsx", source);
            let component_graph = build_component_graph(&parsed);
            let template_graph = build_template_graph(&component_graph);
            let template_ids = template_graph
                .templates
                .iter()
                .map(|template| (template.component_name.as_str(), template.id.to_string()))
                .collect::<BTreeMap<_, _>>();

            component_graph
                .components
                .iter()
                .map(|component| {
                    let ids = component
                        .state_fields
                        .iter()
                        .map(|field| field.id.to_string())
                        .chain(component.methods.iter().map(|method| method.id.to_string()))
                        .chain(component.actions.iter().map(|action| action.id.to_string()))
                        .chain(std::iter::once(
                            template_ids
                                .get(component.class_name.as_str())
                                .expect("expected component template")
                                .clone(),
                        ))
                        .collect::<Vec<_>>();

                    (
                        component.class_name.clone(),
                        (component.id.to_string(), ids),
                    )
                })
                .collect::<BTreeMap<_, _>>()
        };

        assert_eq!(
            ids_for(&format!("{alpha}\n{beta}")),
            ids_for(&format!("{beta}\n{alpha}"))
        );
    }

    #[test]
    fn carries_source_spans_into_nested_template_nodes() {
        let source = include_str!("../../../fixtures/0004-nested-jsx/input/NestedCounter.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0004-nested-jsx/input/NestedCounter.tsx", source);

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let root = template_graph.templates[0]
            .root
            .as_ref()
            .expect("expected root");

        assert_eq!(root.tag_name, "section");
        assert_eq!(root.span.line, 12);
        assert_eq!(root.span.column, 7);

        let TemplateChild::Element(button) = &root.children[0] else {
            panic!("expected nested button");
        };

        assert_eq!(button.tag_name, "button");
        assert_eq!(button.span.line, 13);
        assert_eq!(button.span.column, 9);
        assert_eq!(button.attributes[0].name, "data-ez-on-click");
        assert_eq!(
            button.attributes[0].span.expect("expected event span").line,
            13
        );
        assert_eq!(
            button.attributes[0]
                .span
                .expect("expected event span")
                .column,
            17
        );
        assert_eq!(button.attributes[1].name, "data-ez-bindings");
        assert_eq!(button.attributes[1].span, None);

        let TemplateChild::Text { value, span } = &button.children[0] else {
            panic!("expected nested text");
        };
        assert_eq!(value, "Count:");
        assert_eq!(span.line, 13);
        assert_eq!(span.column, 50);

        let TemplateChild::Binding {
            expression, span, ..
        } = &button.children[1]
        else {
            panic!("expected nested binding");
        };
        assert_eq!(expression, "this.count");
        assert_eq!(span.line, 13);
        assert_eq!(span.column, 57);
    }

    #[test]
    fn preserves_fragment_siblings_without_wrapper_elements() {
        let source = include_str!("../../../fixtures/0016-fragments/input/FragmentPanel.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0016-fragments/input/FragmentPanel.tsx", source);

        let component_graph = build_component_graph(&parsed);
        assert!(component_graph.diagnostics.is_empty());

        let template_graph = build_template_graph(&component_graph);
        let template = &template_graph.templates[0];
        let fragment = template
            .root_fragment
            .as_ref()
            .expect("expected fragment root");

        assert!(template.root.is_none());
        assert_eq!(fragment.id.0, "n0");
        assert_eq!(fragment.children.len(), 2);

        let TemplateChild::Element(heading) = &fragment.children[0] else {
            panic!("expected heading child");
        };
        assert_eq!(heading.id.0, "n1");
        assert_eq!(heading.tag_name, "h1");

        let TemplateChild::Fragment(nested) = &fragment.children[1] else {
            panic!("expected nested fragment child");
        };
        assert_eq!(nested.id.0, "n2");

        let TemplateChild::Element(paragraph) = &nested.children[0] else {
            panic!("expected paragraph child");
        };
        assert_eq!(paragraph.id.0, "n3");
        assert_eq!(paragraph.tag_name, "p");

        let html = generate_static_html(&template_graph);
        assert_eq!(
            html,
            "<h1 data-ez-node=\"n1\">Title</h1><p data-ez-node=\"n3\" data-ez-bindings=\"this.label\">Status:<!-- ez-binding:n4:this.label -->Ready</p><span data-ez-node=\"n5\">Done</span>\n"
        );

        let manifest = build_template_manifest(&component_graph, &template_graph);
        assert_eq!(
            manifest.components[0].template.nodes,
            vec![
                ManifestNode::Element {
                    id: "n1".to_string(),
                    tag: "h1".to_string(),
                },
                ManifestNode::Element {
                    id: "n3".to_string(),
                    tag: "p".to_string(),
                },
                ManifestNode::Binding {
                    id: "n4".to_string(),
                    expression: "this.label".to_string(),
                    initial_value: Some(SerializableValue::String("Ready".to_string())),
                    target: None,
                    element: None,
                    attribute: None,
                },
                ManifestNode::Element {
                    id: "n5".to_string(),
                    tag: "span".to_string(),
                },
            ]
        );
    }

    #[test]
    fn builds_conditional_template_boundaries_and_manifest_branch_html() {
        let source = include_str!(
            "../../../fixtures/0017-conditional-rendering/input/ConditionalStatus.tsx"
        );

        let parsed = ezc_parser::parse_file(
            "fixtures/0017-conditional-rendering/input/ConditionalStatus.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        assert!(component_graph.diagnostics.is_empty());

        let template_graph = build_template_graph(&component_graph);
        let root = template_graph.templates[0]
            .root
            .as_ref()
            .expect("expected root element");

        assert_eq!(root.id.0, "n0");
        assert_eq!(root.attributes[1].name, "data-ez-bindings");

        let TemplateChild::Conditional(conditional) = &root.children[0] else {
            panic!("expected conditional child");
        };

        assert_eq!(conditional.id.0, "n1");
        assert_eq!(conditional.start_id.0, "n2");
        assert_eq!(conditional.end_id.0, "n3");
        assert_eq!(conditional.condition, "this.enabled");
        assert_eq!(
            conditional.initial_value,
            Some(SerializableValue::Boolean(true))
        );

        let TemplateChild::Element(when_true) = &conditional.when_true[0] else {
            panic!("expected true branch element");
        };
        assert_eq!(when_true.id.0, "n4");
        assert_eq!(when_true.tag_name, "span");

        let TemplateChild::Element(when_false) = &conditional.when_false[0] else {
            panic!("expected false branch element");
        };
        assert_eq!(when_false.id.0, "n5");
        assert_eq!(when_false.tag_name, "span");

        let html = generate_static_html(&template_graph);
        assert_eq!(
            html,
            "<button data-ez-node=\"n0\" data-ez-on-click=\"this.toggle\" data-ez-bindings=\"this.enabled\"><!-- ez-conditional-start:n2:this.enabled --><span data-ez-node=\"n4\">On</span><!-- ez-conditional-end:n3 --></button>\n"
        );

        let manifest = build_template_manifest(&component_graph, &template_graph);
        assert_eq!(
            manifest.components[0].template.nodes,
            vec![
                ManifestNode::Element {
                    id: "n0".to_string(),
                    tag: "button".to_string(),
                },
                ManifestNode::Conditional {
                    id: "n1".to_string(),
                    start: "n2".to_string(),
                    end: "n3".to_string(),
                    condition: "this.enabled".to_string(),
                    initial_value: Some(SerializableValue::Boolean(true)),
                    when_true_html: "<span data-ez-node=\"n4\">On</span>".to_string(),
                    when_false_html: "<span data-ez-node=\"n5\">Off</span>".to_string(),
                },
            ]
        );
    }

    #[test]
    fn builds_logical_and_conditional_with_empty_false_branch() {
        let source = include_str!(
            "../../../fixtures/0018-logical-and-conditional/input/LogicalAndStatus.tsx"
        );

        let parsed = ezc_parser::parse_file(
            "fixtures/0018-logical-and-conditional/input/LogicalAndStatus.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        assert!(component_graph.diagnostics.is_empty());

        let template_graph = build_template_graph(&component_graph);
        let root = template_graph.templates[0]
            .root
            .as_ref()
            .expect("expected root element");

        let TemplateChild::Conditional(conditional) = &root.children[0] else {
            panic!("expected conditional child");
        };

        assert_eq!(conditional.when_true.len(), 1);
        assert!(conditional.when_false.is_empty());

        let manifest = build_template_manifest(&component_graph, &template_graph);
        assert_eq!(
            manifest.components[0].template.nodes[1],
            ManifestNode::Conditional {
                id: "n1".to_string(),
                start: "n2".to_string(),
                end: "n3".to_string(),
                condition: "this.enabled".to_string(),
                initial_value: Some(SerializableValue::Boolean(true)),
                when_true_html: "<span data-ez-node=\"n4\">On</span>".to_string(),
                when_false_html: String::new(),
            }
        );
    }

    #[test]
    fn builds_empty_keyed_list_from_a_serializable_array() {
        let source =
            include_str!("../../../fixtures/0019-keyed-list-semantics/input/KeyedList.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0019-keyed-list-semantics/input/KeyedList.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        assert!(component_graph.diagnostics.is_empty());

        let template_graph = build_template_graph(&component_graph);
        let root = template_graph.templates[0]
            .root
            .as_ref()
            .expect("expected root element");

        assert_eq!(root.id.0, "n0");
        assert_eq!(root.attributes[0].name, "data-ez-bindings");

        let TemplateChild::List(list) = &root.children[0] else {
            panic!("expected keyed list child");
        };

        assert_eq!(list.id.0, "n1");
        assert_eq!(list.start_id.0, "n2");
        assert_eq!(list.end_id.0, "n3");
        assert_eq!(list.iterable, "this.items");
        assert_eq!(
            list.initial_value,
            Some(SerializableValue::Array(Vec::new()))
        );
        assert_eq!(list.item_variable, "item");
        assert_eq!(list.index_variable.as_deref(), Some("index"));
        assert_eq!(list.key_expression, "item.id");

        let TemplateChild::Element(item_template) = &list.item_template[0] else {
            panic!("expected list item element");
        };
        assert_eq!(item_template.id.0, "n4");
        assert_eq!(item_template.tag_name, "li");

        assert_eq!(
            generate_static_html(&template_graph),
            "<ul data-ez-node=\"n0\" data-ez-bindings=\"this.items\"><!-- ez-list-start:n2:this.items --><!-- ez-list-end:n3 --></ul>\n"
        );

        let manifest = build_template_manifest(&component_graph, &template_graph);
        assert_eq!(
            manifest.components[0].template.nodes,
            vec![
                ManifestNode::Element {
                    id: "n0".to_string(),
                    tag: "ul".to_string(),
                },
                ManifestNode::List {
                    id: "n1".to_string(),
                    start: "n2".to_string(),
                    end: "n3".to_string(),
                    iterable: "this.items".to_string(),
                    initial_value: Some(SerializableValue::Array(Vec::new())),
                    item_variable: "item".to_string(),
                    index_variable: Some("index".to_string()),
                    key_expression: "item.id".to_string(),
                    item_root: "n4".to_string(),
                    item_template_html: "<li data-ez-node=\"n4:__ez_list_key__\" data-ez-bindings=\"index,item.label\"><!-- ez-binding:n5:__ez_list_key__:index -->__ez_list_index__<!-- ez-list-binding-end:n5:__ez_list_key__ -->:<!-- ez-binding:n6:__ez_list_key__:item.label --><!-- ez-list-binding-end:n6:__ez_list_key__ --></li>".to_string(),
                },
            ]
        );
    }

    #[test]
    fn preserves_recursive_object_values_in_component_and_manifest_models() {
        let source = include_str!(
            "../../../fixtures/0023-recursive-object-values/input/RecursiveObjectValues.tsx"
        );
        let parsed = ezc_parser::parse_file(
            "fixtures/0023-recursive-object-values/input/RecursiveObjectValues.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        assert!(component_graph.diagnostics.is_empty());

        let profile = SerializableValue::Object(BTreeMap::from([
            (
                "name".to_string(),
                SerializableValue::String("North".to_string()),
            ),
            (
                "settings".to_string(),
                SerializableValue::Object(BTreeMap::from([
                    ("enabled".to_string(), SerializableValue::Boolean(true)),
                    (
                        "tags".to_string(),
                        SerializableValue::Array(vec![
                            SerializableValue::String("compiler".to_string()),
                            SerializableValue::Object(BTreeMap::from([
                                (
                                    "name".to_string(),
                                    SerializableValue::String("runtime".to_string()),
                                ),
                                (
                                    "rank".to_string(),
                                    SerializableValue::Number("2".to_string()),
                                ),
                            ])),
                        ]),
                    ),
                ])),
            ),
        ]));
        assert_eq!(
            component_graph.components[0].state_fields[0].initial_value,
            Some(profile.clone())
        );

        let template_graph = build_template_graph(&component_graph);
        let manifest = build_template_manifest(&component_graph, &template_graph);
        assert_eq!(
            manifest.components[0].template.nodes[3],
            ManifestNode::Binding {
                id: "n3".to_string(),
                expression: "this.profile".to_string(),
                initial_value: Some(profile),
                target: None,
                element: None,
                attribute: None,
            }
        );
        assert!(matches!(
            manifest.components[0].actions[0].operand,
            Some(SerializableValue::Object(_))
        ));
    }

    #[test]
    fn renders_static_object_list_members_and_keys() {
        let source = include_str!(
            "../../../fixtures/0024-static-object-keyed-list/input/StaticObjectKeyedList.tsx"
        );
        let parsed = ezc_parser::parse_file(
            "fixtures/0024-static-object-keyed-list/input/StaticObjectKeyedList.tsx",
            source,
        );
        let component_graph = build_component_graph(&parsed);
        assert!(component_graph.diagnostics.is_empty());

        let template_graph = build_template_graph(&component_graph);
        assert_eq!(
            generate_static_html(&template_graph),
            "<ol data-ez-node=\"n0\" data-ez-bindings=\"this.items\"><!-- ez-list-start:n2:this.items --><li data-ez-node=\"n4:north\" data-ez-bindings=\"index,item.label,item.details.region\"><!-- ez-binding:n5:north:index -->0<!-- ez-list-binding-end:n5:north -->:<!-- ez-binding:n6:north:item.label -->North<!-- ez-list-binding-end:n6:north -->(<!-- ez-binding:n7:north:item.details.region -->west<!-- ez-list-binding-end:n7:north -->)</li><li data-ez-node=\"n4:south\" data-ez-bindings=\"index,item.label,item.details.region\"><!-- ez-binding:n5:south:index -->1<!-- ez-list-binding-end:n5:south -->:<!-- ez-binding:n6:south:item.label -->South<!-- ez-list-binding-end:n6:south -->(<!-- ez-binding:n7:south:item.details.region -->east<!-- ez-list-binding-end:n7:south -->)</li><!-- ez-list-end:n3 --></ol>\n"
        );
    }

    #[test]
    fn renders_initial_keyed_list_items_from_a_serializable_array() {
        let source =
            include_str!("../../../fixtures/0020-static-keyed-list/input/StaticKeyedList.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0020-static-keyed-list/input/StaticKeyedList.tsx",
            source,
        );
        let component_graph = build_component_graph(&parsed);
        assert!(component_graph.diagnostics.is_empty());

        let template_graph = build_template_graph(&component_graph);
        let root = template_graph.templates[0]
            .root
            .as_ref()
            .expect("expected root element");
        let TemplateChild::List(list) = &root.children[0] else {
            panic!("expected keyed list child");
        };

        assert_eq!(
            list.initial_value,
            Some(SerializableValue::Array(vec![
                SerializableValue::String("North".to_string()),
                SerializableValue::String("South".to_string()),
            ]))
        );

        assert_eq!(
            generate_static_html(&template_graph),
            "<ol data-ez-node=\"n0\" data-ez-bindings=\"this.labels\"><!-- ez-list-start:n2:this.labels --><li data-ez-node=\"n4:North\" data-ez-bindings=\"index,label\"><!-- ez-binding:n5:North:index -->0<!-- ez-list-binding-end:n5:North -->:<!-- ez-binding:n6:North:label -->North<!-- ez-list-binding-end:n6:North --></li><li data-ez-node=\"n4:South\" data-ez-bindings=\"index,label\"><!-- ez-binding:n5:South:index -->1<!-- ez-list-binding-end:n5:South -->:<!-- ez-binding:n6:South:label -->South<!-- ez-list-binding-end:n6:South --></li><!-- ez-list-end:n3 --></ol>\n"
        );

        let manifest = build_template_manifest(&component_graph, &template_graph);
        assert_eq!(
            manifest.components[0].template.nodes,
            vec![
                ManifestNode::Element {
                    id: "n0".to_string(),
                    tag: "ol".to_string(),
                },
                ManifestNode::List {
                    id: "n1".to_string(),
                    start: "n2".to_string(),
                    end: "n3".to_string(),
                    iterable: "this.labels".to_string(),
                    initial_value: Some(SerializableValue::Array(vec![
                        SerializableValue::String("North".to_string()),
                        SerializableValue::String("South".to_string()),
                    ])),
                    item_variable: "label".to_string(),
                    index_variable: Some("index".to_string()),
                    key_expression: "label".to_string(),
                    item_root: "n4".to_string(),
                    item_template_html: "<li data-ez-node=\"n4:__ez_list_key__\" data-ez-bindings=\"index,label\"><!-- ez-binding:n5:__ez_list_key__:index -->__ez_list_index__<!-- ez-list-binding-end:n5:__ez_list_key__ -->:<!-- ez-binding:n6:__ez_list_key__:label -->__ez_list_item__<!-- ez-list-binding-end:n6:__ez_list_key__ --></li>".to_string(),
                },
            ]
        );
    }

    #[test]
    fn builds_template_manifest_for_nested_jsx() {
        let source = include_str!("../../../fixtures/0004-nested-jsx/input/NestedCounter.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0004-nested-jsx/input/NestedCounter.tsx", source);

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let manifest = build_template_manifest(&component_graph, &template_graph);

        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.components.len(), 1);

        let component = &manifest.components[0];
        assert_eq!(component.name, "NestedCounter");

        assert_eq!(
            component.template.nodes,
            vec![
                ManifestNode::Element {
                    id: "n0".to_string(),
                    tag: "section".to_string(),
                },
                ManifestNode::Element {
                    id: "n1".to_string(),
                    tag: "button".to_string(),
                },
                ManifestNode::Binding {
                    id: "n2".to_string(),
                    expression: "this.count".to_string(),
                    initial_value: Some(SerializableValue::Number("0".to_string())),
                    target: None,
                    element: None,
                    attribute: None,
                }
            ]
        );

        assert_eq!(
            component.template.events,
            vec![ManifestEvent {
                node: "n1".to_string(),
                kind: None,
                event: "click".to_string(),
                handler: "this.increment".to_string(),
                method_id: None,
                action_batch_id: None,
            }]
        );

        assert_eq!(
            component.actions,
            vec![ManifestAction {
                method: "increment".to_string(),
                method_id: None,
                action_batch_id: None,
                operation: ManifestOperation::Increment,
                field: "count".to_string(),
                storage_id: None,
                operand: None,
            }]
        );

        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        assert_eq!(manifest_value["schema_version"], serde_json::json!(1));
    }

    #[test]
    fn builds_template_manifest_for_decrement_action() {
        let source =
            include_str!("../../../fixtures/0009-decrement-counter/input/DecrementCounter.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0009-decrement-counter/input/DecrementCounter.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let manifest = build_template_manifest(&component_graph, &template_graph);

        assert_eq!(
            manifest.components[0].actions,
            vec![ManifestAction {
                method: "decrement".to_string(),
                method_id: None,
                action_batch_id: None,
                operation: ManifestOperation::Decrement,
                field: "count".to_string(),
                storage_id: None,
                operand: None,
            }]
        );

        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        assert_eq!(
            manifest_value["components"][0]["actions"][0]["operation"],
            serde_json::json!("decrement")
        );
    }

    #[test]
    fn builds_template_manifest_for_add_and_subtract_assign_actions() {
        let source =
            include_str!("../../../fixtures/0010-add-subtract-assign/input/StepCounter.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0010-add-subtract-assign/input/StepCounter.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let manifest = build_template_manifest(&component_graph, &template_graph);

        assert_eq!(
            manifest.components[0].actions,
            vec![
                ManifestAction {
                    method: "addTwo".to_string(),
                    method_id: None,
                    action_batch_id: None,
                    operation: ManifestOperation::AddAssign,
                    field: "count".to_string(),
                    storage_id: None,
                    operand: Some(SerializableValue::Number("2".to_string())),
                },
                ManifestAction {
                    method: "subtractThree".to_string(),
                    method_id: None,
                    action_batch_id: None,
                    operation: ManifestOperation::SubtractAssign,
                    field: "count".to_string(),
                    storage_id: None,
                    operand: Some(SerializableValue::Number("3".to_string())),
                }
            ]
        );

        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        assert_eq!(
            manifest_value["components"][0]["actions"][0]["operation"],
            serde_json::json!("add_assign")
        );
        assert_eq!(
            manifest_value["components"][0]["actions"][0]["operand"],
            serde_json::json!("2")
        );
        assert_eq!(
            manifest_value["components"][0]["actions"][1]["operation"],
            serde_json::json!("subtract_assign")
        );
        assert_eq!(
            manifest_value["components"][0]["actions"][1]["operand"],
            serde_json::json!("3")
        );
    }

    #[test]
    fn builds_template_manifest_for_direct_assignment_action() {
        let source =
            include_str!("../../../fixtures/0011-direct-assignment/input/ResetCounter.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0011-direct-assignment/input/ResetCounter.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let manifest = build_template_manifest(&component_graph, &template_graph);

        assert_eq!(
            manifest.components[0].actions,
            vec![ManifestAction {
                method: "reset".to_string(),
                method_id: None,
                action_batch_id: None,
                operation: ManifestOperation::Assign,
                field: "count".to_string(),
                storage_id: None,
                operand: Some(SerializableValue::Number("0".to_string())),
            }]
        );

        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        assert_eq!(
            manifest_value["components"][0]["actions"][0]["operation"],
            serde_json::json!("assign")
        );
        assert_eq!(
            manifest_value["components"][0]["actions"][0]["operand"],
            serde_json::json!("0")
        );
    }

    #[test]
    fn builds_template_manifest_for_boolean_toggle_action() {
        let source = include_str!("../../../fixtures/0012-boolean-toggle/input/ToggleFlag.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0012-boolean-toggle/input/ToggleFlag.tsx", source);

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let manifest = build_template_manifest(&component_graph, &template_graph);

        assert_eq!(
            manifest.components[0].actions,
            vec![ManifestAction {
                method: "toggle".to_string(),
                method_id: None,
                action_batch_id: None,
                operation: ManifestOperation::Toggle,
                field: "enabled".to_string(),
                storage_id: None,
                operand: None,
            }]
        );

        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        assert_eq!(
            manifest_value["components"][0]["actions"][0]["operation"],
            serde_json::json!("toggle")
        );
        assert!(manifest_value["components"][0]["actions"][0]
            .get("operand")
            .is_none());
    }

    #[test]
    fn builds_template_manifest_for_multi_step_action_in_source_order() {
        let source =
            include_str!("../../../fixtures/0013-multi-step-action/input/BatchActionCounter.tsx");

        let parsed = ezc_parser::parse_file(
            "fixtures/0013-multi-step-action/input/BatchActionCounter.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let manifest = build_template_manifest(&component_graph, &template_graph);

        assert_eq!(
            manifest.components[0].actions,
            vec![
                ManifestAction {
                    method: "apply".to_string(),
                    method_id: None,
                    action_batch_id: None,
                    operation: ManifestOperation::AddAssign,
                    field: "count".to_string(),
                    storage_id: None,
                    operand: Some(SerializableValue::Number("2".to_string())),
                },
                ManifestAction {
                    method: "apply".to_string(),
                    method_id: None,
                    action_batch_id: None,
                    operation: ManifestOperation::Decrement,
                    field: "count".to_string(),
                    storage_id: None,
                    operand: None,
                },
                ManifestAction {
                    method: "apply".to_string(),
                    method_id: None,
                    action_batch_id: None,
                    operation: ManifestOperation::Assign,
                    field: "count".to_string(),
                    storage_id: None,
                    operand: Some(SerializableValue::Number("8".to_string())),
                },
                ManifestAction {
                    method: "apply".to_string(),
                    method_id: None,
                    action_batch_id: None,
                    operation: ManifestOperation::Increment,
                    field: "count".to_string(),
                    storage_id: None,
                    operand: None,
                },
                ManifestAction {
                    method: "apply".to_string(),
                    method_id: None,
                    action_batch_id: None,
                    operation: ManifestOperation::Toggle,
                    field: "enabled".to_string(),
                    storage_id: None,
                    operand: None,
                }
            ]
        );

        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        let actions = manifest_value["components"][0]["actions"]
            .as_array()
            .expect("manifest actions should be an array");

        assert_eq!(
            actions
                .iter()
                .map(|action| action["operation"].as_str().expect("operation is a string"))
                .collect::<Vec<_>>(),
            vec!["add_assign", "decrement", "assign", "increment", "toggle"]
        );
        assert_eq!(actions[0]["method"], serde_json::json!("apply"));
        assert_eq!(actions[4]["method"], serde_json::json!("apply"));
        assert!(actions[1].get("operand").is_none());
        assert!(actions[3].get("operand").is_none());
        assert!(actions[4].get("operand").is_none());
    }

    #[test]
    fn builds_template_manifest_for_dynamic_attribute_bindings() {
        let source = include_str!(
            "../../../fixtures/0015-dynamic-attributes/input/DynamicAttributeButton.tsx"
        );

        let parsed = ezc_parser::parse_file(
            "fixtures/0015-dynamic-attributes/input/DynamicAttributeButton.tsx",
            source,
        );

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let manifest = build_template_manifest(&component_graph, &template_graph);
        let manifest_json = template_manifest_json(&manifest);
        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_json).expect("manifest JSON should parse");

        assert_eq!(
            manifest.components[0].template.nodes[1],
            ManifestNode::Binding {
                id: "n1".to_string(),
                expression: "this.disabled".to_string(),
                initial_value: Some(SerializableValue::Boolean(false)),
                target: Some(ManifestBindingTarget::Attribute),
                element: Some("n0".to_string()),
                attribute: Some("disabled".to_string()),
            }
        );
        assert_eq!(
            manifest_value["components"][0]["template"]["nodes"][1]["target"],
            serde_json::json!("attribute")
        );
        assert_eq!(
            manifest_value["components"][0]["template"]["nodes"][1]["element"],
            serde_json::json!("n0")
        );
        assert_eq!(
            manifest_value["components"][0]["template"]["nodes"][1]["attribute"],
            serde_json::json!("disabled")
        );
        assert!(manifest_value["components"][0]["template"]["nodes"][3]
            .get("target")
            .is_none());
    }

    #[test]
    fn generates_standalone_page_with_embedded_manifest() {
        let source = include_str!("../../../fixtures/0004-nested-jsx/input/NestedCounter.tsx");

        let parsed =
            ezc_parser::parse_file("fixtures/0004-nested-jsx/input/NestedCounter.tsx", source);

        let component_graph = build_component_graph(&parsed);
        let template_graph = build_template_graph(&component_graph);
        let html = generate_static_html(&template_graph);
        let manifest = build_template_manifest(&component_graph, &template_graph);
        let page = generate_standalone_page("NestedCounter", &html, &manifest);

        assert!(page.starts_with("<!doctype html>\n"));
        assert!(page.contains("<title>NestedCounter</title>"));
        assert!(page.contains("<section data-ez-node=\"n0\">"));
        assert!(page.contains("id=\"ez-template-manifest\""));
        assert!(page.contains("\"name\": \"NestedCounter\""));
        assert!(page.contains("<script src=\"./runtime.js\" defer></script>"));
    }
}
