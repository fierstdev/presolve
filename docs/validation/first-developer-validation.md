# First developer-validation outreach

This is not a broad Presolve launch plan.

The goal of the first outreach cycle is to expose the portability model to technically qualified outsiders and find the assumptions that fail before Presolve expands scope.

## Validation objective

The flagship proof makes one falsifiable claim:

> One immutable application release can declare provider-neutral requirements, resolve differently against multiple environments, and execute through materially different provider implementations without rebuilding or mutating the application.

External validation should answer four questions:

1. Do experienced infrastructure engineers understand the Application / Environment / Deployment separation quickly?
2. Do they believe the current proof actually supports the portability claim being made?
3. Where do they expect capability semantics or deployment reality to break the abstraction?
4. Is the problem important enough that they would try Presolve on a real application or contribute a provider/environment case?

## Who to ask first

Prioritize practitioners who can attack the architecture rather than general developers who can only react to the pitch.

### Cohort A — platform and infrastructure engineers

Ideal experience:

- Kubernetes/platform engineering;
- internal developer platforms;
- multi-cloud or cloud migration work;
- Terraform/Pulumi infrastructure ownership;
- developer-platform or deployment tooling.

Ask them to focus on whether environment supply can realistically stay outside application identity and whether the DeploymentPlan exposes enough information for operators.

### Cohort B — runtime and WebAssembly engineers

Ideal experience:

- Wasmtime / Component Model / WASI;
- Spin, wasmCloud, Fermyon, component linking, or similar systems;
- container runtimes and workload isolation.

Ask them to focus on whether Presolve is correctly treating WIT/Components as a binding/execution target rather than making the execution substrate the application architecture.

### Cohort C — backend and distributed-systems engineers

Ideal experience:

- Redis, SQL, queues, object storage, or cloud service APIs;
- data consistency and failure semantics;
- service migrations between managed providers.

Ask them to attack the capability-contract layer: what semantics must be explicit before provider substitution is honest rather than lowest-common-denominator abstraction?

## What to send

Do not begin with a long architecture document.

Send three things, in this order:

1. `https://presolve.dev/proof/`
2. the one-command demo: `bash ./demo/portability/run.sh`
3. the concise walkthrough: `docs/proofs/flagship-portability-walkthrough.md`

The recipient should be able to understand the claim before reading the rest of the repository.

## Direct outreach message

Use this for individual engineers you already know, people who have publicly discussed the exact portability problem, or maintainers who have explicitly invited related technical discussion.

> I am building Presolve, an open application-portability layer. The core model separates an immutable application release from the environment that supplies its runtime capabilities.
>
> I have reached the point where I need architectural criticism more than feature ideas. I put together a reproducible proof that builds one `.presolved` release once, resolves it against memory, Redis, and Cloudflare environments, and executes the exact same bytes through different providers without rebuilding the application.
>
> Proof: https://presolve.dev/proof/
>
> If you have 10–15 minutes, I would especially value an answer to one question: **where would this abstraction break in infrastructure you have actually operated?**
>
> “This is already solved by X” or “this becomes dishonest once Y matters” are both useful answers.

Do not ask for stars, reposts, upvotes, or generic support.

## Short public post

Use this in a community surface that permits project sharing and feedback requests.

> I am building Presolve, an open application-portability layer around a simple invariant: an application release should have immutable identity independent of the environment that realizes it.
>
> I now have a runnable proof rather than just an architecture diagram. One `.presolved` release is built once, resolves against memory, Redis, and Cloudflare provider environments, executes unchanged through memory and a real Redis process, and is re-hashed afterward to prove the application bytes never changed.
>
> The interesting question is not whether the demo passes. It is where this model stops being honest in real infrastructure.
>
> I would value criticism from platform/backend engineers, especially around capability semantics, provider substitution, and the Application/Environment boundary.
>
> Proof: https://presolve.dev/proof/
> Source: https://github.com/fierstdev/presolve

## Initial public channels

