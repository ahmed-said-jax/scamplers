import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import type { Specimen } from '../../lib/bindings/specimen'
import {page} from '$app/state'

export const load: PageLoad = async ({ fetch, url }) => {
	const res = await fetch('/api/samples?' + url.searchParams);

	const specimens: Specimen[] = await res.json();

	return {specimens}
};