# @presolve/cli

This package installs the `presolve` compiler command. It selects the matching
platform binary from a package published with the same Presolve release.

It supports macOS on Apple Silicon and Intel, Linux x64, and Windows x64 in the
0.1 alpha train. The launcher never downloads or builds a compiler during an
application install; package bytes are the release artifact.
