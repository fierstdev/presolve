import { loadProfile } from "profile-service";

@component("x-profile")
class Profile extends Component {
  @resource("loadProfile") profile!: Resource<string, string>;

  @computed()
  get profileName(): string | null {
    return this.profile.data;
  }

  render() {
    return <main>{this.profileName}</main>;
  }
}
