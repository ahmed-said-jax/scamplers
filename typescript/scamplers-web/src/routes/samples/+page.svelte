<script lang="ts">
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<h1>samples</h1>
<div>{data.specimens[0].name}</div>