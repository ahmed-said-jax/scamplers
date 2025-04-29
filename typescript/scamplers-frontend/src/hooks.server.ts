import { handle as authenticationHandle } from './auth';
import { redirect, type Handle, type HandleFetch } from '@sveltejs/kit';
import { sequence } from '@sveltejs/kit/hooks';
import { BACKEND_URL, backendRequest } from '$lib/server/backend';

export const handleFetch: HandleFetch = async ({event, fetch}) => {
  let request: Request;

  if (event.url.host === BACKEND_URL) {
    request = await backendRequest({event});
  } else {
    request = event.request;
  }

  return fetch(request);
}

async function authorizationHandle({ event, resolve }) {
  if (event.url.pathname === '/auth/signin') {
    return resolve(event)
  }

  const session = await event.locals.auth();
  if (!session) {
    throw redirect(303, '/auth/signin');
  }

  return resolve(event);
}

export const handle: Handle = sequence(authenticationHandle, authorizationHandle)
