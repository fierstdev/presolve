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
            Self::MissingTarget => "PSASM1289",
            Self::DuplicateTarget => "PSASM1290",
            Self::DuplicateBinding => "PSASM1291",
            Self::TargetConstructorMismatch => "PSASM1292",
            Self::BindingConstructorMismatch => "PSASM1293",
            Self::TargetComponentMismatch => "PSASM1294",
            Self::DeclarationOwnershipMismatch => "PSASM1295",
            Self::EventActionBatchOwnershipMismatch => "PSASM1296",
            Self::ArtifactManifestDrift => "PSASM1297",
            Self::DomMarkerProjectionMismatch => "PSASM1298",
            Self::FormTargetReciprocityMismatch => "PSASM1299",
            Self::StructuralProjectionMismatch => "PSASM1300",
            Self::VersionPairMismatch => "PSASM1301",
            Self::LegacyRecordInPhaseJPath => "PSASM1302",
            Self::MissingRuntimeComponentInstance => "PSASM1303",
            Self::StaleRegistry => "PSASM1304",
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
            Self::MissingProjection => "PSASM1305",
            Self::DuplicateCacheSlot => "PSASM1306",
            Self::DuplicateDirtySlot => "PSASM1307",
            Self::ConstructorMismatch => "PSASM1308",
            Self::OwnershipMismatch => "PSASM1309",
            Self::StaleRegistry => "PSASM1310",
            Self::ArtifactProjectionDrift => "PSASM1311",
        }
    }
}

/// J1-A consumes the next reserved integrity codes before J2 liveness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateInstanceStorageIntegrityCode {
    MissingProjection,
    DuplicateSlot,
    ConstructorMismatch,
    OwnershipMismatch,
    StaleRegistry,
    ArtifactProjectionDrift,
    DeclarationRuntimeKey,
    UnknownResumeSlot,
}

impl StateInstanceStorageIntegrityCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingProjection => "PSASM1312",
            Self::DuplicateSlot => "PSASM1313",
            Self::ConstructorMismatch => "PSASM1314",
            Self::OwnershipMismatch => "PSASM1315",
            Self::StaleRegistry => "PSASM1316",
            Self::ArtifactProjectionDrift => "PSASM1317",
            Self::DeclarationRuntimeKey => "PSASM1318",
            Self::UnknownResumeSlot => "PSASM1319",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ComputedInstanceSlotIntegrityCode, OrdinaryTemplateIntegrityCode,
        StateInstanceStorageIntegrityCode,
    };

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
        assert_eq!(codes.first().map(|code| code.as_str()), Some("PSASM1289"));
        assert_eq!(codes.last().map(|code| code.as_str()), Some("PSASM1304"));
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
        assert_eq!(codes.first().map(|code| code.as_str()), Some("PSASM1305"));
        assert_eq!(codes.last().map(|code| code.as_str()), Some("PSASM1311"));
    }

    #[test]
    fn reserves_j1a_codes_before_j2_in_amendment_order() {
        let codes = [
            StateInstanceStorageIntegrityCode::MissingProjection,
            StateInstanceStorageIntegrityCode::DuplicateSlot,
            StateInstanceStorageIntegrityCode::ConstructorMismatch,
            StateInstanceStorageIntegrityCode::OwnershipMismatch,
            StateInstanceStorageIntegrityCode::StaleRegistry,
            StateInstanceStorageIntegrityCode::ArtifactProjectionDrift,
            StateInstanceStorageIntegrityCode::DeclarationRuntimeKey,
            StateInstanceStorageIntegrityCode::UnknownResumeSlot,
        ];
        assert_eq!(codes.first().map(|code| code.as_str()), Some("PSASM1312"));
        assert_eq!(codes.last().map(|code| code.as_str()), Some("PSASM1319"));
    }
}
