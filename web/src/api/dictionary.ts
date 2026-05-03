import { apiFetch } from "@/api/http";

export interface DictItem {
  label: string;
  value: number;
}

/** 对应 gin-vue-admin `getDictFunc(type)`，类型名如 `querysql_cate`、`sql_datasource_cate` */
export async function getDictionary(type: string): Promise<DictItem[]> {
  const data = await apiFetch<{ list: DictItem[] }>(
    `/api/dictionaries/${encodeURIComponent(type)}`
  );
  return data.list;
}
