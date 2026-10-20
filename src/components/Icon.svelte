<script>
  /** @type {{ name: "docs" | "history" | "play" | "spinner", size?: number }} */
  let { name, size = 16 } = $props();
</script>

<svg
  class="icon-svg"
  class:spin={name === "spinner"}
  viewBox="0 0 24 24"
  width={size}
  height={size}
  aria-hidden="true"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
>
  {#if name === "docs"}
    <path d="M8 4h7l3 3v13a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z" />
    <path d="M14 4v4h4" />
    <path d="M9 13h6" />
    <path d="M9 17h6" />
  {:else if name === "history"}
    <circle cx="12" cy="12" r="9" />
    <path d="M12 7v5l3 2" />
  {:else if name === "play"}
    <path d="M8 5.5v13l11-6.5L8 5.5z" fill="currentColor" stroke="none" />
  {:else if name === "spinner"}
    <path d="M12 3a9 9 0 1 1-6.36 2.64" />
  {/if}
</svg>

<style>
  .icon-svg {
    display: block;
    flex-shrink: 0;
  }

  .spin {
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
