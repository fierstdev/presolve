import {
  Component,
  loader,
  resource,
  type Resource,
  type ResourceContext,
  type RouteParameters,
} from "presolve";
import {
  loadPost,
  loadProfile,
  type NotFound,
  type PostRecord,
} from "./V2PackageHelper.js";

export class V2Loader extends Component {
  post: Resource<PostRecord, NotFound> = loader<PostRecord, NotFound>(
    async (params: RouteParameters, signal: AbortSignal) => loadPost(params, signal),
  );

  render() {
    return <article>{this.post.data?.title}</article>;
  }
}

export class V2Resource extends Component {
  profile: Resource<PostRecord, NotFound> = resource<PostRecord, NotFound>(
    async (context: ResourceContext) => loadProfile(context),
  );

  render() {
    return <article>{this.profile.data?.title}</article>;
  }
}
