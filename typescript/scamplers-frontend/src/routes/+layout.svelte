<script lang="ts">
  import '../app.css';
  import { SignIn } from "@auth/sveltekit/components"

  let { data, children } = $props();
</script>

<div>
    {#if !data.session}
        <SignIn provider="microsoft-entra-id" signInPage="signin" />
    {:else}
        <div>Hello, {data.session.user.name}</div>
    {/if}

</div>

{@render children()}
