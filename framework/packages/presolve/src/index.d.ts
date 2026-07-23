export declare function component(): ClassDecorator;
export declare function action(): MethodDecorator;
export declare function computed(): MethodDecorator;
export declare function effect(): MethodDecorator;
export declare function state<T>(initialValue: T): T;
export declare function slot(): PropertyDecorator;
export declare function context(): PropertyDecorator;
export declare function provide(context: string): PropertyDecorator;
export declare function consume(context: string): PropertyDecorator;
export declare function form(): PropertyDecorator;
export declare function field(form: string, path?: string): PropertyDecorator;
export declare function validate(rule: unknown): PropertyDecorator;
export declare function resource(endpoint: string): PropertyDecorator;

export declare abstract class Component {}
