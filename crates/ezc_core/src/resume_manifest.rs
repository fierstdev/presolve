use serde::Serialize;

use crate::resume_plan::ResumePlan;

pub const RESUME_MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResumeManifest {
    pub schema_version: u32,
    pub components: Vec<crate::resume_plan::ResumeComponentPlan>,
}

#[must_use]
pub fn build_resume_manifest(plan: &ResumePlan) -> ResumeManifest {
    ResumeManifest {
        schema_version: RESUME_MANIFEST_SCHEMA_VERSION,
        components: plan.components.clone(),
    }
}

#[must_use]
///
/// # Panics
///
/// Panics when the compiler-owned resume manifest cannot serialize.
pub fn resume_manifest_json(manifest: &ResumeManifest) -> String {
    serde_json::to_string_pretty(manifest).expect("resume manifest should serialize")
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_resume_manifest, build_resume_plan,
        resume_manifest_json,
    };

    #[test]
    fn serializes_planned_computed_cache_metadata() {
        let parsed = ezc_parser::parse_file(
            "src/ResumeManifestComputed.tsx",
            r#"
@component("x-resume-manifest-computed")
class ResumeManifestComputed extends Component {
  count = state(1);

  @computed()
  get doubled() { return this.count * 2; }
}
"#,
        );
        let model = build_application_semantic_model(&parsed);
        let doubled = model.components[0].id.computed("doubled");
        let manifest = build_resume_manifest(&build_resume_plan(&model));
        let json: serde_json::Value =
            serde_json::from_str(&resume_manifest_json(&manifest)).expect("resume manifest JSON");

        assert_eq!(json["schema_version"], 1);
        assert_eq!(
            json["components"][0]["computed"][0]["computed"],
            doubled.as_str()
        );
        assert_eq!(
            json["components"][0]["computed"][0]["cache_slot"],
            format!("{doubled}/runtime:cache")
        );
        assert_eq!(json["components"][0]["computed"][0]["initial_dirty"], true);
    }
}
