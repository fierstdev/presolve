/** Compiler intrinsics with no framework runtime authority. */
export type PresolveClassDecorator = <TClass extends abstract new (...args: never[]) => object>(
  value: TClass,
  context: ClassDecoratorContext<TClass>
) => TClass | void;
export type PresolveMethodDecorator = <
  This,
  Value extends (this: This, ...args: any[]) => unknown,
>(
  value: Value,
  context: ClassMethodDecoratorContext<This, Value>
) => Value | void;
export type PresolveGetterDecorator = <This, Value>(
  value: (this: This) => Value,
  context: ClassGetterDecoratorContext<This, Value>
) => ((this: This) => Value) | void;
export type PresolveFieldDecorator = <This, Value>(
  value: undefined,
  context: ClassFieldDecoratorContext<This, Value>
) => void;

/** @deprecated Alpha compatibility decorator. V2 components extend `Component`. */
export declare function component(): PresolveClassDecorator;
/**
 * Creates a V2 action instance field.  The overload without a handler is
 * retained solely for alpha decorator compatibility.
 */
export declare function action<This, Args extends readonly unknown[], Value>(
  handler: (this: This, ...args: Args) => Value
): (this: This, ...args: Args) => Value;
/** @deprecated Alpha compatibility decorator. */
export declare function action(): PresolveMethodDecorator;
export declare function computed(): PresolveGetterDecorator;
/** Creates a V2 browser effect instance field. */
export declare function effect<This>(
  handler: (this: This) => void | (() => void)
): void;
/** @deprecated Alpha compatibility decorator. */
export declare function effect(): PresolveMethodDecorator;
export declare function state<T>(initialValue: T): T;

/**
 * Manifest-backed browser environment access for decorator-free V2 source.
 * Values are admitted only when a matching compiler environment manifest
 * classifies the requested name as `PRESOLVE_PUBLIC_*`.
 */
export declare const environment: {
  public(name: string): string;
};

export interface SlotContent {
  readonly __presolveSlotContentBrand: unique symbol;
}
/**
 * A V2 slot field initializer that remains callable as the alpha field
 * decorator while that compatibility surface is supported.
 */
export type PresolveSlotField = SlotContent & PresolveFieldDecorator;
export declare function slot(): PresolveSlotField;

export declare function context(): PresolveFieldDecorator;
export type ContextDesignator = `${string}.${string}`;
export declare function provide(context: ContextDesignator): PresolveFieldDecorator;
export declare function consume(context: ContextDesignator): PresolveFieldDecorator;

export declare function form(): PresolveFieldDecorator;
/** A compiler-owned Form declaration marker; it is not a runtime controller. */
export interface Form {
  readonly __presolveFormBrand: unique symbol;
}
export type FormSerialization = "json" | "form-data" | "url-encoded";
export interface FormField<Value> {
  readonly value: Value;
  readonly pristine: boolean;
  readonly dirty: boolean;
  readonly touched: boolean;
  readonly valid: boolean;
  readonly issues: readonly unknown[];
  readonly __presolveFormFieldBrand: unique symbol;
}
export type FormFieldTree =
  | FormField<unknown>
  | { readonly [name: string]: FormFieldTree }
  | readonly FormFieldTree[];
export type FormValue<Tree> =
  Tree extends FormField<infer Value>
    ? Value
    : Tree extends readonly (infer Item)[]
      ? FormValue<Item>[]
      : Tree extends object
        ? { [Key in keyof Tree]: FormValue<Tree[Key]> }
        : never;
export interface FormFieldOptions<Value> {
  initial: Value;
  validate?: readonly ValidationRule[];
}
export interface FormSubmission<Value> {
  readonly value: Value;
  readonly signal: AbortSignal;
}
export interface FormDefinition<Fields extends FormFieldTree> {
  serialization?: FormSerialization;
  fields: Fields;
  submit?: (submission: FormSubmission<FormValue<Fields>>) => void | Promise<void>;
}
export interface DefinedForm<Fields extends FormFieldTree> extends Form {
  readonly fields: Fields;
  readonly pristine: boolean;
  readonly dirty: boolean;
  readonly touched: boolean;
  readonly submitting: boolean;
  readonly submitted: boolean;
  readonly valid: boolean;
  readonly issues: readonly unknown[];
}
/** Declares a canonical decorator-free V2 Form. */
export declare function defineForm<Fields extends FormFieldTree>(
  definition: FormDefinition<Fields>
): DefinedForm<Fields>;
/** Declares one statically recoverable field inside `defineForm({ fields })`. */
export declare function field<Value>(options: FormFieldOptions<Value>): FormField<Value>;
export declare function serialize(
  format: FormSerialization
): PresolveFieldDecorator;
/** @deprecated Alpha compatibility decorator. */
export declare function field(form: string, path?: string): PresolveFieldDecorator;
export interface ValidationRule {
  readonly __presolveValidationRuleBrand: unique symbol;
}
export declare function validate(rule: ValidationRule): PresolveFieldDecorator;
export declare function required(): ValidationRule;
export declare function submit(form: string): PresolveMethodDecorator;

export type ResourceState = "idle" | "pending" | "ready" | "failed" | "cancelled";
export interface Resource<Data, Error> {
  readonly data: Data | null;
  readonly error: Error | null;
  readonly state: ResourceState;
  readonly __presolveResourceBrand: unique symbol;
}
export declare function resource(endpoint: string): PresolveFieldDecorator;
export declare function loader(endpoint: string): PresolveFieldDecorator;
export declare function serverAction(endpoint: string): PresolveMethodDecorator;
export declare function opaque(
  packageSpecifier: string,
  exportName: string
): PresolveMethodDecorator;

export declare abstract class Component<Props = {}> {}

declare global {
  namespace JSX {
    type Element = unknown;
    interface IntrinsicElements {
      [elementName: string]: unknown;
    }
  }
}
