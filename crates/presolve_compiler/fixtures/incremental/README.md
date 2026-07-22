# L5 focused fixture matrix

The executable L5 service tests use complete request-owned multi-source inputs
and cover content-edit fan-out, no-change, add/delete representation,
configuration fallback, malformed baselines, failed-candidate isolation,
non-persistence, service-session isolation, and 20 fresh-session deterministic
repetitions. These golden documents intentionally contain plans/reports only;
compiler artifacts continue to use their frozen Phase A-K golden contracts.
