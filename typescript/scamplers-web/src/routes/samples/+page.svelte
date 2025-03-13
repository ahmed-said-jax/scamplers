<script lang="ts">
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<h1>samples</h1>
{#each data.specimens as {name, measurements, type, lab}}
	<div>
		{`${name}: ${type}`}
		<a href="{lab.link}">{lab.name}</a>
		{#if measurements != null}
			{#each measurements as m}
				{m.quantity}
				{m.value}
				{m.measured_at}
				{m.measured_by.name}
			{/each}
		{/if}
	</div>
{/each}