### 1. r/devops weekly self-promotion thread

This is the safest first broad technical surface because the subreddit explicitly provides a recurring thread for project promotion and feedback.

Current thread pattern:

`https://www.reddit.com/r/devops/`

Use the short public post above, but make the final question concrete:

> If you operate platform infrastructure, what requirement would force this release to become environment-specific?

Do not make a separate promotional top-level post unless the subreddit rules clearly permit it at the time of posting.

### 2. Hacker News — later, not first

Presolve is technically a good Show HN shape because it is open, runnable locally, requires no account, and has a concrete artifact people can inspect.

However, Hacker News currently states that Show HNs are temporarily restricted and asks prospective posters to first become genuine contributors to the community.

Source:

- `https://news.ycombinator.com/showlim`
- `https://news.ycombinator.com/showhn.html`

Do not create an account solely to post Presolve or automate a submission.

When appropriate, the title should stay literal:

> Show HN: Presolve – build one immutable application release, decide where it runs later

The linked destination should be the runnable project/proof, not a signup or fundraising page.

### 3. Lobsters — only through normal community participation

Lobsters is highly relevant technically, but its guidelines explicitly discourage using the community as a write-only promotional channel. Self-promotion should remain a minority of participation, and membership is invitation-based.

Source:

`https://lobste.rs/about`

If the user is already an active member or Presolve is independently submitted by someone else, participate transparently as the author. Do not seek an invitation solely to post the project.

A suitable technical title would be:

> Presolve: separating immutable application identity from deployment realization

## First batch

Do not optimize for reach yet.

A useful first batch is:

- 3 platform/infrastructure engineers;
- 2 backend/distributed-systems engineers;
- 2 WebAssembly/runtime engineers;
- 1 public r/devops self-promotion-thread post.

Eight serious technical conversations are more valuable at this stage than thousands of low-context impressions.

## Ask for falsification, not approval

Every conversation should include at least one of these questions:

- What real deployment requirement would force environment-specific data into the application release?
- Which capability semantic is missing before you would trust provider substitution?
- What would you need in a DeploymentPlan before allowing this into a production deployment pipeline?
- Which existing system makes Presolve unnecessary, and where is that overlap exact?
- What workload would you use to try to break this proof?
- Would you be willing to describe one real environment for us to model as a fixture/provider test?

Avoid questions such as “Do you like it?” or “Would you use this?” without a concrete scenario. They produce weak validation data.

## Record feedback consistently

For each conversation, record:

| Field | Meaning |
| --- | --- |
| Role | platform, runtime, backend, etc. |
| Context | infrastructure they have actually operated |
| Understood claim? | yes / partially / no |
| Strongest objection | their best argument against the model |
| Semantic gap | capability/provider semantic Presolve may be missing |
| Workflow gap | adoption or operator friction |
| Existing alternative | what they would use instead |
| Reproduction | whether they ran/read the proof |
| Follow-up | concrete test, issue, provider, or no action |

Do not reduce this to a lead-generation CRM. The purpose is architectural evidence.

## Signals that justify the next phase

Move from validation into a broader developer-preview launch only after several of these occur:

- an outsider reproduces the proof without direct help;
- an outsider identifies a semantic gap and the model survives the resulting correction;
- someone supplies a real deployment/environment case Presolve did not design around;
- someone asks to implement or test another provider;
- multiple engineers independently describe the same deployment pain in their own words;
- at least one engineer wants to try Presolve on a non-demo application.

## Signals to stop and reconsider

Pause feature expansion if feedback repeatedly shows that:

- the Application / Environment boundary is not understandable without extensive explanation;
- important provider semantics cannot be represented without putting vendor details into application code;
- the DeploymentPlan is too weak to be operationally trustworthy;
- the same problem is already solved more directly by an existing standard/tool without Presolve adding a distinct layer;
- the proof only works because `kv-demo` avoids the hard parts of real applications.

Those would be valuable findings, not failed marketing.
