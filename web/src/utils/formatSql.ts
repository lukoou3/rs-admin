import { format as sqlFormat } from "@/sql-format";

/**
 * 与旧项目 `queryPlatform.vue`、`format.vue` 中「格式化 sql」选项一致（纯前端）。
 */
export function formatSqlLikeLegacy(sql: string): string {
  return sqlFormat(sql, {
    language: "spark",
    tabWidth: 4,
    keywordCase: "lower",
    logicalOperatorNewline: "before",
    expressionWidth: 200,
    newlineBeforeSemicolon: true,
  });
}
