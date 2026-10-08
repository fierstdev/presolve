use std::collections::HashSet;

use presolve_core::{Diagnostic, DiagnosticCode, DiagnosticSeverity};

use crate::{ApplicationContract, CONTRACT_VERSION, ContractDiagnostics, OutboundNetworkMode};

/// Stable numeric codes emitted by Application Contract validation.
///
/// The `EZ1xxx` range is reserved for Application Contract diagnostics.
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
