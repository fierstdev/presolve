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

  /** Declares a compiler-recognized Action without installing an event wrapper. */
  function action(): PresolveMethodDecorator;

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

  /** A standard-decorator-compatible declaration for `@action()`. */
  type PresolveMethodDecorator = <
    This,
    Value extends (this: This, ...args: any[]) => unknown,
  >(
    value: Value,
    context: ClassMethodDecoratorContext<This, Value>
  ) => Value | void;

  namespace JSX {
    type Element = unknown;

    interface IntrinsicElements {
      [elementName: string]: {
        [attributeName: string]: unknown;
      };
    }
  }
}
