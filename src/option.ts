export function mapOptional<T, U>(
  value: T | null | undefined,
  fn: (value: T) => U
): U | null {
  return value === null || value === undefined ? null : fn(value);
}
