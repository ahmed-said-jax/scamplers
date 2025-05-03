import { SvelteKitAuth, type DefaultSession } from '@auth/sveltekit';
import Entra from '@auth/sveltekit/providers/microsoft-entra-id';
import { NewPerson, CreatedUser, NewPersonBuilder } from 'scamplers-core';
import { AUTH_SECRET, MICROSOFT_ENTRA_ID_ID, MICROSOFT_ENTRA_ID_SECRET } from '$lib/server/secrets';
import { BACKEND_URL } from '$lib/server/backend';
import { backendRequest } from '$lib/server/backend';
import { type JWT } from '@auth/core/jwt';

declare module '@auth/sveltekit' {
	interface Session {
		user: {
			id: string;
			apiKey: string | undefined;
		} & DefaultSession['user'];
	}
}
declare module '@auth/core/jwt' {
	interface JWT {
		userId: string;
		apiKey: string;
	}
}

async function createUser(person: NewPerson): Promise<CreatedUser | null> {
	let request = new Request(`${BACKEND_URL}/user`, { method: 'POST', body: person.to_json() });
	request = await backendRequest({ request });

	const response = await fetch(request);

	return CreatedUser.from_json(await response.text());
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
		async jwt({ token, profile }): Promise<JWT | null> {
			if (!profile) {
				return token;
			}

			if (
				!(
					profile.name &&
					profile.email &&
					typeof profile.oid === 'string' &&
					typeof profile.tid === 'string'
				)
			) {
				return null;
			}

			const { name, email, oid, tid } = profile;

			let newPerson = NewPerson.new()
				.name(name)
				.email(email)
				.ms_user_id(oid)
				.institution_id(tid)
				.build();

			if (!createdUser) {
				return null;
			}

			const { id, api_key } = createdUser;

			token.userId = id;
			token.apiKey = api_key;

			return token;
		},

		async session({ session, token }) {
			const { userId, apiKey } = token;

			session.user.id = userId;
			session.user.apiKey = apiKey;

			return session;
		}
	},
	trustHost: true
});
