use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ApplicationSemanticModel, ComponentBuildRoot, ComponentNode, ComponentRootId, FieldBindingId,
    FieldId, FormEntity, FormFieldBinding, FormFieldEntity, FormId, FormOwnershipGraphId,
    SemanticId, SemanticOwner, SemanticReference, SemanticReferenceKind, SourceProvenance,
    TemplateSemanticEntity, TemplateSemanticKind,
};

/// A typed sum key retaining each existing canonical identity without
/// re-encoding it into a Form-specific node identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormOwnershipNodeKey {
    Component(SemanticId),
    Form(FormId),
    FormField(FieldId),
    TemplateControl(SemanticId),
    FieldBinding(FieldBindingId),
}

impl FormOwnershipNodeKey {
    #[must_use]
    pub fn semantic_id(&self) -> &SemanticId {
        match self {
            Self::Component(id) | Self::TemplateControl(id) => id,
            Self::Form(id) => id.as_semantic_id(),
            Self::FormField(id) => id.as_semantic_id(),
            Self::FieldBinding(id) => id.as_semantic_id(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormOwnershipNode {
    Component {
        id: SemanticId,
        provenance: SourceProvenance,
    },
    Form {
        id: FormId,
        provenance: SourceProvenance,
    },
    FormField {
        id: FieldId,
        provenance: SourceProvenance,
    },
    TemplateControl {
        id: SemanticId,
        provenance: SourceProvenance,
    },
    FieldBinding {
        id: FieldBindingId,
        provenance: SourceProvenance,
    },
}

impl FormOwnershipNode {
    #[must_use]
    pub fn key(&self) -> FormOwnershipNodeKey {
        match self {
            Self::Component { id, .. } => FormOwnershipNodeKey::Component(id.clone()),
            Self::Form { id, .. } => FormOwnershipNodeKey::Form(id.clone()),
            Self::FormField { id, .. } => FormOwnershipNodeKey::FormField(id.clone()),
            Self::TemplateControl { id, .. } => FormOwnershipNodeKey::TemplateControl(id.clone()),
            Self::FieldBinding { id, .. } => FormOwnershipNodeKey::FieldBinding(id.clone()),
        }
    }

    #[must_use]
    pub const fn provenance(&self) -> &SourceProvenance {
        match self {
            Self::Component { provenance, .. }
            | Self::Form { provenance, .. }
            | Self::FormField { provenance, .. }
            | Self::TemplateControl { provenance, .. }
            | Self::FieldBinding { provenance, .. } => provenance,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormOwnershipEdgeKind {
    ComponentOwnsForm,
    FormOwnsField,
    TemplateControlOwnsBinding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormOwnershipEdge {
    pub owner: FormOwnershipNodeKey,
    pub child: FormOwnershipNodeKey,
    pub kind: FormOwnershipEdgeKind,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormReferenceKind {
    FieldBindingField,
    FieldBindingForm,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormReferenceEdge {
    pub kind: FormReferenceKind,
    pub source: FormOwnershipNodeKey,
    pub target: FormOwnershipNodeKey,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormOwnershipIntegrityKind {
    DuplicateNode,
    MissingOwner,
    UnknownOwnershipEndpoint,
    MultipleOwners,
    OwnershipCycle,
    UnreachableNode,
    UnknownReferenceEndpoint,
    ComponentOwnershipMismatch,
    FieldFormMismatch,
    BindingFieldMismatch,
    BindingFormMismatch,
    BindingTemplateMismatch,
    InvalidCandidatePromoted,
    InstanceIdentityInDeclarationGraph,
    MissingProvenance,
    NonCanonicalOrdering,
    GraphIdentityMismatch,
}

impl FormOwnershipIntegrityKind {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::DuplicateNode => "PSASM1203",
            Self::MissingOwner => "PSASM1204",
            Self::UnknownOwnershipEndpoint => "PSASM1205",
            Self::MultipleOwners => "PSASM1206",
            Self::OwnershipCycle => "PSASM1207",
            Self::UnreachableNode => "PSASM1208",
            Self::UnknownReferenceEndpoint => "PSASM1209",
            Self::ComponentOwnershipMismatch => "PSASM1210",
            Self::FieldFormMismatch => "PSASM1211",
            Self::BindingFieldMismatch => "PSASM1212",
            Self::BindingFormMismatch => "PSASM1213",
            Self::BindingTemplateMismatch => "PSASM1214",
            Self::InvalidCandidatePromoted => "PSASM1215",
            Self::InstanceIdentityInDeclarationGraph => "PSASM1216",
            Self::MissingProvenance => "PSASM1217",
            Self::NonCanonicalOrdering => "PSASM1218",
            Self::GraphIdentityMismatch => "PSASM1219",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormOwnershipIntegrityDiagnostic {
    pub code: String,
    pub kind: FormOwnershipIntegrityKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormOwnershipValidation {
    pub diagnostics: Vec<FormOwnershipIntegrityDiagnostic>,
    pub is_valid: bool,
}

impl Default for FormOwnershipValidation {
    fn default() -> Self {
        Self {
            diagnostics: Vec::new(),
            is_valid: true,
        }
    }
}

/// Immutable declaration-level projection of canonical Form ownership and
/// binding reference facts already present in the ASM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormOwnershipGraph {
    pub id: FormOwnershipGraphId,
    pub nodes: BTreeMap<FormOwnershipNodeKey, FormOwnershipNode>,
    pub ownership_edges: Vec<FormOwnershipEdge>,
    pub reference_edges: Vec<FormReferenceEdge>,
    pub roots: Vec<SemanticId>,
    pub validation: FormOwnershipValidation,
}

impl FormOwnershipGraph {
    #[must_use]
    pub fn node(&self, id: &FormOwnershipNodeKey) -> Option<&FormOwnershipNode> {
        self.nodes.get(id)
    }

    #[must_use]
    pub fn owner_of(&self, id: &FormOwnershipNodeKey) -> Option<&FormOwnershipNodeKey> {
        self.ownership_edges
            .iter()
            .find_map(|edge| (&edge.child == id).then_some(&edge.owner))
    }

    #[must_use]
    pub fn children_of(&self, id: &FormOwnershipNodeKey) -> Vec<&FormOwnershipNodeKey> {
        self.ownership_edges
            .iter()
            .filter_map(|edge| (&edge.owner == id).then_some(&edge.child))
            .collect()
    }

    #[must_use]
    pub fn forms_of(&self, component: &SemanticId) -> Vec<&FormId> {
        self.children_of(&FormOwnershipNodeKey::Component(component.clone()))
            .into_iter()
            .filter_map(|child| match child {
                FormOwnershipNodeKey::Form(id) => Some(id),
                _ => None,
            })
            .collect()
    }

    #[must_use]
    pub fn fields_of(&self, form: &FormId) -> Vec<&FieldId> {
        self.children_of(&FormOwnershipNodeKey::Form(form.clone()))
            .into_iter()
            .filter_map(|child| match child {
                FormOwnershipNodeKey::FormField(id) => Some(id),
                _ => None,
            })
            .collect()
    }

    #[must_use]
    pub fn binding_owner(&self, binding: &FieldBindingId) -> Option<&SemanticId> {
        match self.owner_of(&FormOwnershipNodeKey::FieldBinding(binding.clone()))? {
            FormOwnershipNodeKey::TemplateControl(id) => Some(id),
            _ => None,
        }
    }

    #[must_use]
    pub fn bindings_of_field(&self, field: &FieldId) -> Vec<&FieldBindingId> {
        self.reference_edges
            .iter()
            .filter_map(|edge| {
                if edge.kind == FormReferenceKind::FieldBindingField
                    && edge.target == FormOwnershipNodeKey::FormField(field.clone())
                {
                    match &edge.source {
                        FormOwnershipNodeKey::FieldBinding(id) => Some(id),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    #[must_use]
    pub fn bindings_of_form(&self, form: &FormId) -> Vec<&FieldBindingId> {
        self.reference_edges
            .iter()
            .filter_map(|edge| {
                if edge.kind == FormReferenceKind::FieldBindingForm
                    && edge.target == FormOwnershipNodeKey::Form(form.clone())
                {
                    match &edge.source {
                        FormOwnershipNodeKey::FieldBinding(id) => Some(id),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    #[must_use]
    pub fn field_of_binding(&self, binding: &FieldBindingId) -> Option<&FieldId> {
        self.reference_edges.iter().find_map(|edge| {
            if edge.kind == FormReferenceKind::FieldBindingField
                && edge.source == FormOwnershipNodeKey::FieldBinding(binding.clone())
            {
                match &edge.target {
                    FormOwnershipNodeKey::FormField(id) => Some(id),
                    _ => None,
                }
            } else {
                None
            }
        })
    }

    #[must_use]
    pub fn form_of_binding(&self, binding: &FieldBindingId) -> Option<&FormId> {
        self.reference_edges.iter().find_map(|edge| {
            if edge.kind == FormReferenceKind::FieldBindingForm
                && edge.source == FormOwnershipNodeKey::FieldBinding(binding.clone())
            {
                match &edge.target {
                    FormOwnershipNodeKey::Form(id) => Some(id),
                    _ => None,
                }
            } else {
                None
            }
        })
    }

    #[must_use]
    pub fn component_of_form(&self, form: &FormId) -> Option<&SemanticId> {
        match self.owner_of(&FormOwnershipNodeKey::Form(form.clone()))? {
            FormOwnershipNodeKey::Component(id) => Some(id),
            _ => None,
        }
    }

    #[must_use]
    pub fn component_of_field(&self, field: &FieldId) -> Option<&SemanticId> {
        self.component_of_form(
            match self.owner_of(&FormOwnershipNodeKey::FormField(field.clone()))? {
                FormOwnershipNodeKey::Form(id) => id,
                _ => return None,
            },
        )
    }

    #[must_use]
    pub fn component_of_binding(&self, binding: &FieldBindingId) -> Option<&SemanticId> {
        self.component_of_form(self.form_of_binding(binding)?)
    }

    #[must_use]
    pub fn roots(&self) -> &[SemanticId] {
        &self.roots
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
#[must_use]
pub fn collect_form_ownership_graph(
    build_roots: &BTreeMap<ComponentRootId, ComponentBuildRoot>,
    components: &[ComponentNode],
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    bindings: &BTreeMap<FieldBindingId, FormFieldBinding>,
    template_entities: &[TemplateSemanticEntity],
    ownership: &BTreeMap<SemanticId, SemanticOwner>,
    references: &[SemanticReference],
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> FormOwnershipGraph {
    let id = FormOwnershipGraphId::for_build_roots(build_roots.keys());
    let component_ids = components
        .iter()
        .map(|component| component.id.clone())
        .collect::<BTreeSet<_>>();
    let participating_components = forms
        .values()
        .filter_map(|form| form.owner.entity_id().cloned())
        .chain(bindings.values().map(|binding| binding.component.clone()))
        .filter(|component| component_ids.contains(component))
        .collect::<BTreeSet<_>>();
    let template_entities = template_entities
        .iter()
        .map(|entity| (entity.id.clone(), entity))
        .collect::<BTreeMap<_, _>>();

    let mut nodes = BTreeMap::new();
    for component in &participating_components {
        if let Some(provenance) = provenance.get(component) {
            insert_node(
                &mut nodes,
                FormOwnershipNode::Component {
                    id: component.clone(),
                    provenance: provenance.clone(),
                },
            );
        }
    }
    for form in forms.values() {
        insert_node(
            &mut nodes,
            FormOwnershipNode::Form {
                id: form.id.clone(),
                provenance: form.provenance.clone(),
            },
        );
    }
    for field in fields.values() {
        insert_node(
            &mut nodes,
            FormOwnershipNode::FormField {
                id: field.id.clone(),
                provenance: field.provenance.clone(),
            },
        );
    }
    for binding in bindings.values() {
        if let Some(control) = template_entities.get(&binding.control_entity) {
            insert_node(
                &mut nodes,
                FormOwnershipNode::TemplateControl {
                    id: control.id.clone(),
                    provenance: control.provenance.clone(),
                },
            );
        }
        insert_node(
            &mut nodes,
            FormOwnershipNode::FieldBinding {
                id: binding.id.clone(),
                provenance: binding.provenance.clone(),
            },
        );
    }

    let mut ownership_edges = Vec::new();
    for form in forms.values() {
        push_ownership_edge(
            &mut ownership_edges,
            ownership,
            &nodes,
            form.id.as_semantic_id(),
            FormOwnershipEdgeKind::ComponentOwnsForm,
            form.provenance.clone(),
        );
    }
    for field in fields.values() {
        push_ownership_edge(
            &mut ownership_edges,
            ownership,
            &nodes,
            field.id.as_semantic_id(),
            FormOwnershipEdgeKind::FormOwnsField,
            field.provenance.clone(),
        );
    }
    for binding in bindings.values() {
        push_ownership_edge(
            &mut ownership_edges,
            ownership,
            &nodes,
            binding.id.as_semantic_id(),
            FormOwnershipEdgeKind::TemplateControlOwnsBinding,
            binding.attribute_provenance.clone(),
        );
    }
    sort_ownership_edges(&mut ownership_edges);

    let mut reference_edges = references
        .iter()
        .filter_map(|reference| form_reference_edge(reference, &nodes))
        .collect::<Vec<_>>();
    sort_reference_edges(&mut reference_edges);

    let roots = participating_components.into_iter().collect::<Vec<_>>();
    let mut graph = FormOwnershipGraph {
        id,
        nodes,
        ownership_edges,
        reference_edges,
        roots,
        validation: FormOwnershipValidation::default(),
    };
    graph.validation = validate_form_ownership_graph_sources(
        &graph,
        build_roots,
        components,
        forms,
        fields,
        bindings,
        template_entities.values().copied(),
        ownership,
        references,
        provenance,
        &BTreeSet::new(),
    );
    graph
}

#[must_use]
pub fn validate_form_ownership_graph(
    graph: &FormOwnershipGraph,
    model: &ApplicationSemanticModel,
) -> FormOwnershipValidation {
    let instance_ids = model
        .component_instance_plan
        .instances
        .keys()
        .map(|id| id.as_semantic_id().clone())
        .chain(
            model
                .component_instance_plan
                .blocked
                .keys()
                .map(|id| id.as_semantic_id().clone()),
        )
        .collect();
    validate_form_ownership_graph_sources(
        graph,
        &model.component_instance_plan.roots,
        &model.components,
        &model.forms,
        &model.form_fields,
        &model.form_field_bindings,
        model.template_entities.iter(),
        &model.ownership,
        &model.references,
        &model.provenance,
        &instance_ids,
    )
}

fn insert_node(
    nodes: &mut BTreeMap<FormOwnershipNodeKey, FormOwnershipNode>,
    node: FormOwnershipNode,
) {
    nodes.insert(node.key(), node);
}

fn node_key_for_semantic_id(
    nodes: &BTreeMap<FormOwnershipNodeKey, FormOwnershipNode>,
    id: &SemanticId,
) -> Option<FormOwnershipNodeKey> {
    nodes.keys().find(|key| key.semantic_id() == id).cloned()
}

fn push_ownership_edge(
    edges: &mut Vec<FormOwnershipEdge>,
    ownership: &BTreeMap<SemanticId, SemanticOwner>,
    nodes: &BTreeMap<FormOwnershipNodeKey, FormOwnershipNode>,
    child: &SemanticId,
    kind: FormOwnershipEdgeKind,
    provenance: SourceProvenance,
) {
    let Some(child) = node_key_for_semantic_id(nodes, child) else {
        return;
    };
    let Some(owner) = ownership
        .get(child.semantic_id())
        .and_then(SemanticOwner::entity_id)
        .and_then(|owner| node_key_for_semantic_id(nodes, owner))
    else {
        return;
    };
    edges.push(FormOwnershipEdge {
        owner,
        child,
        kind,
        provenance,
    });
}

fn form_reference_edge(
    reference: &SemanticReference,
    nodes: &BTreeMap<FormOwnershipNodeKey, FormOwnershipNode>,
) -> Option<FormReferenceEdge> {
    let kind = match reference.kind {
        SemanticReferenceKind::FieldBindingField => FormReferenceKind::FieldBindingField,
        SemanticReferenceKind::FieldBindingForm => FormReferenceKind::FieldBindingForm,
        _ => return None,
    };
    Some(FormReferenceEdge {
        kind,
        source: node_key_for_semantic_id(nodes, &reference.source)?,
        target: node_key_for_semantic_id(nodes, &reference.target)?,
        provenance: reference.provenance.clone(),
    })
}

fn sort_ownership_edges(edges: &mut [FormOwnershipEdge]) {
    edges.sort_by(|left, right| {
        (
            left.owner.semantic_id().as_str(),
            left.child.semantic_id().as_str(),
            left.kind,
        )
            .cmp(&(
                right.owner.semantic_id().as_str(),
                right.child.semantic_id().as_str(),
                right.kind,
            ))
    });
}

fn sort_reference_edges(edges: &mut [FormReferenceEdge]) {
    edges.sort_by(|left, right| {
        (
            left.source.semantic_id().as_str(),
            left.kind,
            left.target.semantic_id().as_str(),
        )
            .cmp(&(
                right.source.semantic_id().as_str(),
                right.kind,
                right.target.semantic_id().as_str(),
            ))
    });
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn validate_form_ownership_graph_sources<'a>(
    graph: &FormOwnershipGraph,
    build_roots: &BTreeMap<ComponentRootId, ComponentBuildRoot>,
    components: &[ComponentNode],
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    bindings: &BTreeMap<FieldBindingId, FormFieldBinding>,
    template_entities: impl IntoIterator<Item = &'a TemplateSemanticEntity>,
    ownership: &BTreeMap<SemanticId, SemanticOwner>,
    references: &[SemanticReference],
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
    instance_ids: &BTreeSet<SemanticId>,
) -> FormOwnershipValidation {
    let mut diagnostics = Vec::new();
    let template_entities = template_entities
        .into_iter()
        .map(|entity| (entity.id.clone(), entity))
        .collect::<BTreeMap<_, _>>();
    let components = components
        .iter()
        .map(|component| (component.id.clone(), component))
        .collect::<BTreeMap<_, _>>();

    if graph.id != FormOwnershipGraphId::for_build_roots(build_roots.keys()) {
        push_diagnostic(
            &mut diagnostics,
            FormOwnershipIntegrityKind::GraphIdentityMismatch,
            "Form ownership graph identity does not match the canonical build-root set",
        );
    }

    let mut semantic_ids = BTreeSet::new();
    for (key, node) in &graph.nodes {
        if node.key() != *key || !semantic_ids.insert(key.semantic_id().clone()) {
            push_diagnostic(
                &mut diagnostics,
                FormOwnershipIntegrityKind::DuplicateNode,
                format!(
                    "duplicate or mismatched Form ownership node `{}`",
                    key.semantic_id()
                ),
            );
        }
        if missing_provenance(node.provenance()) {
            push_diagnostic(
                &mut diagnostics,
                FormOwnershipIntegrityKind::MissingProvenance,
                format!(
                    "Form ownership node `{}` has no canonical provenance",
                    key.semantic_id()
                ),
            );
        }
        if instance_ids.contains(key.semantic_id()) {
            push_diagnostic(
                &mut diagnostics,
                FormOwnershipIntegrityKind::InstanceIdentityInDeclarationGraph,
                format!(
                    "instance identity `{}` leaked into the declaration graph",
                    key.semantic_id()
                ),
            );
        }
    }

    validate_node_domain(
        graph,
        &components,
        forms,
        fields,
        bindings,
        &template_entities,
        provenance,
        &mut diagnostics,
    );
    validate_edge_domains(graph, &mut diagnostics);

    for form in forms.values() {
        let child = FormOwnershipNodeKey::Form(form.id.clone());
        let expected_component = form.owner.entity_id();
        let expected_owner =
            expected_component.map(|id| FormOwnershipNodeKey::Component(id.clone()));
        validate_exact_owner(
            graph,
            &child,
            expected_owner.as_ref(),
            FormOwnershipEdgeKind::ComponentOwnsForm,
            FormOwnershipIntegrityKind::ComponentOwnershipMismatch,
            &mut diagnostics,
        );
        validate_ownership_provenance(graph, &child, &form.provenance, &mut diagnostics);
        if ownership.get(form.id.as_semantic_id()) != Some(&form.owner) {
            push_diagnostic(
                &mut diagnostics,
                FormOwnershipIntegrityKind::ComponentOwnershipMismatch,
                format!("Form `{}` disagrees with canonical ASM ownership", form.id),
            );
        }
    }

    for field in fields.values() {
        let child = FormOwnershipNodeKey::FormField(field.id.clone());
        let expected_owner = FormOwnershipNodeKey::Form(field.owner_form.clone());
        validate_exact_owner(
            graph,
            &child,
            Some(&expected_owner),
            FormOwnershipEdgeKind::FormOwnsField,
            FormOwnershipIntegrityKind::FieldFormMismatch,
            &mut diagnostics,
        );
        validate_ownership_provenance(graph, &child, &field.provenance, &mut diagnostics);
        let form_component = forms
            .get(&field.owner_form)
            .and_then(|form| form.owner.entity_id());
        if form_component != Some(&field.owner_component)
            || ownership.get(field.id.as_semantic_id())
                != Some(&SemanticOwner::entity(
                    field.owner_form.as_semantic_id().clone(),
                ))
        {
            push_diagnostic(
                &mut diagnostics,
                FormOwnershipIntegrityKind::FieldFormMismatch,
                format!(
                    "Field `{}` disagrees with its canonical Form owner",
                    field.id
                ),
            );
        }
    }

    for binding in bindings.values() {
        let child = FormOwnershipNodeKey::FieldBinding(binding.id.clone());
        let expected_owner = FormOwnershipNodeKey::TemplateControl(binding.control_entity.clone());
        validate_exact_owner(
            graph,
            &child,
            Some(&expected_owner),
            FormOwnershipEdgeKind::TemplateControlOwnsBinding,
            FormOwnershipIntegrityKind::BindingTemplateMismatch,
            &mut diagnostics,
        );
        validate_ownership_provenance(
            graph,
            &child,
            &binding.attribute_provenance,
            &mut diagnostics,
        );
        if ownership.get(binding.id.as_semantic_id())
            != Some(&SemanticOwner::entity(binding.control_entity.clone()))
        {
            push_diagnostic(
                &mut diagnostics,
                FormOwnershipIntegrityKind::BindingTemplateMismatch,
                format!(
                    "binding `{}` disagrees with canonical control ownership",
                    binding.id
                ),
            );
        }

        let expected_field = FormOwnershipNodeKey::FormField(binding.field.clone());
        let expected_form = FormOwnershipNodeKey::Form(binding.form.clone());
        validate_exact_reference(
            graph,
            &child,
            &expected_field,
            FormReferenceKind::FieldBindingField,
            FormOwnershipIntegrityKind::BindingFieldMismatch,
            &mut diagnostics,
        );
        validate_reference_provenance(
            graph,
            &child,
            FormReferenceKind::FieldBindingField,
            &binding.expression_provenance,
            &mut diagnostics,
        );
        validate_exact_reference(
            graph,
            &child,
            &expected_form,
            FormReferenceKind::FieldBindingForm,
            FormOwnershipIntegrityKind::BindingFormMismatch,
            &mut diagnostics,
        );
        validate_reference_provenance(
            graph,
            &child,
            FormReferenceKind::FieldBindingForm,
            &binding.expression_provenance,
            &mut diagnostics,
        );
        let field = fields.get(&binding.field);
        let form = forms.get(&binding.form);
        let template_component =
            component_owner_of(&binding.control_entity, &components, ownership);
        if field.is_none_or(|field| field.owner_form != binding.form) {
            push_diagnostic(
                &mut diagnostics,
                FormOwnershipIntegrityKind::BindingFieldMismatch,
                format!(
                    "binding `{}` references a Field with a different Form",
                    binding.id
                ),
            );
        }
        if form
            .and_then(|form| form.owner.entity_id())
            .is_none_or(|component| component != &binding.component)
        {
            push_diagnostic(
                &mut diagnostics,
                FormOwnershipIntegrityKind::BindingFormMismatch,
                format!(
                    "binding `{}` references a Form from another component",
                    binding.id
                ),
            );
        }
        if field.is_none_or(|field| field.owner_component != binding.component)
            || template_component.as_ref() != Some(&binding.component)
        {
            push_diagnostic(
                &mut diagnostics,
                FormOwnershipIntegrityKind::ComponentOwnershipMismatch,
                format!(
                    "binding `{}` crosses canonical component ownership",
                    binding.id
                ),
            );
        }
    }

    for edge in &graph.ownership_edges {
        if !graph.nodes.contains_key(&edge.owner) || !graph.nodes.contains_key(&edge.child) {
            push_diagnostic(
                &mut diagnostics,
                FormOwnershipIntegrityKind::UnknownOwnershipEndpoint,
                format!(
                    "ownership edge `{}` -> `{}` has an unknown endpoint",
                    edge.owner.semantic_id(),
                    edge.child.semantic_id()
                ),
            );
        }
        if missing_provenance(&edge.provenance) {
            push_diagnostic(
                &mut diagnostics,
                FormOwnershipIntegrityKind::MissingProvenance,
                format!(
                    "ownership edge to `{}` lacks provenance",
                    edge.child.semantic_id()
                ),
            );
        }
    }
    for edge in &graph.reference_edges {
        if !graph.nodes.contains_key(&edge.source) || !graph.nodes.contains_key(&edge.target) {
            push_diagnostic(
                &mut diagnostics,
                FormOwnershipIntegrityKind::UnknownReferenceEndpoint,
                format!(
                    "reference `{}` -> `{}` has an unknown endpoint",
                    edge.source.semantic_id(),
                    edge.target.semantic_id()
                ),
            );
        }
        if missing_provenance(&edge.provenance) {
            push_diagnostic(
                &mut diagnostics,
                FormOwnershipIntegrityKind::MissingProvenance,
                format!(
                    "reference from `{}` lacks provenance",
                    edge.source.semantic_id()
                ),
            );
        }
    }

    validate_parents_and_reachability(graph, &mut diagnostics);
    if ownership_has_cycle(graph) {
        push_diagnostic(
            &mut diagnostics,
            FormOwnershipIntegrityKind::OwnershipCycle,
            "Form ownership edges contain a cycle",
        );
    }

    let expected_roots = forms
        .values()
        .filter_map(|form| form.owner.entity_id().cloned())
        .chain(bindings.values().map(|binding| binding.component.clone()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if graph.roots != expected_roots || !is_strictly_sorted(&graph.roots) {
        push_diagnostic(
            &mut diagnostics,
            FormOwnershipIntegrityKind::NonCanonicalOrdering,
            "Form ownership roots are incomplete or non-canonical",
        );
    }
    let mut ownership_sorted = graph.ownership_edges.clone();
    sort_ownership_edges(&mut ownership_sorted);
    let mut references_sorted = graph.reference_edges.clone();
    sort_reference_edges(&mut references_sorted);
    if ownership_sorted != graph.ownership_edges
        || references_sorted != graph.reference_edges
        || has_duplicate_ownership_edge(&graph.ownership_edges)
        || has_duplicate_reference_edge(&graph.reference_edges)
    {
        push_diagnostic(
            &mut diagnostics,
            FormOwnershipIntegrityKind::NonCanonicalOrdering,
            "Form ownership edges or references are not canonically ordered and unique",
        );
    }

    // Prove that the graph references are a projection of the canonical ASM
    // relations, not an independently invented relation set.
    for binding in bindings.values() {
        for (kind, target) in [
            (
                SemanticReferenceKind::FieldBindingField,
                binding.field.as_semantic_id(),
            ),
            (
                SemanticReferenceKind::FieldBindingForm,
                binding.form.as_semantic_id(),
            ),
        ] {
            if references
                .iter()
                .filter(|reference| {
                    reference.kind == kind
                        && reference.source == *binding.id.as_semantic_id()
                        && reference.target == *target
                })
                .count()
                != 1
            {
                push_diagnostic(
                    &mut diagnostics,
                    if kind == SemanticReferenceKind::FieldBindingField {
                        FormOwnershipIntegrityKind::BindingFieldMismatch
                    } else {
                        FormOwnershipIntegrityKind::BindingFormMismatch
                    },
                    format!("binding `{}` lacks one canonical ASM reference", binding.id),
                );
            }
        }
    }

    diagnostics.sort_by(|left, right| {
        (&left.code, left.kind, &left.message).cmp(&(&right.code, right.kind, &right.message))
    });
    diagnostics.dedup();
    FormOwnershipValidation {
        is_valid: diagnostics.is_empty(),
        diagnostics,
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_node_domain(
    graph: &FormOwnershipGraph,
    components: &BTreeMap<SemanticId, &ComponentNode>,
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    bindings: &BTreeMap<FieldBindingId, FormFieldBinding>,
    templates: &BTreeMap<SemanticId, &TemplateSemanticEntity>,
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
    diagnostics: &mut Vec<FormOwnershipIntegrityDiagnostic>,
) {
    for (key, node) in &graph.nodes {
        let valid = match key {
            FormOwnershipNodeKey::Component(id) => {
                components.contains_key(id)
                    && (forms
                        .values()
                        .any(|form| form.owner.entity_id() == Some(id))
                        || bindings.values().any(|binding| binding.component == *id))
                    && provenance.get(id) == Some(node.provenance())
            }
            FormOwnershipNodeKey::Form(id) => forms
                .get(id)
                .is_some_and(|form| form.provenance == *node.provenance()),
            FormOwnershipNodeKey::FormField(id) => fields
                .get(id)
                .is_some_and(|field| field.provenance == *node.provenance()),
            FormOwnershipNodeKey::TemplateControl(id) => templates.get(id).is_some_and(|entity| {
                entity.kind == TemplateSemanticKind::Element
                    && entity.provenance == *node.provenance()
                    && bindings
                        .values()
                        .any(|binding| binding.control_entity == *id)
            }),
            FormOwnershipNodeKey::FieldBinding(id) => bindings
                .get(id)
                .is_some_and(|binding| binding.provenance == *node.provenance()),
        };
        if !valid {
            push_diagnostic(
                diagnostics,
                FormOwnershipIntegrityKind::InvalidCandidatePromoted,
                format!(
                    "node `{}` is not one valid canonical I2-I4 entity",
                    key.semantic_id()
                ),
            );
        }
    }

    for expected in
        forms
            .keys()
            .cloned()
            .map(FormOwnershipNodeKey::Form)
            .chain(fields.keys().cloned().map(FormOwnershipNodeKey::FormField))
            .chain(
                bindings
                    .keys()
                    .cloned()
                    .map(FormOwnershipNodeKey::FieldBinding),
            )
            .chain(bindings.values().map(|binding| {
                FormOwnershipNodeKey::TemplateControl(binding.control_entity.clone())
            }))
    {
        if !graph.nodes.contains_key(&expected) {
            push_diagnostic(
                diagnostics,
                FormOwnershipIntegrityKind::UnreachableNode,
                format!(
                    "canonical entity `{}` is absent from the Form ownership graph",
                    expected.semantic_id()
                ),
            );
        }
    }
}

fn validate_edge_domains(
    graph: &FormOwnershipGraph,
    diagnostics: &mut Vec<FormOwnershipIntegrityDiagnostic>,
) {
    for edge in &graph.ownership_edges {
        let valid = matches!(
            (&edge.owner, &edge.child, edge.kind),
            (
                FormOwnershipNodeKey::Component(_),
                FormOwnershipNodeKey::Form(_),
                FormOwnershipEdgeKind::ComponentOwnsForm
            ) | (
                FormOwnershipNodeKey::Form(_),
                FormOwnershipNodeKey::FormField(_),
                FormOwnershipEdgeKind::FormOwnsField
            ) | (
                FormOwnershipNodeKey::TemplateControl(_),
                FormOwnershipNodeKey::FieldBinding(_),
                FormOwnershipEdgeKind::TemplateControlOwnsBinding
            )
        );
        if !valid {
            push_diagnostic(
                diagnostics,
                FormOwnershipIntegrityKind::UnknownOwnershipEndpoint,
                format!(
                    "ownership edge to `{}` has an invalid typed domain",
                    edge.child.semantic_id()
                ),
            );
        }
    }
    for edge in &graph.reference_edges {
        let valid = matches!(
            (&edge.source, &edge.target, edge.kind),
            (
                FormOwnershipNodeKey::FieldBinding(_),
                FormOwnershipNodeKey::FormField(_),
                FormReferenceKind::FieldBindingField
            ) | (
                FormOwnershipNodeKey::FieldBinding(_),
                FormOwnershipNodeKey::Form(_),
                FormReferenceKind::FieldBindingForm
            )
        );
        if !valid {
            push_diagnostic(
                diagnostics,
                FormOwnershipIntegrityKind::UnknownReferenceEndpoint,
                format!(
                    "reference from `{}` has an invalid typed domain",
                    edge.source.semantic_id()
                ),
            );
        }
    }
}

fn validate_exact_owner(
    graph: &FormOwnershipGraph,
    child: &FormOwnershipNodeKey,
    expected: Option<&FormOwnershipNodeKey>,
    kind: FormOwnershipEdgeKind,
    mismatch_kind: FormOwnershipIntegrityKind,
    diagnostics: &mut Vec<FormOwnershipIntegrityDiagnostic>,
) {
    let parents = graph
        .ownership_edges
        .iter()
        .filter(|edge| &edge.child == child)
        .collect::<Vec<_>>();
    if parents.is_empty() {
        push_diagnostic(
            diagnostics,
            FormOwnershipIntegrityKind::MissingOwner,
            format!("`{}` has no ownership parent", child.semantic_id()),
        );
        return;
    }
    if parents.len() > 1 {
        push_diagnostic(
            diagnostics,
            FormOwnershipIntegrityKind::MultipleOwners,
            format!("`{}` has multiple ownership parents", child.semantic_id()),
        );
    }
    if expected.is_none_or(|expected| {
        parents.len() != 1 || parents[0].owner != *expected || parents[0].kind != kind
    }) {
        push_diagnostic(
            diagnostics,
            mismatch_kind,
            format!("`{}` has a non-canonical direct owner", child.semantic_id()),
        );
    }
}

fn validate_exact_reference(
    graph: &FormOwnershipGraph,
    source: &FormOwnershipNodeKey,
    target: &FormOwnershipNodeKey,
    kind: FormReferenceKind,
    mismatch_kind: FormOwnershipIntegrityKind,
    diagnostics: &mut Vec<FormOwnershipIntegrityDiagnostic>,
) {
    if graph
        .reference_edges
        .iter()
        .filter(|edge| &edge.source == source && edge.kind == kind)
        .count()
        != 1
        || !graph
            .reference_edges
            .iter()
            .any(|edge| &edge.source == source && &edge.target == target && edge.kind == kind)
    {
        push_diagnostic(
            diagnostics,
            mismatch_kind,
            format!("`{}` has a non-canonical reference", source.semantic_id()),
        );
    }
}

fn validate_ownership_provenance(
    graph: &FormOwnershipGraph,
    child: &FormOwnershipNodeKey,
    expected: &SourceProvenance,
    diagnostics: &mut Vec<FormOwnershipIntegrityDiagnostic>,
) {
    if graph
        .ownership_edges
        .iter()
        .find(|edge| &edge.child == child)
        .is_none_or(|edge| &edge.provenance != expected)
    {
        push_diagnostic(
            diagnostics,
            FormOwnershipIntegrityKind::MissingProvenance,
            format!(
                "ownership edge to `{}` lacks canonical child/use-site provenance",
                child.semantic_id()
            ),
        );
    }
}

fn validate_reference_provenance(
    graph: &FormOwnershipGraph,
    source: &FormOwnershipNodeKey,
    kind: FormReferenceKind,
    expected: &SourceProvenance,
    diagnostics: &mut Vec<FormOwnershipIntegrityDiagnostic>,
) {
    if graph
        .reference_edges
        .iter()
        .find(|edge| &edge.source == source && edge.kind == kind)
        .is_none_or(|edge| &edge.provenance != expected)
    {
        push_diagnostic(
            diagnostics,
            FormOwnershipIntegrityKind::MissingProvenance,
            format!(
                "reference from `{}` lacks canonical binding-expression provenance",
                source.semantic_id()
            ),
        );
    }
}

fn validate_parents_and_reachability(
    graph: &FormOwnershipGraph,
    diagnostics: &mut Vec<FormOwnershipIntegrityDiagnostic>,
) {
    let root_keys = graph
        .roots
        .iter()
        .cloned()
        .map(FormOwnershipNodeKey::Component)
        .collect::<Vec<_>>();
    for key in graph.nodes.keys() {
        match key {
            FormOwnershipNodeKey::Form(_) | FormOwnershipNodeKey::FormField(_) => {
                let reachable = root_keys
                    .iter()
                    .filter(|root| is_reachable(graph, root, key))
                    .count();
                if reachable != 1 {
                    push_diagnostic(
                        diagnostics,
                        FormOwnershipIntegrityKind::UnreachableNode,
                        format!(
                            "`{}` is reachable from {reachable} component roots",
                            key.semantic_id()
                        ),
                    );
                }
            }
            FormOwnershipNodeKey::FieldBinding(_) => {
                let controls = graph
                    .nodes
                    .keys()
                    .filter(|candidate| {
                        matches!(candidate, FormOwnershipNodeKey::TemplateControl(_))
                    })
                    .filter(|control| is_reachable(graph, control, key))
                    .count();
                if controls != 1 {
                    push_diagnostic(
                        diagnostics,
                        FormOwnershipIntegrityKind::UnreachableNode,
                        format!(
                            "binding `{}` is reachable from {controls} controls",
                            key.semantic_id()
                        ),
                    );
                }
            }
            FormOwnershipNodeKey::Component(_) | FormOwnershipNodeKey::TemplateControl(_) => {}
        }
    }
}

fn is_reachable(
    graph: &FormOwnershipGraph,
    start: &FormOwnershipNodeKey,
    target: &FormOwnershipNodeKey,
) -> bool {
    let mut pending = vec![start.clone()];
    let mut seen = BTreeSet::new();
    while let Some(current) = pending.pop() {
        if !seen.insert(current.clone()) {
            continue;
        }
        if current == *target {
            return true;
        }
        pending.extend(graph.children_of(&current).into_iter().cloned());
    }
    false
}

fn ownership_has_cycle(graph: &FormOwnershipGraph) -> bool {
    graph.nodes.keys().any(|start| {
        let mut pending = graph
            .children_of(start)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        let mut seen = BTreeSet::new();
        while let Some(current) = pending.pop() {
            if current == *start {
                return true;
            }
            if seen.insert(current.clone()) {
                pending.extend(graph.children_of(&current).into_iter().cloned());
            }
        }
        false
    })
}

fn component_owner_of(
    id: &SemanticId,
    components: &BTreeMap<SemanticId, &ComponentNode>,
    ownership: &BTreeMap<SemanticId, SemanticOwner>,
) -> Option<SemanticId> {
    let mut current = id;
    let mut seen = BTreeSet::new();
    loop {
        if components.contains_key(current) {
            return Some(current.clone());
        }
        if !seen.insert(current.clone()) {
            return None;
        }
        current = ownership.get(current)?.entity_id()?;
    }
}

fn missing_provenance(provenance: &SourceProvenance) -> bool {
    provenance.path.as_os_str().is_empty() || provenance.span.start >= provenance.span.end
}

fn is_strictly_sorted<T: Ord>(items: &[T]) -> bool {
    items.windows(2).all(|window| window[0] < window[1])
}

fn has_duplicate_ownership_edge(edges: &[FormOwnershipEdge]) -> bool {
    let mut seen = BTreeSet::new();
    edges
        .iter()
        .any(|edge| !seen.insert((edge.owner.clone(), edge.child.clone(), edge.kind)))
}

fn has_duplicate_reference_edge(edges: &[FormReferenceEdge]) -> bool {
    let mut seen = BTreeSet::new();
    edges
        .iter()
        .any(|edge| !seen.insert((edge.source.clone(), edge.kind, edge.target.clone())))
}

fn push_diagnostic(
    diagnostics: &mut Vec<FormOwnershipIntegrityDiagnostic>,
    kind: FormOwnershipIntegrityKind,
    message: impl Into<String>,
) {
    diagnostics.push(FormOwnershipIntegrityDiagnostic {
        code: kind.code().to_string(),
        kind,
        message: message.into(),
    });
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use ezc_parser::SourceSpan;

    use super::{
        validate_form_ownership_graph, FormOwnershipEdge, FormOwnershipEdgeKind,
        FormOwnershipIntegrityKind, FormOwnershipNode, FormOwnershipNodeKey, FormReferenceEdge,
        FormReferenceKind,
    };
    use crate::{
        build_application_semantic_model, build_application_semantic_model_for_unit,
        build_semantic_graph, validate_application_semantic_model, CompilationUnit, FieldId,
        FormId, FormInstanceId, SemanticId, SourceProvenance, SEMANTIC_GRAPH_SCHEMA_VERSION,
    };

    fn form_source() -> &'static str {
        r#"
@component("profile-editor")
class ProfileEditor {
  @form() profileForm!: Form;
  @form() passwordForm!: Form;
  @field(this.profileForm) displayName = "Austin";
  @field(this.profileForm) contactMethod: "email" | "phone" = "email";
  @field(this.passwordForm) password = "";
  render() {
    return <main>
      <input field={this.displayName} />
      <input type="radio" value="email" field={this.contactMethod} />
      <input type="radio" value="phone" field={this.contactMethod} />
    </main>;
  }
}
"#
    }

    #[test]
    fn projects_exact_declaration_and_control_ownership_with_read_only_queries() {
        let parsed = ezc_parser::parse_file("src/ProfileEditor.tsx", form_source());
        let asm = build_application_semantic_model(&parsed);
        let graph = asm.form_ownership();
        let component = &asm.components[0].id;
        let profile = FormId::for_owner(component, "profileForm");
        let password = FormId::for_owner(component, "passwordForm");
        let display = FieldId::for_form(&profile, "displayName");
        let contact = FieldId::for_form(&profile, "contactMethod");

        assert!(
            graph.validation.is_valid,
            "{:?}",
            graph.validation.diagnostics
        );
        assert!(validate_application_semantic_model(&asm).is_empty());
        assert_eq!(graph.roots(), std::slice::from_ref(component));
        assert_eq!(graph.forms_of(component), vec![&password, &profile]);
        assert_eq!(graph.fields_of(&profile), vec![&contact, &display]);
        assert_eq!(graph.bindings_of_field(&contact).len(), 2);
        assert_eq!(graph.bindings_of_form(&profile).len(), 3);
        assert_eq!(graph.ownership_edges.len(), 8);
        assert_eq!(graph.reference_edges.len(), 6);

        for binding in asm.form_field_bindings.values() {
            assert_eq!(
                graph.binding_owner(&binding.id),
                Some(&binding.control_entity)
            );
            assert_eq!(graph.field_of_binding(&binding.id), Some(&binding.field));
            assert_eq!(graph.form_of_binding(&binding.id), Some(&binding.form));
            assert_eq!(graph.component_of_binding(&binding.id), Some(component));
            assert_eq!(
                asm.parent_of(binding.id.as_semantic_id()),
                Some(&binding.control_entity)
            );
            assert!(!graph
                .children_of(&FormOwnershipNodeKey::Form(binding.form.clone()))
                .contains(&&FormOwnershipNodeKey::FieldBinding(binding.id.clone())));
            assert!(!graph
                .children_of(&FormOwnershipNodeKey::FormField(binding.field.clone()))
                .contains(&&FormOwnershipNodeKey::FieldBinding(binding.id.clone())));
        }

        assert!(graph.id.as_str().starts_with("form-ownership-graph:root:"));
        assert!(graph.nodes.values().all(|node| {
            !node.provenance().path.as_os_str().is_empty()
                && node.provenance().span.start < node.provenance().span.end
        }));
        assert!(graph.nodes.keys().all(|key| {
            !asm.component_instance_plan
                .instances
                .keys()
                .any(|instance| instance.as_semantic_id() == key.semantic_id())
        }));
    }

    #[test]
    fn excludes_invalid_candidates_without_poisoning_valid_products() {
        let parsed = ezc_parser::parse_file(
            "src/Mixed.tsx",
            r#"
@component("mixed-editor")
class MixedEditor {
  @form("invalid") brokenForm!: Form;
  @form() validForm!: Form;
  @field(this.brokenForm) broken = "";
  @field(this.validForm) valid = "ok";
  render() {
    return <main>
      <input field={this.broken} />
      <input field={this.valid} />
      <div field={this.valid} />
    </main>;
  }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let graph = asm.form_ownership();
        let component = &asm.components[0].id;
        let valid_form = FormId::for_owner(component, "validForm");
        let valid_field = FieldId::for_form(&valid_form, "valid");

        assert!(
            graph.validation.is_valid,
            "{:?}",
            graph.validation.diagnostics
        );
        assert_eq!(asm.forms.len(), 1);
        assert_eq!(asm.form_fields.len(), 1);
        assert_eq!(asm.form_field_bindings.len(), 1);
        assert!(graph
            .node(&FormOwnershipNodeKey::Form(valid_form))
            .is_some());
        assert!(graph
            .node(&FormOwnershipNodeKey::FormField(valid_field))
            .is_some());
        assert!(asm
            .form_declaration_candidates()
            .iter()
            .any(|candidate| !candidate.violations().is_empty()));
        assert!(asm
            .form_field_declaration_candidates()
            .iter()
            .any(|candidate| !candidate.is_valid()));
        assert!(asm
            .form_field_binding_candidates()
            .iter()
            .any(|candidate| !candidate.is_valid()));
    }

    #[test]
    fn validates_malformed_graphs_without_repairing_them() {
        let parsed = ezc_parser::parse_file("src/ProfileEditor.tsx", form_source());
        let asm = build_application_semantic_model(&parsed);
        let mut graph = asm.form_ownership.clone();
        let form = asm.forms.values().next().expect("Form");
        let field = asm.form_fields.values().next().expect("Field");
        let binding = asm.form_field_bindings.values().next().expect("binding");

        graph
            .ownership_edges
            .retain(|edge| edge.child != FormOwnershipNodeKey::FieldBinding(binding.id.clone()));
        graph.ownership_edges.push(FormOwnershipEdge {
            owner: FormOwnershipNodeKey::FormField(field.id.clone()),
            child: FormOwnershipNodeKey::Form(form.id.clone()),
            kind: FormOwnershipEdgeKind::ComponentOwnsForm,
            provenance: form.provenance.clone(),
        });
        graph.ownership_edges.push(FormOwnershipEdge {
            owner: FormOwnershipNodeKey::Component(SemanticId::component_in_module(
                "src/Missing.tsx",
                Some("missing-owner"),
                "MissingOwner",
            )),
            child: FormOwnershipNodeKey::Form(form.id.clone()),
            kind: FormOwnershipEdgeKind::ComponentOwnsForm,
            provenance: form.provenance.clone(),
        });
        graph.ownership_edges.push(FormOwnershipEdge {
            owner: FormOwnershipNodeKey::FormField(field.id.clone()),
            child: FormOwnershipNodeKey::Form(form.id.clone()),
            kind: FormOwnershipEdgeKind::ComponentOwnsForm,
            provenance: form.provenance.clone(),
        });
        let fake_form = FormId::for_owner(&asm.components[0].id, "notCanonical");
        let fake_field = FieldId::for_form(&fake_form, "missing");
        graph.reference_edges.push(FormReferenceEdge {
            kind: FormReferenceKind::FieldBindingField,
            source: FormOwnershipNodeKey::FieldBinding(binding.id.clone()),
            target: FormOwnershipNodeKey::FormField(fake_field),
            provenance: binding.expression_provenance.clone(),
        });
        let node = graph
            .nodes
            .get_mut(&FormOwnershipNodeKey::FormField(field.id.clone()))
            .expect("field node");
        if let FormOwnershipNode::FormField { provenance, .. } = node {
            *provenance = SourceProvenance::new(
                "",
                SourceSpan {
                    start: 0,
                    end: 0,
                    line: 0,
                    column: 0,
                },
            );
        }
        let instance = asm
            .component_instance_plan
            .instances
            .values()
            .next()
            .expect("component instance");
        graph.nodes.insert(
            FormOwnershipNodeKey::Component(instance.id.as_semantic_id().clone()),
            FormOwnershipNode::Component {
                id: instance.id.as_semantic_id().clone(),
                provenance: instance.provenance.clone(),
            },
        );

        let validation = validate_form_ownership_graph(&graph, &asm);
        let kinds = validation
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.kind)
            .collect::<BTreeSet<_>>();
        assert!(!validation.is_valid);
        assert!(kinds.contains(&FormOwnershipIntegrityKind::MissingOwner));
        assert!(kinds.contains(&FormOwnershipIntegrityKind::MultipleOwners));
        assert!(kinds.contains(&FormOwnershipIntegrityKind::OwnershipCycle));
        assert!(kinds.contains(&FormOwnershipIntegrityKind::UnknownOwnershipEndpoint));
        assert!(kinds.contains(&FormOwnershipIntegrityKind::UnknownReferenceEndpoint));
        assert!(kinds.contains(&FormOwnershipIntegrityKind::MissingProvenance));
        assert!(kinds.contains(&FormOwnershipIntegrityKind::InstanceIdentityInDeclarationGraph));
        assert!(kinds.contains(&FormOwnershipIntegrityKind::NonCanonicalOrdering));
        assert_eq!(
            graph.ownership_edges.len(),
            asm.form_ownership.ownership_edges.len() + 2
        );
    }

    #[test]
    fn validates_product_reciprocity_and_component_mismatches() {
        let parsed = ezc_parser::parse_file("src/ProfileEditor.tsx", form_source());
        let asm = build_application_semantic_model(&parsed);
        let mut malformed = asm.clone();
        let form_ids = malformed.forms.keys().cloned().collect::<Vec<_>>();
        let field_id = malformed.form_fields.keys().next().expect("Field").clone();
        let binding_id = malformed
            .form_field_bindings
            .keys()
            .next()
            .expect("binding")
            .clone();
        malformed
            .form_fields
            .get_mut(&field_id)
            .expect("Field")
            .owner_form = form_ids[1].clone();
        let binding = malformed
            .form_field_bindings
            .get_mut(&binding_id)
            .expect("binding");
        binding.form = form_ids[1].clone();
        binding.component =
            SemanticId::component_in_module("src/Other.tsx", Some("other-editor"), "OtherEditor");

        let validation = validate_form_ownership_graph(&malformed.form_ownership, &malformed);
        let kinds = validation
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.kind)
            .collect::<BTreeSet<_>>();
        assert!(kinds.contains(&FormOwnershipIntegrityKind::FieldFormMismatch));
        assert!(kinds.contains(&FormOwnershipIntegrityKind::BindingFormMismatch));
        assert!(kinds.contains(&FormOwnershipIntegrityKind::ComponentOwnershipMismatch));
    }

    #[test]
    fn is_byte_stable_under_repeated_and_reversed_multi_file_construction() {
        let first = r#"
@component("first-editor")
class FirstEditor {
  @form() editor!: Form;
  @field(this.editor) value = "first";
  render() { return <input field={this.value} />; }
}
"#;
        let second = r#"
@component("second-editor")
class SecondEditor {
  @form() editor!: Form;
  @field(this.editor) value = "second";
  render() { return <textarea field={this.value} />; }
}
"#;
        let ordered =
            CompilationUnit::parse_sources([("src/First.tsx", first), ("src/Second.tsx", second)]);
        let reversed =
            CompilationUnit::parse_sources([("src/Second.tsx", second), ("src/First.tsx", first)]);
        let ordered = build_application_semantic_model_for_unit(&ordered);
        let repeated =
            build_application_semantic_model_for_unit(&CompilationUnit::parse_sources([
                ("src/First.tsx", first),
                ("src/Second.tsx", second),
            ]));
        let reversed = build_application_semantic_model_for_unit(&reversed);

        assert_eq!(ordered.form_ownership, repeated.form_ownership);
        assert_eq!(ordered.form_ownership, reversed.form_ownership);
        assert_eq!(ordered.form_ownership.roots.len(), 2);
        assert_eq!(ordered.form_ownership.nodes.len(), 10);
        assert!(ordered.form_ownership.validation.is_valid);
    }

    #[test]
    fn keeps_repeated_component_instances_out_of_the_declaration_graph() {
        let parsed = ezc_parser::parse_file(
            "src/App.tsx",
            r#"
@component("profile-editor")
class ProfileEditor {
  @form() profile!: Form;
  @field(this.profile) name = "";
  render() { return <input field={this.name} />; }
}

@component("app-root")
class AppRoot {
  render() {
    return <main><ProfileEditor /><ProfileEditor /></main>;
  }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let editor = asm
            .components
            .iter()
            .find(|component| component.element_name.as_deref() == Some("profile-editor"))
            .expect("ProfileEditor");
        let instances = asm
            .component_instances_for_definition(&editor.id)
            .into_iter()
            .collect::<Vec<_>>();
        let form = FormId::for_owner(&editor.id, "profile");

        assert_eq!(instances.len(), 2);
        assert_eq!(
            asm.form_ownership
                .nodes
                .keys()
                .filter(|key| matches!(key, FormOwnershipNodeKey::Form(_)))
                .count(),
            1
        );
        let first = FormInstanceId::for_component_instance(&instances[0].id, &form);
        let second = FormInstanceId::for_component_instance(&instances[1].id, &form);
        assert_ne!(first, second);
        assert_eq!(
            first,
            FormInstanceId::for_component_instance(&instances[0].id, &form)
        );
        assert!(asm.form_ownership.nodes.keys().all(|key| {
            key.semantic_id() != first.as_semantic_id()
                && key.semantic_id() != second.as_semantic_id()
        }));
        assert!(asm.form_ownership.validation.is_valid);
    }

    #[test]
    fn projects_form_ownership_products_into_semantic_graph_v6() {
        let parsed = ezc_parser::parse_file("src/ProfileEditor.tsx", form_source());
        let asm = build_application_semantic_model(&parsed);
        let public = build_semantic_graph(&asm);

        assert_eq!(public.schema_version, SEMANTIC_GRAPH_SCHEMA_VERSION);
        assert_eq!(SEMANTIC_GRAPH_SCHEMA_VERSION, 6);
        assert!(public
            .nodes
            .iter()
            .any(|node| matches!(node.kind, crate::SemanticGraphNodeKind::Form)));
        assert!(public
            .edges
            .iter()
            .any(|edge| matches!(edge.kind, crate::SemanticGraphEdgeKind::FormOwnsField)));
    }
}
