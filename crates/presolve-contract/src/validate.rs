use std::collections::HashSet;

use presolve_core::{Diagnostic, DiagnosticCode, DiagnosticSeverity};

use crate::{ApplicationContract, CONTRACT_VERSION, ContractDiagnostics, OutboundNetworkMode};

/// Stable numeric codes emitted by Application Contract validation.
///
/// The `PS1xxx` range is reserved for Application Contract diagnostics.
pub mod diagnostic_codes {
    /// The source file could not be parsed.
    pub const PARSE_ERROR: u16 = 1_001;

    /// The declared contract version is unsupported.
    pub const UNSUPPORTED_VERSION: u16 = 1_002;

    /// The application name is invalid.
    pub const INVALID_APPLICATION_NAME: u16 = 1_003;

    /// A capability interface identifier is invalid.
    pub const INVALID_CAPABILITY_INTERFACE: u16 = 1_004;

    /// The same capability interface is declared more than once.
    pub const DUPLICATE_CAPABILITY: u16 = 1_005;

    /// A compute resource requirement is invalid.
    pub const INVALID_RESOURCE_REQUIREMENT: u16 = 1_006;

    /// The outbound network policy is internally inconsistent.
    pub const INVALID_NETWORK_POLICY: u16 = 1_007;

    /// A workload name is invalid.
    pub const INVALID_WORKLOAD_NAME: u16 = 1_008;

    /// The same workload name is declared more than once.
    pub const DUPLICATE_WORKLOAD: u16 = 1_009;

    /// An application-internal interface name is invalid.
    pub const INVALID_INTERFACE_NAME: u16 = 1_010;

    /// The same application-internal interface is declared more than once.
    pub const DUPLICATE_INTERFACE: u16 = 1_011;

    /// A relationship references a workload that does not exist.
    pub const UNKNOWN_RELATIONSHIP_WORKLOAD: u16 = 1_012;

    /// A relationship references an interface that does not exist.
    pub const UNKNOWN_RELATIONSHIP_INTERFACE: u16 = 1_013;

    /// A relationship connects a workload to itself.
    pub const SELF_RELATIONSHIP: u16 = 1_014;

    /// The same relationship is declared more than once.
    pub const DUPLICATE_RELATIONSHIP: u16 = 1_015;

    /// A requested capability feature identifier is invalid.
    pub const INVALID_CAPABILITY_FEATURE: u16 = 1_016;

    /// The same capability feature is requested more than once.
    pub const DUPLICATE_CAPABILITY_FEATURE: u16 = 1_017;
}

impl ApplicationContract {
    /// Validates this Application Contract.
    ///
    /// # Errors
    ///
    /// Returns all detected contract diagnostics when the contract is invalid.
    pub fn validate(&self) -> Result<(), ContractDiagnostics> {
        let mut diagnostics = Vec::new();

        self.validate_contract_version(&mut diagnostics);
        self.validate_application(&mut diagnostics);
        self.validate_workloads(&mut diagnostics);
        self.validate_interfaces(&mut diagnostics);
        self.validate_relationships(&mut diagnostics);
        self.validate_capabilities(&mut diagnostics);
        self.validate_resources(&mut diagnostics);
        self.validate_network(&mut diagnostics);

        ContractDiagnostics::from_diagnostics(diagnostics)
    }

    fn validate_contract_version(&self, diagnostics: &mut Vec<Diagnostic>) {
        if self.contract_version() == CONTRACT_VERSION {
            return;
        }

        diagnostics.push(
            error(
                diagnostic_codes::UNSUPPORTED_VERSION,
                format!(
                    "unsupported Application Contract version `{}`",
                    self.contract_version()
                ),
            )
            .with_help(format!(
                "this Presolve build supports Application Contract version `{CONTRACT_VERSION}`"
            )),
        );
    }

    fn validate_application(&self, diagnostics: &mut Vec<Diagnostic>) {
        let name = self.application().name();

        if valid_application_name(name) {
            return;
        }

        diagnostics.push(
            error(
                diagnostic_codes::INVALID_APPLICATION_NAME,
                format!("invalid application name `{name}`"),
            )
            .with_help(
                "use 1-63 lowercase ASCII letters, digits, or hyphens; \
                 begin with a letter and end with a letter or digit",
            ),
        );
    }

