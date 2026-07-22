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

  /** Declares a compiler-recognized getter without caching or invalidation behavior. */
  function computed(): PresolveGetterDecorator;

  /** Declares a compiler-recognized terminal capability method with no hook runtime. */
  function effect(): PresolveMethodDecorator;

  /** Declares a compiler-owned static Context identity and optional default value. */
  function context(): PresolveFieldDecorator;

  /**
   * A compiler-resolved qualified Context identity. The string is source
   * syntax, not a runtime lookup key: the compiler validates and resolves it
   * to the decorated static Context declaration before artifacts are emitted.
   */
  type ContextDesignator = `${string}.${string}`;

  /** Declares a compiler-owned Provider bound to a static Context identity. */
  function provide(contextDesignator: ContextDesignator): PresolveFieldDecorator;

  /** Declares a compiler-owned Consumer bound to a static Context identity. */
  function consume(contextDesignator: ContextDesignator): PresolveFieldDecorator;

  /** Declares a compiler-recognized slot field with no children runtime. */
  function slot(): PresolveFieldDecorator;

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

  /** A standard-decorator-compatible declaration for `@computed()`. */
  type PresolveGetterDecorator = <This, Value>(
    value: (this: This) => Value,
    context: ClassGetterDecoratorContext<This, Value>
  ) => ((this: This) => Value) | void;

  /** A standard-decorator-compatible declaration for compiler field markers. */
  type PresolveFieldDecorator = <This, Value>(
    value: undefined,
    context: ClassFieldDecoratorContext<This, Value>
  ) => void;

  /** The compiler-owned slot-content marker; it has no framework runtime value. */
  interface SlotContent {
    readonly __presolveSlotContentBrand: unique symbol;
  }

  namespace JSX {
    type Element = unknown;

    interface IntrinsicElements {
      [elementName: string]: {
        [attributeName: string]: unknown;
      };
    }
  }
}
