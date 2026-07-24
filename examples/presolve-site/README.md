# Presolve public site dogfood application

This is the production-shaped public Presolve site: an introduction, public
documentation, source examples, and framework capability comparisons authored
as ordinary compiler-routed Presolve components.

```sh
presolve check
presolve build
presolve deploy cloudflare --prepare --name presolve-site
```

The first Cloudflare adapter deliberately deploys the static documentation
surface. Server-capability pages are added only with an explicit compiler and
provider executor contract.
