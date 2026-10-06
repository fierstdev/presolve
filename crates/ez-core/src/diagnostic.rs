use std::{fmt, num::ParseIntError, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use thiserror::Error;

/// Minimum numeric diagnostic code.
pub const MIN_DIAGNOSTIC_CODE: u16 = 1;

/// Maximum numeric diagnostic code.
pub const MAX_DIAGNOSTIC_CODE: u16 = 9_999;

/// Stable machine-readable `EdgeZero` diagnostic code.
///
/// Diagnostic codes are rendered as `EZ` followed by four decimal digits.
///
/// Examples:
///
/// - `EZ0001`
/// - `EZ1024`
/// - `EZ9999`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DiagnosticCode(u16);

impl DiagnosticCode {
    /// Creates a diagnostic code from its numeric portion.
    ///
    /// # Errors
    ///
    /// Returns [`DiagnosticCodeError::OutOfRange`] when `number` is outside
    /// the supported `0001..=9999` range.
    pub const fn try_from_number(number: u16) -> Result<Self, DiagnosticCodeError> {
        if number < MIN_DIAGNOSTIC_CODE || number > MAX_DIAGNOSTIC_CODE {
            return Err(DiagnosticCodeError::OutOfRange(number));
        }

        Ok(Self(number))
    }

    /// Returns the numeric portion of the diagnostic code.
    #[must_use]
    pub const fn number(self) -> u16 {
        self.0
    }
}

/// Error returned when a diagnostic code is invalid.
#[derive(Debug, Error)]
pub enum DiagnosticCodeError {
    /// The code is outside the supported numeric range.
    #[error("diagnostic code number must be between 1 and 9999, found {0}")]
    OutOfRange(u16),

    /// The textual representation has the wrong structure.
    #[error("diagnostic code must use `EZ` followed by four decimal digits")]
    InvalidFormat,

    /// The numeric portion could not be parsed.
    #[error("invalid diagnostic code number: {0}")]
    InvalidNumber(#[from] ParseIntError),
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let number = self.0;
        write!(formatter, "EZ{number:04}")
    }
}

impl FromStr for DiagnosticCode {
    type Err = DiagnosticCodeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() != 6 {
            return Err(DiagnosticCodeError::InvalidFormat);
        }

        let Some(number) = value.strip_prefix("EZ") else {
            return Err(DiagnosticCodeError::InvalidFormat);
        };

        if !number.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(DiagnosticCodeError::InvalidFormat);
        }

        Self::try_from_number(number.parse()?)
    }
}

impl Serialize for DiagnosticCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for DiagnosticCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(D::Error::custom)
    }
}

/// Severity of an `EdgeZero` diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    /// Informational diagnostic.
    Info,

    /// Condition that may require attention but does not prevent the operation.
    Warning,

    /// Condition that prevents the requested operation.
    Error,
}

/// Structured diagnostic emitted by `EdgeZero`.
///
/// Diagnostics are intended for both humans and automation. The diagnostic code
/// is stable and machine-readable while the message and help text may improve
/// over time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    code: DiagnosticCode,
    severity: DiagnosticSeverity,
    message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    help: Option<String>,
}

impl Diagnostic {
    /// Creates a diagnostic.
    #[must_use]
    pub fn new(
        code: DiagnosticCode,
        severity: DiagnosticSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity,
            message: message.into(),
            help: None,
        }
    }

    /// Adds remediation or explanatory help text.
    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Returns the stable diagnostic code.
    #[must_use]
    pub const fn code(&self) -> DiagnosticCode {
        self.code
    }

    /// Returns the diagnostic severity.
    #[must_use]
    pub const fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }

    /// Returns the human-readable diagnostic message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns optional diagnostic help text.
    #[must_use]
    pub fn help(&self) -> Option<&str> {
        self.help.as_deref()
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} {:?}: {}",
            self.code, self.severity, self.message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_code_is_zero_padded() {
        let code = DiagnosticCode::try_from_number(7).expect("code should be valid");

        assert_eq!(code.to_string(), "EZ0007");
    }

    #[test]
    fn diagnostic_code_round_trips() {
        let code: DiagnosticCode = "EZ1420".parse().expect("code should parse");

        assert_eq!(code.number(), 1_420);
        assert_eq!(code.to_string(), "EZ1420");
    }

    #[test]
    fn diagnostic_code_rejects_zero() {
        let result = "EZ0000".parse::<DiagnosticCode>();

        assert!(matches!(result, Err(DiagnosticCodeError::OutOfRange(0))));
    }

    #[test]
    fn diagnostic_serializes_structurally() {
        let code = DiagnosticCode::try_from_number(1).expect("code should be valid");

        let diagnostic = Diagnostic::new(
            code,
            DiagnosticSeverity::Error,
            "application contract is invalid",
        )
        .with_help("correct the contract and retry");

        let json = serde_json::to_value(diagnostic).expect("diagnostic should serialize");

        assert_eq!(json["code"], "EZ0001");
        assert_eq!(json["severity"], "error");
        assert_eq!(json["message"], "application contract is invalid");
        assert_eq!(json["help"], "correct the contract and retry");
    }

    #[test]
    fn diagnostic_without_help_omits_help_field() {
        let code = DiagnosticCode::try_from_number(2).expect("code should be valid");

        let diagnostic = Diagnostic::new(code, DiagnosticSeverity::Warning, "example warning");

        let json = serde_json::to_value(diagnostic).expect("diagnostic should serialize");

        assert!(json.get("help").is_none());
    }
}
