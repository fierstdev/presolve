# Phase P P3 atomic application CLI

**Status:** P3 implementation authority.

## Public command

```sh
presolve application build --config presolve.json \
  --source src/App.tsx=src/App.tsx --source src/Card.tsx=src/Card.tsx \
  --entry src/App.tsx --out dist
```

The command requires exactly one `--config`, `--entry`, and `--out`; at least
one explicit contained `--source`; and accepts repeatable existing
`--package-contract`/`--package-runtime` mappings plus `--production`. It
loads no unlisted source and does not derive an entry from filenames, exports,
or working-directory state. Compiler `PSAPP` diagnostics pass through without
translation.

## Atomic output representation

P3 amends P0's output-root wording to make replacement genuinely atomic on
ordinary filesystems. `--out` is a Presolve-owned symbolic publication pointer,
not an author-created directory. Each successful build writes and validates a
new immutable sibling release directory, then atomically renames a new pointer
over the previous pointer. Consumers continue to use `dist/index.html` and
other normal `dist/<artifact>` paths through that pointer.

An existing non-symbolic `--out` directory is rejected with
`PSAPP3004_OUTPUT_ROOT_NOT_PUBLICATION_POINTER`; P3 never deletes or mutates
it. A lowering, write, digest, manifest, or pointer-creation failure removes
only its new staged release and leaves the previous publication pointer
unchanged. The previous immutable release is deliberately retained, so a
reader which already resolved it cannot observe a partial replacement.

Unix and Windows directory symbolic links are supported through their native
platform operations. Other hosts fail closed rather than falling back to a
non-atomic directory copy or JavaScript artifact shim.

## Verification boundary

Before commit the CLI verifies the exact compiler-generated manifest bytes,
the manifest inventory cardinality, normalized relative paths, and SHA-256
digest of every staged artifact. It cannot add an artifact, decode one, or
rewrite a manifest. The compiler still owns artifact semantics; the CLI owns
only filesystem containment and atomic pointer publication.
