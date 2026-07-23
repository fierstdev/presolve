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
   * Declares one compiler-recorded third-party terminal on an otherwise empty
   * Action. The compiler resolves the package contract and supplies all
   * runtime behavior; this declaration never imports or invokes package code.
   */
  function opaque(packageSpecifier: string, exportName: string): PresolveMethodDecorator;

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

  /** The compiler-owned Form declaration marker; it has no runtime value. */
  interface Form {
    readonly __presolveFormBrand: unique symbol;
  }

  /** Declares a component-owned Form without creating a form controller. */
  function form(): PresolveFieldDecorator;

  /** Declares the compiler-owned serialization policy for a Form. */
  function serialize(
    format: "json" | "form-data" | "url-encoded"
  ): PresolveFieldDecorator;

  /**
   * A compiler-resolved local Form identity. It is source syntax, never a
   * runtime controller lookup or an instance expression evaluated by a
   * decorator.
   */
  type FormDesignator = string;

  /**
   * Declares a compiler-owned Field attached to the named local Form. An
   * optional static path changes only the compiler-issued serialization path;
   * it never creates a runtime object controller or dynamic lookup.
   */
  function field(form: FormDesignator, path?: string): PresolveFieldDecorator;

  /** A compiler-owned validation-rule expression marker. */
  interface ValidationRule {
    readonly __presolveValidationRuleBrand: unique symbol;
  }

  /** Declares compiler-owned validation for one Form Field. */
  function validate(rule: ValidationRule): PresolveFieldDecorator;

  /** Declares compiler-owned required validation; it has no runtime behavior. */
  function required(): ValidationRule;

  /** Declares that a compiler-owned Action is the named Form's submit action. */
  function submit(form: FormDesignator): PresolveMethodDecorator;

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
