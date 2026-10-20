<script>
  import JSONTree from "svelte-json-tree";

  let { value = undefined, emptyText = "" } = $props();

  const hasValue = $derived(value !== undefined && value !== null);
</script>

{#if hasValue}
  <div class="tree-wrap">
    <JSONTree {value} defaultExpandedLevel={99} shouldShowPreview={false} />
  </div>
{:else}
  <p class="muted empty">{emptyText}</p>
{/if}

<style>
  .tree-wrap {
    flex: 1 1 auto;
    min-height: 180px;
    overflow: auto;
    border: 1px solid #243041;
    border-radius: 10px;
    background: linear-gradient(180deg, #0d131b 0%, #0b1016 100%);
    padding: 10px 12px 12px;
    color: #d7e0ea;

    --json-tree-font-family: "IBM Plex Mono", ui-monospace, SFMono-Regular, Menlo,
      Consolas, monospace;
    --json-tree-font-size: 12.5px;
    --json-tree-li-line-height: 1.45;
    /* Used by the library; keep in sync with the single indent source below. */
    --json-tree-li-indentation: 2ch;

    --json-tree-string-color: #9ece6a;
    --json-tree-symbol-color: #9ece6a;
    --json-tree-boolean-color: #7dcfff;
    --json-tree-function-color: #7aa2f7;
    --json-tree-number-color: #ff9e64;
    --json-tree-label-color: #7aa2f7;
    --json-tree-property-color: #c0caf5;
    --json-tree-arrow-color: #7b849c;
    --json-tree-operator-color: #565f89;
    --json-tree-null-color: #787c99;
    --json-tree-undefined-color: #787c99;
    --json-tree-date-color: #787c99;
    --json-tree-internal-color: #565f89;
    --json-tree-regex-color: #9ece6a;
  }

  /*
    Library applies BOTH nested-ul margin AND .indent padding using the same
    indentation variable. Zero one of them so each level is exactly 2ch.
  */
  .tree-wrap :global(ul) {
    margin-left: 0 !important;
  }

  .tree-wrap :global(ul ul) {
    margin-left: 2ch !important;
    padding-left: 0 !important;
    border-left: 1px solid #1e293b;
  }

  .tree-wrap :global(li) {
    padding: 0 !important;
    border-radius: 4px;
  }

  .tree-wrap :global(.indent) {
    padding-left: 0 !important;
  }

  .tree-wrap :global(.root),
  .tree-wrap :global(.label) {
    border-radius: 4px;
    padding: 0;
  }

  .tree-wrap :global(.root:hover),
  .tree-wrap :global(li > .label:hover) {
    background: rgba(61, 139, 253, 0.08);
  }

  .tree-wrap :global(.arrow) {
    display: inline-block !important;
    width: 1ch !important;
    margin-right: 0.5ch !important;
    font-size: 8px !important;
    line-height: 1 !important;
    vertical-align: middle;
    color: var(--json-tree-arrow-color) !important;
    opacity: 0.85;
  }

  .tree-wrap :global(.arrow.expanded) {
    color: #3d8bfd !important;
    opacity: 1;
  }

  .tree-wrap :global(.property) {
    font-weight: 500;
  }

  .tree-wrap :global(.operator) {
    margin: 0 0.1ch;
  }

  .muted {
    opacity: 0.68;
    font-size: 12px;
  }

  .empty {
    margin: 8px 0 0;
  }
</style>
