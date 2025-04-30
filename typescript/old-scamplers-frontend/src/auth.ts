import { SvelteKitAuth, type DefaultSession } from '@auth/sveltekit';
import Entra from '@auth/sveltekit/providers/microsoft-entra-id';
import { NewPerson } from 'scamplers-core';
import { AUTH_SECRET, MICROSOFT_ENTRA_ID_ID, MICROSOFT_ENTRA_ID_SECRET } from '$lib/server/secrets';
import { BACKEND_URL } from '$lib/server/backend';
import { backendRequest } from '$lib/server/backend';

declare module '@auth/sveltekit' {
	interface Session {
		user: {
			id: string;
			apiKey: string | undefined;
		} & DefaultSession['user'];
	}
}

async function createUser(person: NewPerson): Promise<object> {
	let request = new Request(`${BACKEND_URL}/user`, { method: 'POST', body: person.toString() });
	request = await backendRequest({ request });

	const response = await fetch(request);

	return await response.json();
}

export const { handle, signIn, signOut } = SvelteKitAuth({
	secret: AUTH_SECRET,
	providers: [
		Entra({
			clientId: MICROSOFT_ENTRA_ID_ID,
			clientSecret: MICROSOFT_ENTRA_ID_SECRET
		})
	],
	callbacks: {
		async signIn({ profile }) {
			if (profile && profile.tid && profile.oid) {
				return true;
			}
			return false;
		},
		async jwt({ token, profile }) {
			if (!profile) {
				return token;
			}

			const newPerson = new NewPerson(profile.name, profile.email, profile.tid, profile.oid);

			const { id, api_key } = await createUser(newPerson);

			token.userId = id;
			token.apiKey = api_key;

			return token;
		},
		async session({ session, token }) {
			session.user.id = token.userId;
			session.user.apiKey = token.apiKey;

			return session;
		}
	},
	trustHost: true
});
