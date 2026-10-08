use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use thiserror::Error;
use uuid::Uuid;

/// Error returned when an `Presolve` resource identifier cannot be parsed.
#[derive(Debug, Error)]
pub enum IdParseError {
    /// The identifier has the wrong resource prefix.
    #[error("invalid resource identifier prefix: expected `{expected}`, found `{actual}`")]
    InvalidPrefix {
        expected: &'static str,
        actual: String,
    },

    /// The UUID portion of the identifier is invalid.
    #[error("invalid UUID in resource identifier: {0}")]
    InvalidUuid(#[from] uuid::Error),
}

macro_rules! resource_id {
    (
        $(#[$meta:meta])*
        $name:ident,
        $prefix:literal
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(Uuid);

        impl $name {
            /// Creates a new randomly generated identifier.
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Creates an identifier from an existing UUID.
            #[must_use]
            pub const fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            /// Returns the underlying UUID.
            #[must_use]
            pub const fn as_uuid(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "{}{}", $prefix, self.0)
            }
        }

        impl FromStr for $name {
            type Err = IdParseError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                let Some(uuid) = value.strip_prefix($prefix) else {
                    let actual = value
                        .split_once('_')
                        .map_or_else(|| value.to_owned(), |(prefix, _)| format!("{prefix}_"));

                    return Err(IdParseError::InvalidPrefix {
                        expected: $prefix,
                        actual,
                    });
                };

                Ok(Self(Uuid::parse_str(uuid)?))
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.collect_str(self)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                value.parse().map_err(D::Error::custom)
            }
        }
    };
}

resource_id!(
    /// Stable identifier for an `Presolve` application.
    ApplicationId,
    "app_"
);

resource_id!(
    /// Stable identifier for an `Presolve` execution environment.
    EnvironmentId,
    "env_"
);

resource_id!(
    /// Stable identifier for an `Presolve` capability provider.
    ProviderId,
    "prv_"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn application_id_round_trips_through_text() {
        let id = ApplicationId::new();
        let encoded = id.to_string();
        let decoded: ApplicationId = encoded.parse().expect("identifier should parse");

        assert_eq!(decoded, id);
        assert!(encoded.starts_with("app_"));
    }

    #[test]
    fn environment_id_rejects_application_prefix() {
        let application_id = ApplicationId::new().to_string();
        let result = application_id.parse::<EnvironmentId>();

        assert!(matches!(result, Err(IdParseError::InvalidPrefix { .. })));
    }

    #[test]
    fn provider_id_round_trips_through_json() {
        let id = ProviderId::new();

        let json = serde_json::to_string(&id).expect("identifier should serialize");
        let decoded: ProviderId =
            serde_json::from_str(&json).expect("identifier should deserialize");

        assert_eq!(decoded, id);
    }

    #[test]
    fn uuid_can_be_recovered() {
        let uuid = Uuid::new_v4();
        let id = ApplicationId::from_uuid(uuid);

        assert_eq!(id.as_uuid(), uuid);
    }
}
