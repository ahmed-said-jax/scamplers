import { env } from "$env/dynamic/private";
const BACKEND_URL = `http://${env.BACKEND_HOST}:${env.BACKEND_PORT}/api`;

export function backendRequest(data: object, route: string, method: string ='POST', api_key: string | undefined=undefined): Request {
  const url = `${BACKEND_URL}/${route}`;

  const request = new Request(url, { method, body: JSON.stringify(data), headers: { "Content-Type": "application/json" } });
  if (api_key) {
    request.headers.append("X-API-Key", api_key)
  }

  return request;
}

export class ApiError {
  readonly status: number;
  readonly error: object;
}
