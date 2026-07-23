# Phase N N3-A serializable State replacement contract

State may hold recursively serializable records and arrays. An Action may
replace the complete State field with a serializable literal record or array;
the compiler publishes that exact value as the Action operand and generated
runtime writes it as one field replacement. The value remains component-instance
owned and is eligible for the existing resume codecs. Nested/member writes,
aliases, mutation through a retained object reference, spreads, and collection
methods are not admitted by N3-A.
