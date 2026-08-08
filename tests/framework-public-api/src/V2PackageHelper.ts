export function recordMetric(_category: string, _value: number): void {}

export async function recordMetricAsync(
  _category: string,
  _signal: AbortSignal,
): Promise<void> {}

export interface PostRecord {
  slug: string;
  title: string;
}

export interface NotFound {
  code: "not_found";
}

export async function loadPost(
  params: import("presolve").RouteParameters,
  _signal: AbortSignal,
): Promise<PostRecord> {
  return { slug: params.slug ?? "", title: "Post" };
}
