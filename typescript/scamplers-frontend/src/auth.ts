import { SvelteKitAuth, type DefaultSession } from '@auth/sveltekit';
import Entra from '@auth/sveltekit/providers/microsoft-entra-id';
import { Institution, NewPerson } from 'scamplers-core';
import {
	AUTH_SECRET,
	MICROSOFT_ENTRA_ID_ID,
	MICROSOFT_ENTRA_ID_SECRET,
	MICROSOFT_ENTRA_ID_ISSUER
} from '$lib/server/secrets';
import { scamplersClient } from '$lib/server/backend';
import { type JWT } from '@auth/core/jwt';

declare module '@auth/sveltekit' {
	interface Session {
		user: {
			id: string;
			apiKey: string | undefined;
			institution: Institution;
		} & DefaultSession['user'];
	}
}
declare module '@auth/core/jwt' {
	interface JWT {
		userId: string;
		userApiKey: string;
	}
}

export const { handle, signIn, signOut } = SvelteKitAuth({
	secret: AUTH_SECRET,
	providers: [
		Entra({
			clientId: MICROSOFT_ENTRA_ID_ID,
			clientSecret: MICROSOFT_ENTRA_ID_SECRET,
			issuer: MICROSOFT_ENTRA_ID_ISSUER
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

			const newPerson = NewPerson.new()
				.name(name)
				.email(email)
				.ms_user_id(oid)
				.institution_id(tid)
				.build();

			const createdUser = await scamplersClient.send_new_ms_login(newPerson);

			token.userId = createdUser.person.id;
			token.userApiKey = createdUser.api_key;

			return token;
		},

		async session({ session, token }) {
			const { userId, userApiKey } = token;

			session.user.id = userId;
			session.user.apiKey = userApiKey;

			return session;
		}
	},
	trustHost: true
});
