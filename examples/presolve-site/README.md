# Presolve documentation-site example

This is a local dogfood example: an introduction, documentation, source
examples, and framework capability comparisons authored as ordinary
compiler-routed Presolve components. It is not the source of the separately
operated public Presolve website.

```sh
presolve check
presolve build
presolve deploy cloudflare --prepare --name presolve-site
```

The first Cloudflare adapter deliberately deploys the static documentation
surface. Server-capability pages are added only with an explicit compiler and
provider executor contract.
