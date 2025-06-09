<script lang="ts">
	import '../app.css';
	import { signOut } from '@auth/sveltekit/client';

	let { data, children } = $props();
	const { session } = data;
</script>

<div>
	{#if session}
		<div>Signed in as {session.user.name}</div>
		<button
			onclick={() => {
				signOut();
			}}>Sign Out</button
		>
	{/if}
	<nav>
		<ul>
			<li>
				<a href="/profile">Profile</a>
			</li>
		</ul>
	</nav>
</div>

{@render children()}
