export interface ProjectRecord {
  readonly name: string;
}

export function createRecord(name: string): ProjectRecord {
  return { name };
}
