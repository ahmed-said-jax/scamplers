import type { LayoutServerLoad } from "./$types"

export const load: LayoutServerLoad = async (event) => {
  const session = await event.locals.auth();

  if (session) {
    session.user.api_key = undefined;
  }

  return {
    session,
  }
}
