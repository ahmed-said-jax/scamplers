export { handle } from './auth';
import { type HandleFetch, type RequestEvent } from '@sveltejs/kit';
import { env } from "$env/dynamic/private";
import { AUTH_SECRET } from '$lib/server/secrets';
import { updated } from '$app/state';

const BACKEND_HOST = env.SCAMPLERS_BACKEND_HOST ?? env.BACKEND_HOST;
const BACKEND_PORT = env.SCAMPLERS_BACKEND_PORT ?? env.BACKEND_PORT;

export const BACKEND_URL = `http://${BACKEND_HOST}:${BACKEND_PORT}`;

export async function backendRequest({ event, request}: {event?: RequestEvent, request?: Request}): Promise<Request> {
  if ((typeof request === 'undefined') === (typeof event === 'undefined')) {
    throw 'must specify exactly one of `request` and `event`';
  }

  let updated_request: Request;

  if (request) {
    updated_request = request;
  } else if (event) {
    updated_request = event.request;
  } else {
    throw 'this code is unreachable';
  }

  const auth = btoa(`scamplers-frontend:${AUTH_SECRET}`);

  const headers = [["Content-Type", "application/json"], ["Authorization", `Basic ${auth}`]];

  const session = await event?.locals.auth();
  if (session && session.user.api_key) {
    headers.push(["X-API-Key", session.user.api_key])
  }

  for (const [name, value] of headers) {
    updated_request.headers.set(name, value);
  }

  return updated_request
}

export const handleFetch: HandleFetch = async ({event, fetch}) => {
  let request: Request;

  if (event.request.url.startsWith(BACKEND_URL)) {
    request = await backendRequest({event});
  } else {
    request = event.request;
  }

  return fetch(request);
}
