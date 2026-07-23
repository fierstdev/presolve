# Phase O O4 multi-source application publication adoption

**Status:** complete through Phase P P4.

Phase O consumes Phase P's new canonical multi-source product only through the
`@presolve/application` argument projection. The package projects explicit
caller inputs to `presolve application build`; it neither owns source loading
nor compiler/product/artifact interpretation. P3's compiler CLI remains the
sole multi-source publication and atomic-output authority.
