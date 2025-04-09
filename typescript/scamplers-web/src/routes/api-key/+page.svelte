<script lang="ts">
    let api_key: string | null = $state(null);

    async function generate_api_key() {
        const response = await fetch("/web/api/api-key", {
            method: "POST",
        });

        let result = await response.json();
        api_key = result.api_key;
    }
</script>

<button onclick={generate_api_key}>Generate API Key</button>
{#if api_key}
    <div>API Key: {api_key}</div>
{/if}
