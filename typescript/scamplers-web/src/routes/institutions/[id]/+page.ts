import type {PageLoad} from '../../$types'
import type {Institution} from '../../../lib/types/institution';

export const load: PageLoad = async ({fetch, params}) => {
    const res = await fetch(`/api/institutions/${params.id}`);
    const institution: Institution =  await res.json();

    return { institution }
}
