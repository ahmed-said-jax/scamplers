<script lang="ts">
	import '../app.css';
	import type { LayoutProps } from './$types';
	import { onMount } from 'svelte';

	let { children }: LayoutProps = $props();

	let me: {name: string} | null = $state(null);

	onMount(async () => {
		const response = await fetch('/frontend/api/me', { method: 'GET' });
		me = await response.json();
	});
</script>

<h1>Scamplers</h1>

{#if me}
	<nav>
		<a href="/">Home</a>
		<a href="/profile">Profile</a>
	</nav>
	<p>Hello {me.name}</p>
{/if}

{@render children()}
