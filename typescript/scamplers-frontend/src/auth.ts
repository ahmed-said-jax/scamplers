import { SvelteKitAuth, type DefaultSession } from "@auth/sveltekit"
import Entra, {type MicrosoftEntraIDProfile} from "@auth/sveltekit/providers/microsoft-entra-id"
import { env } from "$env/dynamic/private"
import { JWT } from "@auth/core/jwt"

declare module "@auth/sveltekit" {
  interface Session {
    user: {
      id: string
      api_key: string,
    } & DefaultSession["user"]
  }
}

async function create_user() {}


export const { handle, signIn, signOut } = SvelteKitAuth({
  providers: [
    Entra({
      clientId: env.AUTH_MICROSOFT_ENTRA_ID_ID,
      clientSecret: env.AUTH_MICROSOFT_ENTRA_ID_SECRET,
      client: {token_endpoint_auth_method: "client_secret_post"},
      async profile(profile) {
        return {...profile}
      }
    }),
  ],
  callbacks: {
    jwt({ token, user, profile }) {
      if (user && profile) {
        token.user_id = profile.oid;
        token.institution_id = profile.tid;
      }
      return token
    },
    async session({ session, token }) {
      session.user.id = token.id
      session.user.tid = token.tid

      return session
    },
  },
  trustHost: true
}
)
