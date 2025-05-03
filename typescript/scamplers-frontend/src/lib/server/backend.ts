import { env } from '$env/dynamic/private';
import { Client } from 'scamplers-core';
import { AUTH_SECRET } from './secrets';

const BACKEND_HOST = env.SCAMPLERS_BACKEND_HOST ?? env.BACKEND_HOST;
const BACKEND_PORT = env.SCAMPLERS_BACKEND_PORT ?? env.BACKEND_PORT;

export const BACKEND_URL = `http://${BACKEND_HOST}:${BACKEND_PORT}`;

export const scamplersClient = new Client(BACKEND_URL, 'scamplers-frontend', AUTH_SECRET ?? '');
