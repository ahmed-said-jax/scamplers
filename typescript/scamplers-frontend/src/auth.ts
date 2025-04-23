import { SvelteKitAuth, type DefaultSession } from "@auth/sveltekit"
import Entra from "@auth/sveltekit/providers/microsoft-entra-id"
import { CreatedUser, NewPerson } from "scamplers-core";
import { AUTH_SECRET, MICROSOFT_ENTRA_ID_ID, MICROSOFT_ENTRA_ID_SECRET } from "$lib/server/secrets";
import { BACKEND_URL, backendRequest, handleFetch } from "./hooks.server";

declare module "@auth/sveltekit" {
  interface Session {
    user: {
      id: string,
      api_key: string | undefined
    } & DefaultSession["user"]
  }
}

async function createUser (person: NewPerson): Promise<CreatedUser> {
  let request = new Request(`${BACKEND_URL}/user`, { method: 'POST', body: JSON.stringify({name: person.name, email: person.email, institution_id: person.institution_id, ms_user_id: person.ms_user_id, roles: person.roles}) });
  request = await backendRequest({ request });

  const response = await fetch(request);
  const created_user: CreatedUser = await response.json(); // TODO: this should be type-checked

  return created_user;
}

export const { handle, signIn, signOut } = SvelteKitAuth({
  secret: AUTH_SECRET,
  providers: [
    Entra({
      clientId: MICROSOFT_ENTRA_ID_ID,
      clientSecret: MICROSOFT_ENTRA_ID_SECRET,
      client: {token_endpoint_auth_method: "client_secret_post"},
    }),
  ],
  callbacks: {
    async signIn({user, profile}) {
      if (user && profile) {
        if (profile.tid && profile.oid) {
          return true;
        }
      }
      return false;
    },
    async jwt({ token, user, profile }) {
      const new_person = new NewPerson(user.name, user.email, profile.tid, profile.oid);

      const createdUser = await createUser(new_person);

      token.user_id = createdUser.id;
      token.api_key = createdUser.api_key;

      return token
    },
    async session({ session, token }) {

      session.user.id = token.user_id;
      session.user.api_key = token.api_key;

      return session
    }
  },
}
)
