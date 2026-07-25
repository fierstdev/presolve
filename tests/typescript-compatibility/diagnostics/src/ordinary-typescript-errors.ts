export {};

const count: number = "not-a-number";

function acceptsNumber(value: number): number {
  return value;
}

acceptsNumber("not-a-number");
void count;
