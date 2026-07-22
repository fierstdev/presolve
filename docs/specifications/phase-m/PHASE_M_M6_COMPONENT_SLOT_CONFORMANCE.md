# Phase M M6-A component and Slot conformance

**Status:** M6-A implementation authority.

## Scope

M6-A adds only the compiler's existing Component invocation and Slot field
forms. The declaration package supplies `SlotContent` and `@slot()` typing; it
does not create a children object, JSX runtime, outlet renderer, forwarding
protocol, or ownership lookup.

The conformance source is byte-identical to the compiler's valid component
declaration fixture. It retains the existing `<slot />` and named outlet syntax
unchanged.

## Evidence

The verifier proves TypeScript 7.0 resolution, unchanged explicit compiler
check success, and the existing real-browser component-runtime proof. That
browser proof verifies compiler-emitted instance plans, caller-owned slotted
content, exact Slot-binding programs, instance-qualified State, and absence of
runtime lookup/traversal authority.

## Boundary

M6-A does not add inputs, props, children arguments, component registration,
Slot forwarding, Context declarations, Context lookup, or lifecycle helpers.
Context is deliberately separate: the inherited `Owner.instanceField`
designator cannot be truthfully typed by TypeScript without a language change
or a suppression. Any departure must be a documented compiler/framework
language contract, never a framework shim.
