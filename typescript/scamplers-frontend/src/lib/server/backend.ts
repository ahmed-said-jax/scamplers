import { env } from '$env/dynamic/private';
import { AUTH_SECRET } from './secrets';
import type { RequestEvent } from '@sveltejs/kit';

const BACKEND_HOST = env.SCAMPLERS_BACKEND_HOST ?? env.BACKEND_HOST;
const BACKEND_PORT = env.SCAMPLERS_BACKEND_PORT ?? env.BACKEND_PORT;

export const BACKEND_URL = `http://${BACKEND_HOST}:${BACKEND_PORT}`;

export async function backendRequest({ event, request }: { event?: RequestEvent; request?: Request; }): Promise<Request> {
  if (!request === !event ) {
    throw 'must specify exactly one of `request` or `event`';
  }

  const updatedRequest = request ?? event!.request;

  const auth = btoa(`scamplers-frontend:${AUTH_SECRET}`);

  const headers = [["Content-Type", "application/json"], ["Authorization", `Basic ${auth}`]];

  const session = await event?.locals.auth();
  if (session && session.user.apiKey) {
    headers.push(["X-API-Key", session.user.apiKey]);
  }

  for (const [name, value] of headers) {
    updatedRequest.headers.set(name, value);
  }

  return updatedRequest;
}
