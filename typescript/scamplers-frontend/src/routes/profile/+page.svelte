<script lang="ts">
  let {data} = $props();
  let apiKey = $state("");

  async function toggleApiKey() {
    if (apiKey) {
      apiKey = "";
      return;
    }

    console.log(JSON.stringify(data));

    const response = await fetch("/auth/session");
    const session = await response.json();
    apiKey = session.user.apiKey;
  }

  const {name, email} = data.session.user;
</script>

<div>
    <ul>
        <li><strong>{name}</strong></li>
        <li>{email}</li>
    </ul>
    <button onclick={toggleApiKey}>{#if !apiKey}View API Key{:else}{apiKey} - Hide API Key{/if}</button>
</div>
