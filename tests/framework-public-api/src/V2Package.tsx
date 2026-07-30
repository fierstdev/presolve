import { action, Component } from "presolve";
import {
  recordMetric as emitMetric,
  recordMetricAsync as emitMetricAsync,
} from "./V2PackageHelper.js";

function recordMetric(_category: string, _value: number): void {}

export class V2Package extends Component {
  send = action((category: string, value: number) => {
    emitMetric(category, value);
  });

  sendAsync = action(async (category: string, signal: AbortSignal) => {
    await emitMetricAsync(category, signal);
  });

  local = action((category: string, value: number) => {
    recordMetric(category, value);
  });
}
