export function recordVisit(category, value, enabled, metadata) {
  const output = document.getElementById("visit-count");
  if (output !== null) {
    output.textContent = String(Number(output.textContent ?? "0") + value);
  }
  const detail = document.getElementById("visit-detail");
  if (detail !== null) {
    detail.textContent = `${category}:${value}:${enabled}:${String(metadata)}`;
  }
}

export async function recordVisitAsync(category, signal) {
  window.__PACKAGE_ASYNC_STARTS__ = (window.__PACKAGE_ASYNC_STARTS__ ?? 0) + 1;
  await new Promise((resolve, reject) => {
    const timer = setTimeout(resolve, category === "slow" ? 250 : 20);
    signal.addEventListener("abort", () => {
      clearTimeout(timer);
      reject(new DOMException("package invocation aborted", "AbortError"));
    }, { once: true });
  });
  if (category === "fail") {
    throw new Error("analytics rejected fail");
  }
  const output = document.getElementById("async-result");
  if (output !== null) output.textContent = category;
}
