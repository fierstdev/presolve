#!/usr/bin/env python3

import json
import os
import urllib.request

EXPECTED_PROTOCOL = "presolve-http-json/0.1"
PROOF_KEY = "presolve.chunk16.checkpoint"
PROOF_VALUE = "container-capability-ok"

endpoint = os.environ["PRESOLVE_CAPABILITY_ENDPOINT"]
token = os.environ["PRESOLVE_CAPABILITY_TOKEN"]
protocol = os.environ["PRESOLVE_CAPABILITY_PROTOCOL"]

if protocol != EXPECTED_PROTOCOL:
    raise SystemExit(
        f"unexpected Presolve capability protocol: {protocol!r}"
    )

body = json.dumps(
    {
        "interface": "presolve:kv/store",
        "operation": "set",
        "input": {
            "key": PROOF_KEY,
            "value": PROOF_VALUE,
        },
    }
).encode("utf-8")

request = urllib.request.Request(
    endpoint,
    data=body,
    method="POST",
    headers={
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json",
    },
)

with urllib.request.urlopen(request, timeout=10) as response:
    result = json.loads(response.read().decode("utf-8"))

if result.get("status") != "ok":
    raise SystemExit(f"capability bridge rejected KV write: {result!r}")

print(f"wrote {PROOF_KEY}={PROOF_VALUE} through Presolve")
