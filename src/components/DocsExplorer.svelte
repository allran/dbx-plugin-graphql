<script>
  import {
    buildRootFieldDocument,
    findType,
    formatTypeRef,
    namedType,
    rootTypeNames,
    searchSchema,
    sortFields,
  } from "../lib/schema.js";

  /**
   * @typedef {{ kind: "root" } | { kind: "type", name: string } | { kind: "field", typeName: string, fieldName: string }} DocsFrame
   */

  let {
    schema = null,
    schemaError = "",
    text,
    onAddQuery = (_doc) => {},
  } = $props();

  /** @type {DocsFrame[]} */
  let stack = $state([{ kind: "root" }]);
  let schemaQuery = $state("");
  let sortMode = $state("alpha");

  const roots = $derived(rootTypeNames(schema));
  const current = $derived(stack[stack.length - 1] || { kind: "root" });
  const searchResults = $derived(searchSchema(schema, schemaQuery));
  const searching = $derived(schemaQuery.trim().length > 0);

  const currentType = $derived.by(() => {
    if (!schema) return null;
    if (current.kind === "type") return findType(schema, current.name);
    if (current.kind === "field") return findType(schema, current.typeName);
    return null;
  });

  const currentField = $derived.by(() => {
    if (current.kind !== "field" || !currentType) return null;
    const fields = [
      ...(currentType.fields || []),
      ...(currentType.inputFields || []),
    ];
    return fields.find((field) => field.name === current.fieldName) || null;
  });

  const breadcrumb = $derived.by(() => {
    const items = [{ label: text.docsRoot, index: 0 }];
    for (let i = 1; i < stack.length; i += 1) {
      const frame = stack[i];
      if (frame.kind === "type") {
        items.push({ label: frame.name, index: i });
      } else if (frame.kind === "field") {
        items.push({ label: frame.fieldName, index: i });
      }
    }
    return items;
  });

  const rootEntries = $derived.by(() => {
    const entries = [];
    if (roots.query) {
      entries.push({
        role: "query",
        label: "query",
        typeName: roots.query,
      });
    }
    if (roots.mutation) {
      entries.push({
        role: "mutation",
        label: "mutation",
        typeName: roots.mutation,
      });
    }
    if (roots.subscription) {
      entries.push({
        role: "subscription",
        label: "subscription",
        typeName: roots.subscription,
      });
    }
    return entries;
  });

  const displayedFields = $derived.by(() => {
    if (!currentType?.fields) return [];
    return sortFields(currentType.fields, sortMode);
  });

  const displayedInputFields = $derived.by(() => {
    if (!currentType?.inputFields) return [];
    return sortFields(currentType.inputFields, sortMode);
  });

  let lastSchema = $state(null);
  $effect(() => {
    if (schema !== lastSchema) {
      lastSchema = schema;
      stack = [{ kind: "root" }];
      schemaQuery = "";
    }
  });

  function goToIndex(index) {
    stack = stack.slice(0, index + 1);
  }

  function openType(name) {
    if (!name) return;
    stack = [...stack, { kind: "type", name }];
  }

  function openField(typeName, fieldName) {
    stack = [...stack, { kind: "field", typeName, fieldName }];
  }

  function openNamedFromRef(typeRef) {
    const leaf = namedType(typeRef);
    if (leaf?.name) openType(leaf.name);
  }

  function openSearchResult(item) {
    schemaQuery = "";
    if (item.kind === "type") {
      stack = [{ kind: "root" }, { kind: "type", name: item.typeName }];
      return;
    }
    stack = [
      { kind: "root" },
      { kind: "type", name: item.typeName },
      { kind: "field", typeName: item.typeName, fieldName: item.fieldName },
    ];
  }

  function addRootField(role, field) {
    const doc = buildRootFieldDocument(role, field);
    if (doc) onAddQuery(doc);
  }

  function isRootType(typeName) {
    return (
      typeName === roots.query ||
      typeName === roots.mutation ||
      typeName === roots.subscription
    );
  }

  function rootRoleForType(typeName) {
    if (typeName === roots.query) return "query";
    if (typeName === roots.mutation) return "mutation";
    if (typeName === roots.subscription) return "subscription";
    return "query";
  }
</script>

