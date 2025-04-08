<script lang="ts">
    let api_key: string | null = $state(null);

    async function generate_api_key() {
        const response = await fetch("/api/api-key", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
        });

        api_key = await response.text();
    }
</script>

<button onclick={generate_api_key}>Generate API Key</button>
{#if api_key}
    <div>API Key: {api_key}</div>
{/if}
