import { action, Component } from "presolve";
import { recordMetric as emitMetric } from "./V2PackageHelper.js";

function recordMetric(): void {}

export class V2Package extends Component {
  send = action(() => {
    emitMetric();
  });

  local = action(() => {
    recordMetric();
  });
}
