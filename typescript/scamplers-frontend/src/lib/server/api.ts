import { env } from "$env/dynamic/private";
export async function scamplersBackendRequest(data: object, route: string, fetch: (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>): object {
  const url = `http://${env.BACKEND_HOST}:${env.BACKEND_PORT}/${route}`
}
