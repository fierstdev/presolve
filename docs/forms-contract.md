# Forms contract

Phase I Forms are compiler-owned, declaration-level semantics. This contract
is frozen at the I20 boundary; Phase J may add live restoration but must not
reconstruct Form semantics from browser state.

## Syntax and identity

`@form()` declares one component-owned `Form`; `@field(this.form)` declares a
Form-owned Field; `field={this.field}` binds one supported intrinsic control.
Validation, tracking, submission, serialization, reset, instances, slots, IR,
runtime registry, and artifacts all consume those canonical IDs. The only
submit host syntax is `<form form={this.form}>`. Its `form` attribute is a
compiler-only bridge and is never ordinary emitted HTML.

Forms and Fields are not inherited. Fields, validators, and controls never
derive ownership from DOM ancestry. A host is one explicit same-component Form
use site: nested hosts, multiple hosts for one Form, and controls for another
Form inside a host are invalid. Controls outside a host remain valid.

## Execution

The compiler plans validation, dirty/touched tracking, submission,
serialization (`json`, `form-data`, or `url-encoded`), reset, exact
component/Form instances, and optimized Form IR. Runtime accepts only emitted
instance-qualified control and submit-host anchors. It listens only to the
emitted submit event and calls `preventDefault()` only when the host artifact
requires it. Submission validates compiler-owned state, serializes compiler
records, and invokes the emitted action batch.

Runtime never scans forms or controls, searches nearest ancestors, uses
authored names as authority, reconstructs dependencies, creates semantic IDs,
or uses browser-native validation, `FormData(formElement)`, or DOM ancestry as
semantic authority.

## Inspection, diagnostics, and schemas

Semantic graph v6 exposes Form, Field, FieldBinding, and ValidationRule nodes
with typed Forms edges. ASM inspection v9 exposes the shared canonical Forms
projection. Check JSON v5 projects `PSC1084` through `PSC1095` only from
retained compiler facts. Template manifest v4 and component artifact v3 extend
the existing Forms bridge with reciprocal `TemplateInstanceTargetId` and
`ComponentInstanceId` records; `forms.runtime.json` remains v1 and retains
Phase I ownership. Form control/host dispatch uses that ordinary target marker
directly and does not create a Form-only ownership path. Resume manifest
remains v5.

## Unsupported through Phase I

Nested or inherited Forms, dynamic Fields/registration, uncontrolled or custom
controls, files, checkbox groups, custom/async/server validation, validation
messages/localization, submit parameters, async/network submission, automatic
success reset, authored reset controls, DOM-derived ownership, dynamic formats,
custom serialized names, multipart files, and live Form restoration are not
supported. Phase J has not begun.
