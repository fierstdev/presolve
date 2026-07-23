const invalidAttributes: PresolveIntrinsicAttributes = {
  "aria-invalid": "true",
  "aria-live": false,
  onKeydown: (event: unknown) => event,
};

void invalidAttributes;
