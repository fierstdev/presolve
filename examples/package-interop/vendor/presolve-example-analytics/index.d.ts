export declare function recordVisit(
  category: string,
  value: number,
  enabled: boolean,
  metadata: null,
): void;

export declare function recordVisitAsync(
  category: string,
  signal: AbortSignal,
): Promise<void>;
