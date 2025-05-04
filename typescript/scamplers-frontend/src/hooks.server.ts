import { handle as authenticationHandle } from './auth';
import { redirect, type Handle } from '@sveltejs/kit';
import { sequence } from '@sveltejs/kit/hooks';

const authorizationHandle: Handle = async ({ event, resolve }) => {
	if (['/auth/signin', '/health'].includes(event.url.pathname)) {
		return resolve(event);
	}

	const session = await event.locals.auth();
	if (!session) {
		throw redirect(303, '/auth/signin');
	}

	return resolve(event);
};

export const handle = sequence(authenticationHandle, authorizationHandle);
