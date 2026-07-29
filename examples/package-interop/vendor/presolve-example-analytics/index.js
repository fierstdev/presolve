export function recordVisit() {
  const output = document.getElementById("visit-count");
  if (output !== null) {
    output.textContent = String(Number(output.textContent ?? "0") + 1);
  }
}
