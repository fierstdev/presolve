/** Compiler intrinsics: these declarations deliberately have no runtime authority. */
const identityClass = (value) => value;
const emptyField = () => undefined;

export function component() { return identityClass; }
export function action() { return identityClass; }
export function computed() { return identityClass; }
export function effect() { return identityClass; }
export function state(value) { return value; }
export function slot() { return emptyField; }
export function context() { return emptyField; }
export function provide() { return emptyField; }
export function consume() { return emptyField; }
export function form() { return emptyField; }
export function field() { return emptyField; }
export function validate() { return emptyField; }
export function resource() { return emptyField; }
export function loader() { return emptyField; }
export function serverAction() { return identityClass; }
