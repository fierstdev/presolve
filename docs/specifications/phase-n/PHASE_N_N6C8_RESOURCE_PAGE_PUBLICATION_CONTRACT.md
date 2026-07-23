# Phase N N6-C8 resource page publication contract

`generate_standalone_page_with_resume_runtime_and_resources` is the dedicated
compiler page-publication path for a host-bound Resource artifact. It serializes
the artifact as escaped JSON in `presolve-resources-runtime` immediately before
`runtime.js`, preserving the existing artifact ordering and boot script.

The existing page generator remains unchanged for applications without a
Resource artifact. The CLI does not select this path yet, because Resource
source remains rejected and the runtime has not installed a Resource artifact
validator or endpoint executor.

Verification is `scripts/verify-n6c8-resource-page-publication.sh`.
