use std::{error::Error, fmt, str::FromStr};

use edgezero_core::{Diagnostic, DiagnosticCode, DiagnosticSeverity};

use crate::{ApplicationContract, diagnostic_codes};

/// Canonical filename for an `EdgeZero` Application Contract.
pub const CONTRACT_FILE_NAME: &str = "edgezero.toml";

/// Collection of diagnostics that prevented a contract from being accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractDiagnostics {
    diagnostics: Vec<Diagnostic>,
}

impl ContractDiagnostics {
    pub(crate) fn single(diagnostic: Diagnostic) -> Self {
        Self {
            diagnostics: vec![diagnostic],
        }
    }

    pub(crate) fn from_diagnostics(diagnostics: Vec<Diagnostic>) -> Result<(), Self> {
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(Self { diagnostics })
        }
    }

    /// Returns all diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Consumes this value and returns the diagnostics.
    #[must_use]
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}

impl fmt::Display for ContractDiagnostics {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.diagnostics.as_slice() {
            [] => write!(formatter, "Application Contract validation failed"),
            [diagnostic] => diagnostic.fmt(formatter),
            diagnostics => write!(
                formatter,
                "{} Application Contract diagnostics; first: {}",
                diagnostics.len(),
                diagnostics[0]
            ),
        }
    }
}

impl Error for ContractDiagnostics {}

/// Parses and validates an `EdgeZero` Application Contract.
///
/// # Errors
///
/// Returns structured diagnostics if the TOML cannot be parsed or if the
/// resulting Application Contract fails semantic validation.
pub fn parse_contract(source: &str) -> Result<ApplicationContract, ContractDiagnostics> {
    let contract: ApplicationContract = match toml::from_str(source) {
        Ok(contract) => contract,
        Err(error) => {
            return Err(ContractDiagnostics::single(parse_diagnostic(&error)));
        }
    };

    contract.validate()?;

    Ok(contract)
}

impl FromStr for ApplicationContract {
    type Err = ContractDiagnostics;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        parse_contract(source)
    }
}

fn parse_diagnostic(error: &toml::de::Error) -> Diagnostic {
    Diagnostic::new(
        diagnostic_code(diagnostic_codes::PARSE_ERROR),
        DiagnosticSeverity::Error,
        format!("could not parse `{CONTRACT_FILE_NAME}`"),
    )
    .with_help(error.to_string())
}

fn diagnostic_code(number: u16) -> DiagnosticCode {
    match DiagnosticCode::try_from_number(number) {
        Ok(code) => code,
        Err(error) => {
            panic!("invalid static EdgeZero diagnostic code {number}: {error}")
        }
    }
}
