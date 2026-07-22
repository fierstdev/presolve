export {};

declare global {
  /**
   * The compiler-recognized base class for the frozen component declaration
   * form. It carries no framework runtime behavior.
   */
  abstract class Component {}

  /**
   * Declares the compiler-recognized component form without registering or
   * wrapping the class at runtime.
   */
  function component(elementName: string): PresolveClassDecorator;

  /**
   * Describes the initializer shape for compiler-recognized State. The
   * compiler, not this declaration, establishes reactive State semantics.
   */
  function state<T>(initialValue: T): T;

  /** A standard-decorator-compatible declaration for `@component(...)`. */
  type PresolveClassDecorator = <TClass extends abstract new (...args: never[]) => object>(
    value: TClass,
    context: ClassDecoratorContext<TClass>
  ) => TClass | void;

  namespace JSX {
    type Element = unknown;

    interface IntrinsicElements {
      [elementName: string]: {
        [attributeName: string]: unknown;
      };
    }
  }
}
