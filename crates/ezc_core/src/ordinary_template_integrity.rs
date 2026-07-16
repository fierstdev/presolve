//! Internal-only J1-P integrity allocation.
//!
//! These findings are deliberately not public compiler diagnostics. J19 is the
//! first slice authorized to project Phase-J products into public diagnostics.

/// The first contiguous Phase-J internal allocation, in the amendment's
/// required order. J2 begins after this closed J1-P range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrdinaryTemplateIntegrityCode {
    MissingTarget,
    DuplicateTarget,
    DuplicateBinding,
    TargetConstructorMismatch,
    BindingConstructorMismatch,
    TargetComponentMismatch,
    DeclarationOwnershipMismatch,
    EventActionBatchOwnershipMismatch,
    ArtifactManifestDrift,
    DomMarkerProjectionMismatch,
    FormTargetReciprocityMismatch,
    StructuralProjectionMismatch,
    VersionPairMismatch,
    LegacyRecordInPhaseJPath,
    MissingRuntimeComponentInstance,
    StaleRegistry,
}

impl OrdinaryTemplateIntegrityCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingTarget => "EZASM1289",
            Self::DuplicateTarget => "EZASM1290",
            Self::DuplicateBinding => "EZASM1291",
            Self::TargetConstructorMismatch => "EZASM1292",
            Self::BindingConstructorMismatch => "EZASM1293",
            Self::TargetComponentMismatch => "EZASM1294",
            Self::DeclarationOwnershipMismatch => "EZASM1295",
            Self::EventActionBatchOwnershipMismatch => "EZASM1296",
            Self::ArtifactManifestDrift => "EZASM1297",
            Self::DomMarkerProjectionMismatch => "EZASM1298",
            Self::FormTargetReciprocityMismatch => "EZASM1299",
            Self::StructuralProjectionMismatch => "EZASM1300",
            Self::VersionPairMismatch => "EZASM1301",
            Self::LegacyRecordInPhaseJPath => "EZASM1302",
            Self::MissingRuntimeComponentInstance => "EZASM1303",
            Self::StaleRegistry => "EZASM1304",
        }
    }
}

/// J1-C consumes the next reserved integrity codes before State addressing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComputedInstanceSlotIntegrityCode {
    MissingProjection,
    DuplicateCacheSlot,
    DuplicateDirtySlot,
    ConstructorMismatch,
    OwnershipMismatch,
    StaleRegistry,
    ArtifactProjectionDrift,
}

impl ComputedInstanceSlotIntegrityCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingProjection => "EZASM1305",
            Self::DuplicateCacheSlot => "EZASM1306",
            Self::DuplicateDirtySlot => "EZASM1307",
            Self::ConstructorMismatch => "EZASM1308",
            Self::OwnershipMismatch => "EZASM1309",
            Self::StaleRegistry => "EZASM1310",
            Self::ArtifactProjectionDrift => "EZASM1311",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ComputedInstanceSlotIntegrityCode, OrdinaryTemplateIntegrityCode};

    #[test]
    fn reserves_j1p_codes_before_j2_in_amendment_order() {
        let codes = [
            OrdinaryTemplateIntegrityCode::MissingTarget,
            OrdinaryTemplateIntegrityCode::DuplicateTarget,
            OrdinaryTemplateIntegrityCode::DuplicateBinding,
            OrdinaryTemplateIntegrityCode::TargetConstructorMismatch,
            OrdinaryTemplateIntegrityCode::BindingConstructorMismatch,
            OrdinaryTemplateIntegrityCode::TargetComponentMismatch,
            OrdinaryTemplateIntegrityCode::DeclarationOwnershipMismatch,
            OrdinaryTemplateIntegrityCode::EventActionBatchOwnershipMismatch,
            OrdinaryTemplateIntegrityCode::ArtifactManifestDrift,
            OrdinaryTemplateIntegrityCode::DomMarkerProjectionMismatch,
            OrdinaryTemplateIntegrityCode::FormTargetReciprocityMismatch,
            OrdinaryTemplateIntegrityCode::StructuralProjectionMismatch,
            OrdinaryTemplateIntegrityCode::VersionPairMismatch,
            OrdinaryTemplateIntegrityCode::LegacyRecordInPhaseJPath,
            OrdinaryTemplateIntegrityCode::MissingRuntimeComponentInstance,
            OrdinaryTemplateIntegrityCode::StaleRegistry,
        ];
        assert_eq!(codes.first().map(|code| code.as_str()), Some("EZASM1289"));
        assert_eq!(codes.last().map(|code| code.as_str()), Some("EZASM1304"));
    }

    #[test]
    fn reserves_j1c_codes_before_state_storage_in_amendment_order() {
        let codes = [
            ComputedInstanceSlotIntegrityCode::MissingProjection,
            ComputedInstanceSlotIntegrityCode::DuplicateCacheSlot,
            ComputedInstanceSlotIntegrityCode::DuplicateDirtySlot,
            ComputedInstanceSlotIntegrityCode::ConstructorMismatch,
            ComputedInstanceSlotIntegrityCode::OwnershipMismatch,
            ComputedInstanceSlotIntegrityCode::StaleRegistry,
            ComputedInstanceSlotIntegrityCode::ArtifactProjectionDrift,
        ];
        assert_eq!(codes.first().map(|code| code.as_str()), Some("EZASM1305"));
        assert_eq!(codes.last().map(|code| code.as_str()), Some("EZASM1311"));
    }
}
