# Serialization and codec protocol contract

`presolve_compiler::codec_protocol` is schema v1 of the V2 codec declaration
ledger. It preserves the existing Form serialization, resume codec, and platform
durable representations as their own authorities. This product does not encode
or decode values and does not change their bytes.

## Independent classifications

Every declaration receives six separate classifications: runtime validated,
form serializable, network serializable, HTML publishable, resume serializable,
and structured cloneable. They are deliberately not one boolean. For example, a
structurally serializable tuple is network serializable but is rejected by the
closed existing resume codec and is not HTML publishable.

The classifications use canonical `SemanticType` values. Unknown, nominal
compiler-only, and unsupported values are rejected at the declaration boundary;
the protocol records an `unsupported_source_type` diagnostic rather than
allowing a later encoder to guess.

## Codec declaration

Every codec declares its source type, serialized representation, participating
environments, encode behavior, decode behavior, positive version, and failure
behavior. Duplicate IDs, no environments, and version zero are diagnostics.
Inspection records retain these exact fields in deterministic ID order.

Approved platform or class codecs can be added by a later canonical lowering.
They must supply this declaration; no source spelling or package name creates a
codec contract implicitly.
