import { Component, loader, type Resource, type RouteParameters } from "presolve";
import { loadPost, type NotFound, type PostRecord } from "./V2PackageHelper.js";

export class V2Loader extends Component {
  post: Resource<PostRecord, NotFound> = loader<PostRecord, NotFound>(
    async (params: RouteParameters, signal: AbortSignal) => loadPost(params, signal),
  );

  render() {
    return <article>{this.post.data?.title}</article>;
  }
}
