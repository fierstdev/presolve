# Node capability executor contract

## Purpose

This contract turns Presolve's compiler-issued server handoffs into an
executable Node release without introducing a second router, evaluating route
source, or importing an unclassified package export. It closes the previous
`node`-classification-only boundary in two explicit gates:

1. a canonical Form submission bound to one integrity-qualified server-action
   capability; and
2. a route loader whose result codecs and Resource bootstrap target are fully
   named by the compiler.

Both gates are implemented. A raw HTTP endpoint with no authored application
binding is not completion evidence: every executable record must originate in
the canonical component, Form or Resource, route, package, and runtime-module
products described below.

Status: the canonical Form/server-action gate is implemented in authority
schema v14, server-action plan schema v2, and Forms artifact schema v7. The
canonical route-loader gate is implemented in route-loader plan schema v2 and
Resource artifact schema v4. Node deployment schema v3 carries both digest-bound
registries. Focused compiler, deterministic preparation, real-request,
cancellation, cache-policy, and real-browser proofs are green for both gates.

## Canonical Form-bound server action

A route component selects a server action through the existing canonical Form
surface:

```tsx
import { Component, defineForm, field, required } from "presolve";
import { saveContact } from "contact-service";

export class Contact extends Component {
  contact = defineForm({
    serialization: "form-data",
    fields: {
      name: field({ initial: "", validate: [required()] }),
      email: field({ initial: "", validate: [required()] }),
    },
    submit: async ({ formData, signal }) => saveContact(formData, signal),
  });

  render() {
    return (
      <main>
        <form form={this.contact}>
          <input name="name" bind:value={this.contact.fields.name} />
          <input name="email" type="email" bind:value={this.contact.fields.email} />
          <button type="submit">Send</button>
        </form>
      </main>
    );
  }
}
```

The TypeScript authority must prove the canonical `defineForm` declaration,
the imported `saveContact` symbol, the canonical DOM `FormData` and
`AbortSignal` parameters, and the Promise result. Parser spelling alone grants
no server meaning. The direct call is the complete callback body; captures,
member/default/namespace imports, reordered arguments, additional statements,
and ambient calls fail closed.

The selected semantic-package export is `server_action` with the exact type
signature `(FormData, AbortSignal) -> Promise<ServerActionResult>`,
`cold_fallback` resume, FormData input, one declared `json` or `redirect`
response family, and typed failure. The former decorated empty method remains
readable only as a compatibility handoff and is not evidence for this canonical
gate.

## Compiler execution product

The server-action plan advances to a new schema rather than changing schema-v1
meaning in place. Each executable record is joined from the canonical route,
Form submission plan, Form host, semantic-package binding, and package runtime
module table. It includes:

- route path and component, Form, Form instance, submission-plan, and host IDs;
- capability ID, package/version/integrity, export, and runtime module;
- the exact request coordinate selected by the compiler;
- serialization, input, response, failure, cancellation, and resume facts; and
- the browser artifact bridge that owns submission of that coordinate.

The request coordinate is
`/_presolve/actions/<lowercase SHA-256 of the UTF-8 capability-record-id>`. The compiler
publishes the complete coordinate in both the route server-action plan and the
Form runtime artifact. Browser and Node products compare the exact string;
neither reconstructs it from source names.

Multiple Forms and multiple actions per route are valid because every Form
host names one exact request coordinate. Duplicate IDs, coordinates, hosts, or
cross-route ownership fail publication.

## Executable registry

`presolve deploy node --prepare` consumes the route manifest and the exact
server-action plan from `dist/`. It uses the project-local Vite installation to
bundle only the compiler-listed runtime module and named export into
`.presolve/node/presolve.server-actions.mjs`. The generated module exports a
frozen registry keyed by capability-record ID.

The Node deployment plan inventories the registry path and SHA-256 digest.
Preparation fails when a runtime path escapes its package, a named export is
missing, Vite emits an unexpected external import, two records disagree about
one ID, or the bundle is absent from the release inventory. The generated host
verifies the registry digest before importing it. Package source remains opaque
to the compiler; Vite performs physical bundling only after the compiler has
selected the exact coordinate.

## Request lifecycle

The generated host uses only compiler-issued route patterns and action
coordinates.

- Only `POST` is admitted for an action coordinate. Other methods return `405`
  with `Allow: POST`.
- Requests with an `Origin` header must match the effective request origin.
  Cross-origin submissions return `403` before reading the body.
- The default body limit is 8 MiB. An oversized or malformed body returns
  `413` or `400` without invoking the capability.
- `multipart/form-data` and `application/x-www-form-urlencoded` are decoded
  through the platform `Request.formData()` implementation. Other media types
  return `415`.
- One `AbortController` belongs to one accepted request. Disconnect or host
  shutdown aborts it. Settlement after abort is ignored.