    fn validate_workloads(&self, diagnostics: &mut Vec<Diagnostic>) {
        let mut names = HashSet::new();

        for workload in self.workloads() {
            let name = workload.name();

            if !valid_application_name(name) {
                diagnostics.push(
                    error(
                        diagnostic_codes::INVALID_WORKLOAD_NAME,
                        format!("invalid workload name `{name}`"),
                    )
                    .with_help(
                        "use 1-63 lowercase ASCII letters, digits, or hyphens; \
                         begin with a letter and end with a letter or digit",
                    ),
                );
            }

            if !names.insert(name) {
                diagnostics.push(
                    error(
                        diagnostic_codes::DUPLICATE_WORKLOAD,
                        format!("workload `{name}` is declared more than once"),
                    )
                    .with_help("declare each workload name once within an application"),
                );
            }
        }
    }

    fn validate_interfaces(&self, diagnostics: &mut Vec<Diagnostic>) {
        let mut names = HashSet::new();

        for interface in self.interfaces() {
            let name = interface.name();

            if !valid_kebab_segment(name) {
                diagnostics.push(
                    error(
                        diagnostic_codes::INVALID_INTERFACE_NAME,
                        format!("invalid interface name `{name}`"),
                    )
                    .with_help(
                        "use 1-63 lowercase ASCII letters, digits, or hyphens; \
                         begin with a letter and end with a letter or digit",
                    ),
                );
            }

            if !names.insert(name) {
                diagnostics.push(
                    error(
                        diagnostic_codes::DUPLICATE_INTERFACE,
                        format!("interface `{name}` is declared more than once"),
                    )
                    .with_help("declare each application-internal interface name once"),
                );
            }
        }
    }

    fn validate_relationships(&self, diagnostics: &mut Vec<Diagnostic>) {
        let workloads = self
            .workloads()
            .iter()
            .map(crate::WorkloadDefinition::name)
            .collect::<HashSet<_>>();

        let interfaces = self
            .interfaces()
            .iter()
            .map(crate::InterfaceDefinition::name)
            .collect::<HashSet<_>>();

        let mut relationships = HashSet::new();

        for relationship in self.relationships() {
            let from = relationship.from();
            let to = relationship.to();
            let interface = relationship.interface();

            if !workloads.contains(from) {
                diagnostics.push(
                    error(
                        diagnostic_codes::UNKNOWN_RELATIONSHIP_WORKLOAD,
                        format!("relationship references unknown source workload `{from}`"),
                    )
                    .with_help("declare the source workload in `[[workloads]]`"),
                );
            }

            if !workloads.contains(to) {
                diagnostics.push(
                    error(
                        diagnostic_codes::UNKNOWN_RELATIONSHIP_WORKLOAD,
                        format!("relationship references unknown target workload `{to}`"),
                    )
                    .with_help("declare the target workload in `[[workloads]]`"),
                );
            }

            if !interfaces.contains(interface) {
                diagnostics.push(
                    error(
                        diagnostic_codes::UNKNOWN_RELATIONSHIP_INTERFACE,
                        format!("relationship references unknown interface `{interface}`"),
                    )
                    .with_help("declare the interface in `[[interfaces]]`"),
                );
            }

            if from == to {
                diagnostics.push(
                    error(
                        diagnostic_codes::SELF_RELATIONSHIP,
                        format!("relationship connects workload `{from}` to itself"),
                    )
                    .with_help(
                        "application topology relationships must connect distinct workloads",
                    ),
                );
            }

            if !relationships.insert((from, to, interface)) {
                diagnostics.push(
                    error(
                        diagnostic_codes::DUPLICATE_RELATIONSHIP,
                        format!(
                            "relationship `{from}` -> `{to}` using `{interface}` \
                             is declared more than once"
                        ),
                    )
                    .with_help("declare each workload relationship once"),
                );
            }
        }
    }

