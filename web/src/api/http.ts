const TOKEN_KEY = "rs-admin-token";

export function getToken(): string | null {
  return sessionStorage.getItem(TOKEN_KEY);
}

export function setToken(token: string | null) {
  if (token) sessionStorage.setItem(TOKEN_KEY, token);
  else sessionStorage.removeItem(TOKEN_KEY);
}

export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public body?: unknown
  ) {
    super(message);
    this.name = "ApiError";
  }
}

async function parseJson(res: Response): Promise<unknown> {
  const text = await res.text();
  if (!text) return null;
  try {
    return JSON.parse(text);
  } catch {
    return text;
  }
}

export async function apiFetch<T>(
  path: string,
  init?: RequestInit
): Promise<T> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(init?.headers as Record<string, string> | undefined),
  };
  const tok = getToken();
  if (tok) headers["Authorization"] = `Bearer ${tok}`;

  const res = await fetch(path, {
    ...init,
    headers,
  });
  if (res.status === 204) return undefined as T;
  const data = await parseJson(res);
  if (res.status === 401 && !path.includes("/api/auth/login")) {
    setToken(null);
    const loginPath = "/login";
    if (
      typeof window !== "undefined" &&
      window.location.pathname !== loginPath
    ) {
      window.location.assign(
        `${loginPath}?redirect=${encodeURIComponent(window.location.pathname + window.location.search)}`
      );
    }
  }
  if (!res.ok) {
    const msg =
      typeof data === "object" &&
      data !== null &&
      "message" in data &&
      typeof (data as { message: unknown }).message === "string"
        ? (data as { message: string }).message
        : res.statusText;
    throw new ApiError(msg, res.status, data);
  }
  return data as T;
}

export interface PageResult<T> {
  list: T[];
  total: number;
  page: number;
  pageSize: number;
}
