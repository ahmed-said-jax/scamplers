<script lang="ts">
    import "../app.css";

    let { children } = $props();
</script>

<nav>
    <a href="/">Home</a>
    <a href="/api-key">API Key</a>
</nav>

{@render children()}
