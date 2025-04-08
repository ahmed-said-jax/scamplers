<script lang="ts">
    import type { PageProps } from "./$types";

    let api_key = $state(null);

    let { data }: PageProps = $props();
    async function generate_api_key() {
        const response = await fetch("/api/api-key", {
            method: "POST",
            body: JSON.stringify({ id: data.user_id }),
            headers: { "Content-Type": "application/json" },
        });

        api_key = await response.json();
    }
</script>

<button onclick={generate_api_key}>Generate API Key</button>
{#if api_key}
    <div>API Key: {api_key}</div>
{/if}
