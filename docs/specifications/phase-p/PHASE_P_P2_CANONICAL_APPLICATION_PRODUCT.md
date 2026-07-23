# Phase P P2 canonical application publication product

**Status:** P2 implementation authority.

## Compiler-owned lowering

`build_application_publication_product_v1` accepts only a previously validated
`ApplicationPublicationRequestV1`. It parses the exact caller-authorized
source set, preserves the selected entry semantic identity, and reuses the
existing canonical application semantic model, IR, runtime, Resource,
opaque-terminal, resume, production, report, and page products. It does not
introduce a framework renderer, a source parser, or an artifact merger.

The request now owns source text as explicit `ApplicationPublicationSourceV1`
records. The compiler derives the `CompilationUnit` itself, which makes the
workspace snapshot a digest of the canonical workspace configuration and the
complete exact source set rather than a host-supplied identity.

## Product and manifest

The product returns an ordered relative-path-to-bytes inventory plus an
`ApplicationPublicationManifestV1`.

The schema-v1 manifest contains:

* `compiler_contract: "presolve-application-publication:1"`;
* a deterministic `application-workspace:sha256:...` snapshot ID;
* the validated entry semantic ID;
* `development` or `production` profile; and
* the sorted digest/path inventory of every generated artifact.

`application.manifest.json` is generated from that manifest but intentionally
does not list itself: its bytes are the manifest representation, while every
other published byte is integrity-bound by the inventory. The caller must not
rewrite, supplement, or merge this inventory.

Production publication includes the canonical `production/` module layout;
development publication does not. Both profiles retain the same compiler
generated runtime/resume artifact family and never select an entry implicitly.

The selected entry is also the sole ordinary HTML materialization root. Other
top-level components in the complete workspace remain compiler semantic inputs
and may appear in compiler metadata needed for whole-workspace validation, but
they must not render into the selected entry page. This is a Phase R
production-correctness amendment to the frozen v1 interpretation; it changes
neither the request nor manifest schema.

## Host boundary

This product has no filesystem side effect. P3 alone may stage exactly these
bytes below a caller-owned output root, validate the inventory, and commit the
directory. That separation keeps lowering and artifact identity in the
compiler while leaving durable output replacement to the CLI host.

## Compatibility

P2 is a new v1 product. It does not alter `presolve build <source>`, its
artifact names, or its output behavior. A caller with an incompatible request
or package runtime mapping receives a typed `PSAPP` failure before it is given
an artifact inventory.
