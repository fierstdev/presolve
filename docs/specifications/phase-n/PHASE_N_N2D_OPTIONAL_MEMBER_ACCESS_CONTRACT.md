# Phase N N2-D optional member access contract

`value?.property` is admitted only in a supported Computed getter. The compiler
retains its optional flag through `MemberAccess`, `GetMember`, and the emitted
runtime artifact. The generated read is already null-safe and owns-property
only; no JavaScript source, prototype lookup, optional call, optional index,
or write is evaluated. N2-D advances the runtime-computed artifact to schema
`8` by making optionality explicit in `get-member` metadata.
