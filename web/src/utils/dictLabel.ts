import type { DictItem } from "@/api/dictionary";

/** 与 gin-vue-admin `filterDict(value, options)` 一致 */
export function dictLabel(
  value: number | null | undefined,
  options: DictItem[]
): string {
  if (value == null) return "—";
  const row = options.find((item) => item.value === value);
  return row?.label ?? String(value);
}
