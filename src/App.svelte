<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let name = $state("");
  let greetMsg = $state("");

  async function greet() {
    greetMsg = await invoke("greet", { name });
  }
</script>

<main class="container">
  <h1>Welcome to Claude Prompt Editor</h1>

  <div class="row">
    <form onsubmit={(e: Event) => { e.preventDefault(); greet(); }}>
      <input id="greet-input" placeholder="Enter a name..." bind:value={name} />
      <button type="submit">Greet</button>
    </form>
  </div>

  <p>{greetMsg}</p>
</main>