- The registry function receives exactly `(formData, signal)`. Application
  component source is never executed on the server.
- Action responses are `Cache-Control: no-store`; `HEAD`, speculative GET, and
  method tunnelling never invoke an action.

For a `json` capability, success is a JSON-compatible value and the host emits
`200 application/json`. For a `redirect` capability, success is an object with
one same-origin absolute-path `location`; the host emits `303` and `Location`.
The host rejects non-finite numbers, cycles, unsupported platform values,
foreign redirects, and response-family mismatches as executor failures.

A typed failure is a plain record with non-empty `code` and `message`, an
integer status from 400 through 599, and an optional JSON-compatible `issues`
value. It is emitted as a stable JSON error envelope. Unknown exceptions are
not reflected to the client: the host records the failure and returns the
stable `500 PSNODE2009_ACTION_EXECUTION_FAILED` envelope.

## Browser behavior

The existing compiler-owned Form host still owns validation, serialization,
duplicate suppression, reset cancellation, and submission state. For a server
capability it sends the canonical serialized FormData to the compiler-issued
coordinate with the submission-owned signal. It never imports the server
module into the browser.

Successful JSON settles the Form as completed and exposes the admitted result
to the submission result record. A redirect response follows normal navigation.
Typed failure settles the Form as failed with its normalized issues. Network
failure and abort retain the existing failed/cancelled lifecycle. Pending work
is not serialized or replayed during resume.

Cloudflare Static Assets continues to reject every server-action handoff. A
future Cloudflare dynamic adapter may consume this same execution plan, but it
must provide its own capability registry and request host proof.

## Route-loader gate

The canonical source form is an authority-proven component field:

```tsx
import { Component, loader, type Resource, type RouteParameters } from "presolve";
import { loadPost } from "post-service";

export class Post extends Component {
  post: Resource<PostRecord, NotFound> = loader(
    async (params: RouteParameters, signal: AbortSignal) =>
      loadPost(params, signal),
  );
}
```

The handler is one direct named-import call. TypeScript proves canonical
`loader`, the imported symbol, canonical `RouteParameters` and DOM
`AbortSignal`, and Promise completion. The package export is a server/shared
`resource` with a `route_loader` contract and exact signature
`(RouteParameters, AbortSignal) -> Promise<RouteLoaderResult>`.

Schema-v1 route-loader records did not contain a data codec, error codec,
Resource declaration/activation target, or initial bootstrap coordinate.
Schema v2 joins the authority-proven field, route instance, semantic
`Resource<Data, Error>` type, and package binding. Each record publishes the
closed data/error codecs, declaration/activation and state/data/error slot IDs,
ordered route-parameter names and segment indexes, strict UTF-8 percent-decoding
policy, and a cache-key recipe over the normalized parameter record.

Resource artifact schema v4 adds compiler-owned server-bootstrap descriptors,
not values. Per request, the Node host executes the exact loader registry,
codec-validates either data or typed error, then injects one script-safe
bootstrap value for the exact activation into the route document. The browser
Resource runtime consumes that value before Computed reads and never imports a
server module. A missing, duplicate, stale, or codec-invalid bootstrap fails
closed; returning raw loader JSON or inventing an unrelated page-data object is
forbidden.

`no_store` performs no cache lookup and omits `max_age_seconds`. `private` and
`public` require a positive `max_age_seconds`; both caches live only within the
current host process. A private key includes the authorization/cookie partition
digest, while a public key excludes all private request material. Every key
also includes the loader capability ID and canonical JSON of the ordered,
normalized parameter record. Pending work is coalesced only for the same
complete key, and every waiter retains independent disconnect cancellation.

The focused executor proof covers strict percent-decoding and invalid segment
rejection, exact data/error codec validation, no-store/private/public cache
behavior and partitioning, request and host-shutdown abort, deterministic
preparation, browser bootstrap restoration and reactive rendering, multiple
loader routes, and missing or unbundleable runtime modules. Route selection and
Resource artifacts are isolated per published route, so sibling server
declarations cannot leak into another page.

## Completion evidence

The Node capability executor requires all of the following before publication:

1. TypeScript alias/lookalike/signature proof and parser shape coverage;
2. deterministic compiler plan and browser artifact fixtures;
3. Node preparation failures for missing, mismatched, or unbundleable exports;
4. real HTTP multipart and URL-encoded submissions, JSON success, redirect,
   typed failure, body limits, origin rejection, cancellation, and mixed static
   routes;
5. real-browser validation, duplicate suppression, success, typed failure, and
   reset cancellation; exact HTTP redirect; and the shared Forms resume proof
   that active submissions are never replayed; and
6. route-loader authority, exact parameter and codec planning, cache-scope,
   typed-failure, cancellation, deterministic registry, and browser Resource
   rendering proofs; and
7. full application-platform, documentation, deterministic release, and public
   package gates.
