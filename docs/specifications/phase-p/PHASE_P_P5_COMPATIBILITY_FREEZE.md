# Phase P P5 compatibility freeze

**Status:** frozen.

Phase P v1 consists of the following compatible public surface:

| Surface | Versioned contract |
| --- | --- |
| Compiler request | `ApplicationPublicationRequestV1` with exact source text, configuration, entry, package tables, profile, and output root |
| Compiler product | `ApplicationPublicationProductV1` and `ApplicationPublicationManifestV1` schema 1 |
| Compiler identity | `presolve-application-publication:1` |
| CLI | `presolve application build --config --source... --entry --out` |
| Output | atomic `--out` publication pointer to an immutable compiler-generated release |
| Application projector | `createApplicationPublicationInvocation` / `application-build` envelope selector |

The manifest inventories all generated artifacts other than its own serialized
file and binds each listed path to SHA-256 bytes. Requests fail closed on an
invalid configuration/source set/entry, unsupported package mapping, generated
artifact mismatch, or a pre-existing non-pointer output root. The legacy
single-source `presolve build <source>` contract remains unchanged.

The v1 product does not provide source discovery, inferred entries, manifest
migration, JavaScript artifact merging, a server, routing, SSR, or a framework
runtime. Any future extension requires a separately versioned request,
manifest, and compatibility proof.
