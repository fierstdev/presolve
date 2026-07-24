import { Filter, Todo } from "./types";

@component("x-type-system")
class TypeSystem extends Component {
  filter: Filter = state("all");
  selected: Todo = state({ id: "1", title: "Write tests", completed: false });
  todos: { id: string; title: string; completed: boolean }[] = state([]);
  enabled = state(true);

  @action()
  add(todo: Todo): boolean {
    this.enabled = todo.completed;
    return true;
  }

  render() {
    return <section>{this.enabled && <p>{this.filter}</p>}<ul>{this.todos.map((todo, index) => <li key={todo.id}>{index}: {todo.title}{todo.missing}</li>)}</ul></section>;
  }
}