<div class="docs">
  <div class="docs-toolbar">
    <input
      class="search"
      placeholder={text.searchSchema}
      bind:value={schemaQuery}
    />
    <button
      type="button"
      class="ghost tiny"
      class:active={sortMode === "alpha"}
      onclick={() => (sortMode = sortMode === "alpha" ? "schema" : "alpha")}
      title={text.sortFields}
    >
      {sortMode === "alpha" ? "A↓" : "#"}
    </button>
  </div>

  {#if schemaError}
    <p class="error">{schemaError}</p>
  {:else if !schema}
    <p class="muted">{text.emptyDocs}</p>
  {:else if searching}
    <div class="section-label">{text.searchResults}</div>
    {#if searchResults.length === 0}
      <p class="muted">{text.noSearchResults}</p>
    {:else}
      <div class="list">
        {#each searchResults as item (item.label + item.detail)}
          <button type="button" class="row" onclick={() => openSearchResult(item)}>
            <span class="row-main">
              <span class="name">{item.label}</span>
              <span class="type-link">{item.detail}</span>
            </span>
            <small class="muted">{item.kind}</small>
          </button>
        {/each}
      </div>
    {/if}
  {:else}
    <nav class="crumbs" aria-label="docs breadcrumb">
      {#each breadcrumb as crumb, i}
        {#if i > 0}<span class="sep">/</span>{/if}
        <button
          type="button"
          class="crumb"
          class:current={i === breadcrumb.length - 1}
          onclick={() => goToIndex(crumb.index)}
        >
          {crumb.label}
        </button>
      {/each}
    </nav>

    {#if current.kind === "root"}
      <div class="section-label">{text.rootTypes}</div>
      <div class="list">
        {#each rootEntries as entry (entry.role)}
          <button type="button" class="row root-row" onclick={() => openType(entry.typeName)}>
            <span class="row-main">
              <span class="name root-name">{entry.label}</span>
              <span class="colon">:</span>
              <span class="type-link">{entry.typeName}</span>
            </span>
          </button>
        {/each}
      </div>
      {#if rootEntries.length === 0}
        <p class="muted">{text.emptyDocs}</p>
      {/if}
    {:else if current.kind === "type" && currentType}
      <header class="type-head">
        <div class="type-title">
          <h3>{currentType.name}</h3>
          <span class="kind-badge">{currentType.kind}</span>
        </div>
        {#if currentType.description}
          <p class="desc">{currentType.description}</p>
        {/if}
      </header>

      {#if displayedFields.length}
        <div class="section-label">{text.fields}</div>
        <div class="list">
          {#each displayedFields as field (field.name)}
            <div class="field-block">
              <div class="row field-row">
                <button
                  type="button"
                  class="name-btn"
                  onclick={() => openField(currentType.name, field.name)}
                >
                  <span class="name" class:deprecated={field.isDeprecated}>{field.name}</span>
                  {#if field.args?.length}
                    <span class="args-hint">({field.args.length})</span>
                  {/if}
                </button>
                <span class="colon">:</span>
                <button
                  type="button"
                  class="type-link"
                  onclick={() => openNamedFromRef(field.type)}
                >
                  {formatTypeRef(field.type)}
                </button>
                {#if isRootType(currentType.name)}
                  <button
                    type="button"
                    class="ghost tiny add"
                    title={text.addToQuery}
                    onclick={() =>
                      addRootField(rootRoleForType(currentType.name), field)}
                  >
                    +
                  </button>
                {/if}
              </div>
              {#if field.description}
                <p class="field-desc">{field.description}</p>
              {/if}
              {#if field.isDeprecated}
                <p class="deprecated-note">
                  {text.deprecated}{field.deprecationReason
                    ? `: ${field.deprecationReason}`
                    : ""}
                </p>
              {/if}
            </div>
          {/each}
        </div>
      {/if}

      {#if displayedInputFields.length}
        <div class="section-label">{text.inputFields}</div>
        <div class="list">
          {#each displayedInputFields as field (field.name)}
            <button
              type="button"
              class="row"
              onclick={() => openField(currentType.name, field.name)}
            >
              <span class="row-main">
                <span class="name">{field.name}</span>
                <span class="colon">:</span>
                <span class="type-link">{formatTypeRef(field.type)}</span>
              </span>
            </button>
          {/each}
        </div>
      {/if}

      {#if currentType.enumValues?.length}
        <div class="section-label">{text.enumValues}</div>
        <div class="list">
          {#each sortFields(currentType.enumValues, sortMode) as value (value.name)}
            <div class="row static">
              <span class="name" class:deprecated={value.isDeprecated}>{value.name}</span>
              {#if value.description}
                <small class="muted">{value.description}</small>
              {/if}
            </div>
          {/each}
        </div>
      {/if}

      {#if currentType.possibleTypes?.length}
        <div class="section-label">{text.possibleTypes}</div>
        <div class="list">
          {#each currentType.possibleTypes as item}
            <button type="button" class="row" onclick={() => openNamedFromRef(item)}>
              <span class="type-link">{formatTypeRef(item)}</span>
            </button>
          {/each}
        </div>
      {/if}

      {#if currentType.interfaces?.length}
        <div class="section-label">{text.interfaces}</div>
        <div class="list">
          {#each currentType.interfaces as item}
            <button type="button" class="row" onclick={() => openNamedFromRef(item)}>
              <span class="type-link">{formatTypeRef(item)}</span>
            </button>
          {/each}
        </div>
      {/if}
    {:else if current.kind === "field" && currentField}
      <header class="type-head">
        <div class="type-title">
          <h3>{currentField.name}</h3>
        </div>
        {#if currentField.description}
          <p class="desc">{currentField.description}</p>
        {/if}
        <div class="type-line">
          <span class="muted">{text.returns}</span>
          <button
            type="button"
            class="type-link"
            onclick={() => openNamedFromRef(currentField.type)}
          >
            {formatTypeRef(currentField.type)}
          </button>
        </div>
        {#if currentField.isDeprecated}
          <p class="deprecated-note">
            {text.deprecated}{currentField.deprecationReason
              ? `: ${currentField.deprecationReason}`
              : ""}
          </p>
        {/if}
        {#if isRootType(current.typeName)}
          <button
            type="button"
            class="ghost tiny add-block"
            onclick={() =>
              addRootField(rootRoleForType(current.typeName), currentField)}
          >
            {text.addToQuery}
          </button>
        {/if}
      </header>

      {#if currentField.args?.length}
        <div class="section-label">{text.arguments}</div>
        <div class="list">
          {#each sortFields(currentField.args, sortMode) as arg (arg.name)}
            <div class="row static">
              <span class="row-main">
                <span class="name">{arg.name}</span>
                <span class="colon">:</span>
                <button
                  type="button"
                  class="type-link"
                  onclick={() => openNamedFromRef(arg.type)}
                >
                  {formatTypeRef(arg.type)}
                </button>
              </span>
              {#if arg.defaultValue != null}
                <small class="muted">= {arg.defaultValue}</small>
              {/if}
            </div>
            {#if arg.description}
              <p class="field-desc">{arg.description}</p>
            {/if}
          {/each}
        </div>
      {:else}
        <p class="muted">{text.noArguments}</p>
      {/if}
    {:else}
      <p class="muted">{text.emptyDocs}</p>
    {/if}
  {/if}
</div>

<style>
  .docs {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-height: 0;
    flex: 1;
  }

  .docs-toolbar {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .search {
    flex: 1;
    width: 100%;
    box-sizing: border-box;
    border: 1px solid #2a3648;
    border-radius: 8px;
    background: #0b1016;
    color: inherit;
    padding: 8px 10px;
  }

  button.ghost {
    background: transparent;
    border: 1px solid #334155;
    border-radius: 8px;
    color: inherit;
    cursor: pointer;
    font: inherit;
  }

  button.tiny {
    padding: 4px 8px;
    font-size: 12px;
  }

  button.ghost.active {
    background: #243447;
  }

  .crumbs {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px;
    font-size: 12px;
  }

  .crumb {
    border: 0;
    background: transparent;
    color: #8eb6ff;
    cursor: pointer;
    padding: 0;
    font: inherit;
  }

  .crumb.current {
    color: #e7ecf3;
    cursor: default;
  }

  .sep {
    opacity: 0.45;
  }

  .section-label {
    margin-top: 6px;
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    opacity: 0.55;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow: auto;
    min-height: 0;
  }

  .row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 8px;
    text-align: left;
    border: 1px solid transparent;
    border-radius: 8px;
    background: transparent;
    color: inherit;
    padding: 6px 8px;
    font: inherit;
    cursor: pointer;
  }

  .row:not(.static):hover,
  .field-row:hover {
    border-color: #334155;
    background: #182230;
  }

  .row.static {
    cursor: default;
  }

  .row-main {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 4px;
    min-width: 0;
  }

  .field-block {
    display: flex;
    flex-direction: column;
  }

  .name-btn {
    display: inline-flex;
    align-items: baseline;
    gap: 4px;
    border: 0;
    background: transparent;
    color: inherit;
    font: inherit;
    padding: 0;
    cursor: pointer;
  }

  .name {
    font-family: "IBM Plex Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 13px;
  }

  .root-name {
    color: #c4b5fd;
  }

  .name.deprecated {
    text-decoration: line-through;
    opacity: 0.65;
  }

  .colon {
    opacity: 0.45;
  }

  .args-hint {
    opacity: 0.5;
    font-size: 11px;
  }

  .type-link {
    border: 0;
    background: transparent;
    color: #f0b429;
    cursor: pointer;
    padding: 0;
    font: inherit;
    font-family: "IBM Plex Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 12px;
  }

  .type-head h3 {
    margin: 0;
    font-size: 15px;
  }

  .type-title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .kind-badge {
    font-size: 10px;
    border: 1px solid #334155;
    border-radius: 999px;
    padding: 2px 8px;
    opacity: 0.8;
  }

  .desc,
  .field-desc {
    margin: 6px 0 0;
    font-size: 12px;
    opacity: 0.72;
    line-height: 1.45;
  }

  .field-desc {
    margin: 0 8px 6px;
  }

  .type-line {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin-top: 8px;
  }

  .deprecated-note {
    margin: 4px 8px 8px;
    color: #ff9d9d;
    font-size: 12px;
  }

  .add {
    flex-shrink: 0;
  }

  .add-block {
    margin-top: 10px;
    align-self: flex-start;
  }

  .muted {
    opacity: 0.68;
    font-size: 12px;
  }

  .error {
    color: #ff8e8e;
    font-size: 13px;
  }
</style>
