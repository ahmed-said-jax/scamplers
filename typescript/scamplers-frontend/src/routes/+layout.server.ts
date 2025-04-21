import type { LayoutServerLoad } from "./$types"
import { getToken } from "@auth/core/jwt";
import {decode} from '@auth/core/jwt';

export const load: LayoutServerLoad = async (event) => {
  const session = await event.locals.auth()

  console.log(session.user.id_token);

  // const tok = await getToken({req: event.request, raw: true});
  // const dec = await decode({token: tok, secret: "EydAz3E1gw+09MWOhICgk2E9ZKeDrQhFMzXGSaX8xLU=", salt: ""})

  return {
    session,
  }
}
