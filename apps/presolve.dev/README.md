# presolve.dev

This is Presolve's dogfooded public website. It introduces the framework,
links its documentation, presents real application examples, and is built and
deployed through the same CLI and Cloudflare adapter available to users.

```sh
pnpm check
pnpm build
pnpm deploy:prepare
```

The first Cloudflare adapter deploys a static documentation surface. Server
capabilities are deliberately outside this site's deployment contract.
