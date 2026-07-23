# Phase N N4-C keyboard Action event contract

N4-C admits onKeydown only for an existing compiler-recognized Action binding.
The compiler records keydown as an exact template event and generated runtime
delegates it to the existing completed Action batch. No keyboard payload,
pressed-key branch, browser-event object, callback closure, or generic listener
is exposed to authored code.

All other event names remain rejected by PSC1005. Key-specific behavior needs a
separate compiler contract because it requires a typed event projection and
branch semantics.
