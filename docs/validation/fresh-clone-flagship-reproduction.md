# Fresh-clone flagship reproduction

The flagship portability demonstration has been reproduced from a fresh public clone on a separate Linux development machine rather than only inside the repository's GitHub Actions environment.

The reproduction used the documented entry point:

```bash
git clone https://github.com/fierstdev/presolve.git
cd presolve
bash ./demo/portability/run.sh
```

No application artifact was supplied out of band.

## Observed release identity

The fresh clone independently built the same canonical release identity currently expected by the flagship proof:

```text
release digest: sha256:fa8cb59cf09dd5ed507d865fd4b739f6229e1422367bfe1f9afdfba6adc57c5e
archive sha256: 45e6f9dca20dce54337f34f403e772e510a1f6b6a333a44112d82c09af014344
```

## Observed planning

The exact detached release resolved successfully against all three flagship environments:

| Environment | Selected provider | Result |
| --- | --- | --- |
| `portability-local` | `memory-kv` | resolved |
| `portability-redis` | `redis-kv` | resolved |
| `portability-cloudflare` | `cloudflare-kv` | resolved |

## Observed execution

The default demonstration executed those exact release bytes through both automatic realizations:

- local in-process memory KV
- network Redis started by the demonstration

Cloudflare remained plan-only because `--cloudflare` was not supplied, as designed.

The Redis test image resolved to:

```text
redis:7.4-alpine
sha256:858f009f9709ce576febc734aa78b8f6d624b82571f9ddb6bda4377c833b3499
```

## Final identity check

After both realizations completed, the independently reproduced run reported:

```text
release digest unchanged: yes
archive sha256 unchanged: yes
```

and finished with:

```text
PASS — one .presolved release, three plans, two provider realizations, zero application rebuilds.
```

## Why this evidence matters

GitHub Actions proves that the repository continuously exercises the portability invariant in a controlled environment. A fresh-clone reproduction adds a different kind of evidence: the public repository contains enough source, configuration, build logic, and provider proof infrastructure for another machine to reproduce the same application identity and execute the documented demonstration without private build artifacts.

This is still an owner-operated validation rather than independent third-party adoption. It should therefore be treated as reproducibility evidence, not external market validation.