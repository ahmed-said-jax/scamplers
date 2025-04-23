import { env } from "$env/dynamic/private";
import { readFile } from "node:fs/promises";

async function read_secret(name: string): Promise<string | undefined> {
  if (env.IN_DOCKER) {
    return await readFile(`/run/secrets/${name}`, {encoding: 'utf8'});
  }

  return env[`SCAMPLERS_${name.toUpperCase()}`]
}

export const AUTH_SECRET = await read_secret('AUTH_SECRET');
export const MICROSOFT_ENTRA_ID_ID = await read_secret('MICROSOFT_ENTRA_ID_ID');
export const MICROSOFT_ENTRA_ID_SECRET = await read_secret('MICROSOFT_ENTRA_ID_SECRET');
