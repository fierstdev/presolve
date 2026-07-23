# Phase N N5-A component invocation contract

N5-A admits the compiler's existing resolved component-invocation path. A
supported JSX component use produces a canonical invocation identity, caller
and callee ownership facts, a component-instance plan, composition analysis,
Slot bindings, Context-instance bindings, and existing component/resume
artifacts.

Dynamic constructors, reflection, arbitrary spread inputs, runtime component
lookup, and framework-owned rendering remain unsupported. Inputs and callback
contracts continue to require their own compiler type/lowering admissions.
