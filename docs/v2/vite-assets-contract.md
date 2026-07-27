# Vite styles and assets contract

`buildPresolveProduction` accepts explicit `viteEntries` alongside its one
compiler-selected virtual entry. The compiler entry remains the only entry
that receives a Presolve component identity. Every additional entry is a
caller-declared physical Vite path, resolved under Vite's configured root and
reported without a compiler identity.

This boundary lets Vite own CSS, CSS Modules, PostCSS, Tailwind, imported
fonts/media, public directories, physical output names, hashes, and source
maps. `@presolve/vite` forwards the caller's Vite `css`, `publicDir`, and
`build` options unchanged; it neither interprets styles nor creates an asset
manifest with semantic ownership. Duplicate entry names and attempts to use
the reserved compiler-entry name are rejected before Vite starts.

The production smoke fixture proves a CSS Module, an imported SVG URL, and a
public `robots.txt` are all emitted by Vite. The CSS/media entry has no
compiler artifact or component identity, while the virtual compiler entry
retains its manifest-bound identity and source-map translation.
