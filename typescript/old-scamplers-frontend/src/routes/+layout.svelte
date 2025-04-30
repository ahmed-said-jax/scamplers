<script lang="ts">
	import '../app.css';

	let { data, children } = $props();
</script>

<div>
	<div>Hello, {data.session.user.name}</div>
	<nav>
		<ul>
			<li>
				<a href="/profile">Profile</a>
			</li>
		</ul>
	</nav>
</div>

{@render children()}
