import { SvelteKitAuth, type DefaultSession } from "@auth/sveltekit"
import Entra from "@auth/sveltekit/providers/microsoft-entra-id"
import { env } from "$env/dynamic/private"
import { CreatedUser, NewPerson } from "scamplers-core";

declare module "@auth/sveltekit" {
  interface Session {
    user: {
      id: string
    } & DefaultSession["user"]
  }
}

async function create_user(person: NewPerson): CreatedUser {}


export const { handle, signIn, signOut } = SvelteKitAuth({
  providers: [
    Entra({
      clientId: env.AUTH_MICROSOFT_ENTRA_ID_ID,
      clientSecret: env.AUTH_MICROSOFT_ENTRA_ID_SECRET,
      client: {token_endpoint_auth_method: "client_secret_post"},
    }),
  ],
  callbacks: {
    async jwt({ token, user, profile }) {
      if (user && profile) {
        const user = await create_user({ name: "", orcid: "", ms_user_id: "", email: "", institution_id: "", roles: [] });
        token.user_id = user.id;

      }
      return token
    },
    async session({ session, token }) {
      if (typeof token.user_id === "string") {
        session.user.id = token.user_id
      }

      return session
    },
  },
  trustHost: true
}
)
