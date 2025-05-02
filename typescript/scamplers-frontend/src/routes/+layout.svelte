<script lang="ts">
	import '../app.css';

	let { data, children } = $props();
	const { session } = data;
</script>

<div>
	{#if session}
		<div>Hello, {session.user.name}</div>
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