    fn validate_capabilities(&self, diagnostics: &mut Vec<Diagnostic>) {
        let mut interfaces = HashSet::new();

        for capability in self.capabilities() {
            let interface = capability.interface();

            if !valid_capability_interface(interface) {
                diagnostics.push(
                    error(
                        diagnostic_codes::INVALID_CAPABILITY_INTERFACE,
                        format!("invalid capability interface `{interface}`"),
                    )
                    .with_help(
                        "use `<namespace>:<package>/<interface>` with lowercase \
                         kebab-case segments",
                    ),
                );
            }

            if !interfaces.insert(interface) {
                diagnostics.push(
                    error(
                        diagnostic_codes::DUPLICATE_CAPABILITY,
                        format!("capability interface `{interface}` is declared more than once"),
                    )
                    .with_help(
                        "declare each capability interface once and express compatible \
                         versions with a single version requirement",
                    ),
                );
            }
            let mut features = HashSet::new();

            for feature in capability.required_features() {
                if !valid_kebab_segment(feature) {
                    diagnostics.push(
                        error(
                            diagnostic_codes::INVALID_CAPABILITY_FEATURE,
                            format!(
                                "invalid required feature `{feature}` for capability `{interface}`"
                            ),
                        )
                        .with_help("use lowercase kebab-case semantic feature identifiers"),
                    );
                }

                if !features.insert(feature) {
                    diagnostics.push(
                        error(
                            diagnostic_codes::DUPLICATE_CAPABILITY_FEATURE,
                            format!(
                                "capability feature `{feature}` is requested more than once for `{interface}`"
                            ),
                        )
                        .with_help(
                            "declare each semantic feature once across required and preferred features",
                        ),
                    );
                }
            }

            for feature in capability.preferred_features() {
                if !valid_kebab_segment(feature) {
                    diagnostics.push(
                        error(
                            diagnostic_codes::INVALID_CAPABILITY_FEATURE,
                            format!(
                                "invalid preferred feature `{feature}` for capability `{interface}`"
                            ),
                        )
                        .with_help("use lowercase kebab-case semantic feature identifiers"),
                    );
                }

                if !features.insert(feature) {
                    diagnostics.push(
                        error(
                            diagnostic_codes::DUPLICATE_CAPABILITY_FEATURE,
                            format!(
                                "capability feature `{feature}` is requested more than once for `{interface}`"
                            ),
                        )
                        .with_help(
                            "declare each semantic feature once across required and preferred features",
                        ),
                    );
                }
            }
        }
    }

    fn validate_resources(&self, diagnostics: &mut Vec<Diagnostic>) {
        if self.resources().memory_mib() == Some(0) {
            diagnostics.push(error(
                diagnostic_codes::INVALID_RESOURCE_REQUIREMENT,
                "resource requirement `memory_mib` must be greater than zero",
            ));
        }

        if self.resources().cpu_millis() == Some(0) {
            diagnostics.push(error(
                diagnostic_codes::INVALID_RESOURCE_REQUIREMENT,
                "resource requirement `cpu_millis` must be greater than zero",
            ));
        }
    }

    fn validate_network(&self, diagnostics: &mut Vec<Diagnostic>) {
        let network = self.network();

        match network.outbound() {
            OutboundNetworkMode::Deny | OutboundNetworkMode::AllowAll => {
                if !network.allow().is_empty() {
                    diagnostics.push(error(
                        diagnostic_codes::INVALID_NETWORK_POLICY,
                        "`network.allow` may only be used when outbound mode is `allow_list`",
                    ));
                }
            }
            OutboundNetworkMode::AllowList => {
                if network.allow().is_empty() {
                    diagnostics.push(error(
                        diagnostic_codes::INVALID_NETWORK_POLICY,
                        "outbound mode `allow_list` requires at least one allowed target",
                    ));
                }
            }
        }

        for target in network.allow() {
            if !valid_network_target(target) {
                diagnostics.push(
                    error(
                        diagnostic_codes::INVALID_NETWORK_POLICY,
                        format!("invalid outbound network target `{target}`"),
                    )
                    .with_help("use a host or host-and-port target, not a URL or path"),
                );
            }
        }
    }
}

fn error(number: u16, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(diagnostic_code(number), DiagnosticSeverity::Error, message)
}

fn diagnostic_code(number: u16) -> DiagnosticCode {
    match DiagnosticCode::try_from_number(number) {
        Ok(code) => code,
        Err(error) => {
            panic!("invalid static Presolve diagnostic code {number}: {error}")
        }
    }
}

fn valid_application_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 63 {
        return false;
    }

    let bytes = name.as_bytes();

    let Some(first) = bytes.first() else {
        return false;
    };

    let Some(last) = bytes.last() else {
        return false;
    };

    first.is_ascii_lowercase()
        && last.is_ascii_alphanumeric()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

fn valid_capability_interface(interface: &str) -> bool {
    let Some((namespace, remainder)) = interface.split_once(':') else {
        return false;
    };

    if remainder.contains(':') {
        return false;
    }

    let Some((package, name)) = remainder.split_once('/') else {
        return false;
    };

    if name.contains('/') {
        return false;
    }

    valid_kebab_segment(namespace) && valid_kebab_segment(package) && valid_kebab_segment(name)
}

fn valid_kebab_segment(segment: &str) -> bool {
    if segment.is_empty() || segment.len() > 63 {
        return false;
    }

    let bytes = segment.as_bytes();

    let Some(first) = bytes.first() else {
        return false;
    };

    let Some(last) = bytes.last() else {
        return false;
    };

    first.is_ascii_lowercase()
        && last.is_ascii_alphanumeric()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

fn valid_network_target(target: &str) -> bool {
    !target.is_empty()
        && !target.chars().any(char::is_whitespace)
        && !target.contains("://")
        && !target.contains('/')
}
