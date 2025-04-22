import { SvelteKitAuth, type DefaultSession } from "@auth/sveltekit"
import Entra from "@auth/sveltekit/providers/microsoft-entra-id"
import { env } from "$env/dynamic/private"
import { CreatedUser, NewPerson } from "scamplers-core";
import { backendRequest, ApiError } from "$lib/server/api";
import { read } from "$app/server";

declare module "@auth/sveltekit" {
  interface Session {
    user: {
      id: string,
      api_key: string | undefined
    } & DefaultSession["user"]
  }
}

const auth_secret = await read('/run/secrets/auth_secret').text() || env.AUTH_SECRET;
const microsoft_entra_id_id = await read('/run/secrets/auth_microsoft_entra_id_id').text() || env.AUTH_MICROSOFT_ENTRA_ID_ID;
const microsoft_entra_id_secret = await read('/run/secrets/auth_microsoft_entra_id_secret').text() || env.AUTH_MICROSOFT_ENTRA_ID_SECRET;

export const { handle, signIn, signOut } = SvelteKitAuth({
  secret: auth_secret,
  providers: [
    Entra({
      clientId: microsoft_entra_id_id,
      clientSecret: microsoft_entra_id_secret,
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
      const request = backendRequest(new_person, "user");

      const response = await fetch(request);
      const created_user: CreatedUser | ApiError = await response.json();

      token.user_id = created_user.id;
      token.api_key = created_user.api_key;

      return token
    },
    async session({ session, token }) {
      session.user.id = token.user_id;
      session.user.api_key = token.api_key;

      return session
    },
  },
}
)
