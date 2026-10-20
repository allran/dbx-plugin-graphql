<script>
  import { onMount } from "svelte";
  import DocsExplorer from "./components/DocsExplorer.svelte";
  import Icon from "./components/Icon.svelte";
  import QueryEditor from "./components/QueryEditor.svelte";
  import ResponseTree from "./components/ResponseTree.svelte";
  import { resolveMessages } from "./lib/i18n.js";
  import { loadDraft, loadHistory, saveDraft, saveHistory } from "./lib/storage.js";

  const DEFAULT_QUERY = `query {\n  __typename\n}\n`;

  let text = $state(resolveMessages("zh"));
  let connectionId = $state("");
  let endpointLabel = $state("");
  let bootError = $state("");
  /** @type {"docs" | "history" | null} */
  let drawer = $state(null);
  let responseTab = $state("body");
  let schema = $state(null);
  let schemaError = $state("");
  let history = $state([]);
  let pageSeq = $state(1);
  let toastMessage = $state("");
  let toastVisible = $state(false);
  /** @type {ReturnType<typeof setTimeout> | 0} */
  let toastTimer = 0;

  const initialPage = createPage(1);
  /** @type {{ id: string, title: string, query: string, response: any, running: boolean }[]} */
  let pages = $state([initialPage]);
  let activePageId = $state(initialPage.id);

  const activePage = $derived(
    pages.find((page) => page.id === activePageId) || pages[0] || null,
  );
  const activeRunning = $derived(Boolean(activePage?.running));

  const responseBodyValue = $derived.by(() => {
    const response = activePage?.response;
    if (!response) return undefined;
    if (response.error != null && response.body == null) {
      return { error: response.error };
    }
    return response.body ?? response;
  });
  const responseHeadersValue = $derived(
    activePage?.response?.headers ?? undefined,
  );

  function createPage(seq, query = DEFAULT_QUERY) {
    return {
      id: `page-${seq}-${Date.now()}`,
      title: `${seq}`,
      query,
      response: null,
      running: false,
    };
  }

  function tabTitle(page, index) {
    const n = page.title || String(index + 1);
    return `${text.pageLabel} ${n}`;
  }

  onMount(() => {
    window.dbxPlugin.ready
      .then(async () => {
        const locale = window.dbxPlugin.locale || "zh-CN";
        text = resolveMessages(locale);
        connectionId = window.dbxPlugin.context?.connectionId || "";
        endpointLabel =
          window.dbxPlugin.context?.name ||
          window.dbxPlugin.context?.endpoint ||
          window.dbxPlugin.context?.connectionName ||
          "";
        if (!connectionId) {
          bootError = text.noConnection;
          return;
        }

        const draft = await loadDraft(connectionId);
        if (Array.isArray(draft?.pages) && draft.pages.length > 0) {
          pages = draft.pages.map((page, index) => ({
            id: page.id || `page-${index + 1}`,
            title: page.title || String(index + 1),
            query: page.query || DEFAULT_QUERY,
            response: page.response ?? null,
            running: false,
          }));
          const maxTitle = pages.reduce((max, page) => {
            const n = Number.parseInt(String(page.title), 10);
            return Number.isFinite(n) ? Math.max(max, n) : max;
          }, pages.length);
          pageSeq = maxTitle;
          activePageId =
            draft.activePageId && pages.some((p) => p.id === draft.activePageId)
              ? draft.activePageId
              : pages[0].id;
        } else if (draft?.query) {
          pages = [createPage(1, draft.query)];
          activePageId = pages[0].id;
        }

        history = await loadHistory(connectionId);
        await reloadSchema();
      })
      .catch((error) => {
        bootError = `${text.bootError}: ${error.message || error}`;
      });
  });

  async function persistDraft() {
    if (!connectionId) return;
    await saveDraft(connectionId, {
      activePageId,
      pages: pages.map((page) => ({
        id: page.id,
        title: page.title,
        query: page.query,
        response: page.response,
      })),
    });
  }

  function updateActiveQuery(value) {
    pages = pages.map((page) =>
      page.id === activePageId ? { ...page, query: value } : page,
    );
  }

  function patchPage(pageId, patch) {
    pages = pages.map((page) =>
      page.id === pageId ? { ...page, ...patch } : page,
    );
  }

  function addPage() {
    pageSeq += 1;
    const page = createPage(pageSeq);
    pages = [...pages, page];
    activePageId = page.id;
    responseTab = "body";
    persistDraft();
  }

  function selectPage(id) {
    activePageId = id;
    responseTab = "body";
  }

  function closePage(id, event) {
    event?.stopPropagation();
    if (pages.length <= 1) return;
    const index = pages.findIndex((page) => page.id === id);
    const next = pages.filter((page) => page.id !== id);
    pages = next;
    if (activePageId === id) {
      const fallback = next[Math.max(0, index - 1)] || next[0];
      activePageId = fallback.id;
    }
    persistDraft();
  }

  function toggleDrawer(kind) {
    drawer = drawer === kind ? null : kind;
  }

  async function reloadSchema() {
    if (!connectionId) return;
    schemaError = "";
    try {
      const result = await window.dbxPlugin.invoke("graphql/introspect", {
        connectionId,
      });
      schema = result?.schema || null;
    } catch (error) {
      schema = null;
      schemaError = error.message || String(error);
    }
  }

  async function runQuery() {
    if (!connectionId || !activePage || activePage.running) return;

    const pageId = activePage.id;
    const queryText = activePage.query;
    patchPage(pageId, { running: true });

    try {
      const result = await window.dbxPlugin.invoke("graphql/execute", {
        connectionId,
        query: queryText,
      });
      patchPage(pageId, { response: result, running: false });
      if (activePageId === pageId) {
        responseTab = "body";
      }
      const item = {
        id: `${Date.now()}`,
        at: new Date().toISOString(),
        query: queryText,
        status: result?.status ?? 0,
        elapsedMs: result?.elapsedMs ?? 0,
      };
      history = [item, ...history].slice(0, 50);
      await saveHistory(connectionId, history);
      await persistDraft();
    } catch (error) {
      patchPage(pageId, {
        response: { error: error.message || String(error) },
        running: false,
      });
      if (activePageId === pageId) {
        responseTab = "body";
      }
    }
  }

  function applyHistory(item) {
    updateActiveQuery(item.query || DEFAULT_QUERY);
    drawer = "history";
    persistDraft();
  }

  function addQueryFromDocs(doc) {
    updateActiveQuery(doc);
    persistDraft();
  }

  function showToast(message) {
    toastMessage = message;
    toastVisible = true;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      toastVisible = false;
      toastTimer = 0;
    }, 1600);
  }

  async function clearHistory() {
    history = [];
    if (connectionId) await saveHistory(connectionId, []);
    showToast(text.cleared);
  }

  async function copyResponse() {
    const raw =
      responseTab === "headers" ? responseHeadersValue : responseBodyValue;
    if (raw === undefined) return;
    const value = JSON.stringify(raw, null, 2);
    try {
      if (window.dbxPlugin?.clipboard?.writeText) {
        await window.dbxPlugin.clipboard.writeText(value);
      } else if (window.dbxPlugin?.copy) {
        await window.dbxPlugin.copy(value);
      } else {
        await navigator.clipboard.writeText(value);
      }
      showToast(text.copied);
    } catch {
      showToast(text.copyFailed);
    }
  }
</script>

<div class="app">
  <header class="toolbar">
    <div class="toolbar-left">
      <strong class="brand-title">{text.title}</strong>
      {#if endpointLabel}
        <span class="muted endpoint">{endpointLabel}</span>
      {/if}
      <div class="page-tabs" role="tablist">
        {#each pages as page, index (page.id)}
          <button
            type="button"
            class="page-tab"
            class:active={page.id === activePageId}
            role="tab"
            aria-selected={page.id === activePageId}
            onclick={() => selectPage(page.id)}
          >
            <span class="page-tab-label">{tabTitle(page, index)}</span>
            {#if pages.length > 1}
              <span
                class="page-tab-close"
                role="button"
                tabindex="0"
                title={text.closePage}
                onclick={(event) => closePage(page.id, event)}
                onkeydown={(event) => {
                  if (event.key === "Enter" || event.key === " ") {
                    closePage(page.id, event);
                  }
                }}
              >
                ×
              </span>
            {/if}
          </button>
        {/each}
        <button type="button" class="add-new" onclick={addPage} title={text.newPage}>
          <span class="add-plus">+</span>
          <span class="add-label">{text.newPage}</span>
        </button>
      </div>
    </div>
    <div class="toolbar-right">
      <button
        type="button"
        class="ghost icon-btn"
        class:active={drawer === "docs"}
        title={text.docs}
        aria-label={text.docs}
        onclick={() => toggleDrawer("docs")}
      >
        <Icon name="docs" size={16} />
      </button>
      <button
        type="button"
        class="ghost icon-btn"
        class:active={drawer === "history"}
        title={text.history}
        aria-label={text.history}
        onclick={() => toggleDrawer("history")}
      >
        <Icon name="history" size={16} />
      </button>
    </div>
  </header>

  {#if toastVisible && toastMessage}
    <div class="toast" role="status">{toastMessage}</div>
  {/if}

  {#if bootError}
    <div class="banner">{bootError}</div>
  {:else}
    <div class="workspace" class:drawer-open={drawer != null}>
      <div class="main">
        <section class="editor-pane">
          <div class="pane-head">
            <span class="label">{text.query}</span>
            <div class="pane-actions">
              <button
                type="button"
                class="primary icon-btn"
                disabled={activeRunning || !connectionId}
                title={activeRunning ? text.running : text.run}
                aria-label={activeRunning ? text.running : text.run}
                onclick={runQuery}
              >
                <Icon name={activeRunning ? "spinner" : "play"} size={15} />
              </button>
            </div>
          </div>
          <QueryEditor
            value={activePage?.query || ""}
            {schema}
            onChange={updateActiveQuery}
            onBlur={persistDraft}
          />
        </section>

        <section class="response-pane">
          <div class="response-meta">
            <div class="meta-title">
              <span class="label">{text.response}</span>
              <button type="button" class="ghost tiny" onclick={copyResponse}
                >{text.copy}</button
              >
            </div>
            {#if activePage?.response}
              <div class="meta-chips">
                {#if activePage.response.elapsedMs != null}
                  <span class="chip"
                    >{text.elapsed} {activePage.response.elapsedMs}ms</span
                  >
                {/if}
                {#if activePage.response.error || activePage.response.httpOk === false}
                  <span class="chip fail">{text.httpFail}</span>
                {:else if activePage.response.httpOk === true}
                  <span class="chip ok">{text.httpOk}</span>
                {/if}
              </div>
            {:else}
              <p class="muted empty">{text.emptyResponse}</p>
            {/if}
          </div>

          <div class="tabs response-tabs">
            <button
              type="button"
              class:active={responseTab === "body"}
              onclick={() => (responseTab = "body")}>{text.responseBody}</button
            >
            <button
              type="button"
              class:active={responseTab === "headers"}
              onclick={() => (responseTab = "headers")}
              >{text.responseHeaders}</button
            >
          </div>

          {#if responseTab === "body"}
            <ResponseTree value={responseBodyValue} emptyText={text.emptyResponse} />
          {:else}
            <ResponseTree
              value={responseHeadersValue}
              emptyText={'{}'}
            />
          {/if}
        </section>
      </div>

      {#if drawer}
        <aside class="drawer">
          <div class="drawer-head">
            <strong>{drawer === "docs" ? text.docs : text.history}</strong>
            <div class="drawer-actions">
              {#if drawer === "docs"}
                <button type="button" class="ghost tiny" onclick={reloadSchema}
                  >{text.reloadSchema}</button
                >
              {:else}
                <button type="button" class="ghost tiny" onclick={clearHistory}
                  >{text.clearHistory}</button
                >
              {/if}
              <button
                type="button"
                class="ghost tiny"
                onclick={() => (drawer = null)}>{text.closePanel}</button
              >
            </div>
          </div>

          {#if drawer === "docs"}
            <DocsExplorer
              {schema}
              {schemaError}
              {text}
              onAddQuery={addQueryFromDocs}
            />
          {:else}
            {#if history.length === 0}
              <p class="muted">{text.emptyHistory}</p>
            {:else}
              <div class="history-list">
                {#each history as item (item.id)}
                  <button
                    type="button"
                    class="history-item"
                    title={text.historyHint}
                    onclick={() => applyHistory(item)}
                  >
                    <code>{item.query.trim().split("\n")[0]}</code>
                    <small class="muted"
                      >{item.status} · {item.elapsedMs}ms</small
                    >
                  </button>
                {/each}
              </div>
            {/if}
          {/if}
        </aside>
      {/if}
    </div>
  {/if}

</div>

<style>
  :global(html, body, #app) {
    margin: 0;
    height: 100%;
  }

  :global(body) {
    font-family: "IBM Plex Sans", "Noto Sans SC", ui-sans-serif, system-ui, sans-serif;
    background: #0f1419;
    color: #e7ecf3;
  }

  .app {
    height: 100%;
    display: grid;
    grid-template-rows: auto 1fr;
    min-height: 100vh;
  }

  .toolbar {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    align-items: center;
    padding: 8px 12px;
    border-bottom: 1px solid #243041;
    background: #121821;
  }

  .toolbar-left,
  .toolbar-right,
  .pane-actions,
  .drawer-actions,
  .drawer-head {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .toolbar-left {
    min-width: 0;
    flex: 1;
  }

  .toolbar-right {
    flex-shrink: 0;
  }

  .brand-title {
    font-size: 14px;
    flex-shrink: 0;
  }

  .endpoint {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 160px;
    flex-shrink: 1;
  }

  .page-tabs {
    display: flex;
    align-items: stretch;
    gap: 0;
    min-width: 0;
    overflow-x: auto;
    max-width: min(56vw, 640px);
    margin-left: 10px;
    padding: 2px;
    border-radius: 10px;
    background: #0b1016;
    border: 1px solid #1e293b;
    scrollbar-width: thin;
  }

  .page-tab {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    max-width: 140px;
    min-height: 30px;
    padding: 0 10px 0 12px;
    border-radius: 8px;
    border: 0;
    background: transparent;
    color: #94a3b8;
    font-size: 12px;
    font-weight: 500;
    letter-spacing: 0.01em;
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }

  .page-tab:hover {
    background: #151c26;
    color: #e2e8f0;
  }

  .page-tab.active {
    background: #1a2330;
    color: #f8fafc;
    box-shadow: inset 0 -2px 0 #3d8bfd;
  }

  .page-tab-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .page-tab-close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    border-radius: 999px;
    opacity: 0;
    font-size: 13px;
    line-height: 1;
    color: #cbd5e1;
    transition:
      opacity 0.12s ease,
      background 0.12s ease;
  }

  .page-tab:hover .page-tab-close,
  .page-tab.active .page-tab-close {
    opacity: 0.7;
  }

  .page-tab-close:hover {
    opacity: 1;
    background: #334155;
  }

  .add-new {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
    margin-left: 2px;
    padding: 0 10px;
    min-height: 30px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: #64748b;
    font-size: 12px;
    font-weight: 500;
    white-space: nowrap;
  }

  .add-new:hover {
    background: #151c26;
    color: #e2e8f0;
  }

  .add-plus {
    font-size: 14px;
    line-height: 1;
  }

  button {
    font: inherit;
    border: 0;
    border-radius: 8px;
    padding: 8px 12px;
    cursor: pointer;
    background: #1c2633;
    color: inherit;
  }

  button:disabled {
    opacity: 0.55;
    cursor: wait;
  }

  button.primary {
    background: #3d8bfd;
    color: #04101f;
    font-weight: 650;
  }

  button.ghost {
    background: transparent;
    border: 1px solid #334155;
  }

  button.ghost.active {
    background: #243447;
    border-color: #3d8bfd;
  }

  button.tiny {
    padding: 4px 8px;
    font-size: 12px;
  }

  button.icon-btn {
    width: 34px;
    height: 34px;
    padding: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .toast {
    position: fixed;
    left: 50%;
    bottom: 24px;
    transform: translateX(-50%);
    z-index: 1000;
    padding: 8px 14px;
    border-radius: 999px;
    background: #e7ecf3;
    color: #0f1419;
    font-size: 12px;
    font-weight: 600;
    pointer-events: none;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.28);
  }

  .banner {
    margin: 16px;
    padding: 14px 16px;
    border-radius: 10px;
    background: #3a1d1d;
    color: #ffb4b4;
  }

  .workspace {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    min-height: 0;
    height: 100%;
    position: relative;
  }

  .workspace.drawer-open {
    grid-template-columns: minmax(0, 1fr) minmax(280px, 360px);
  }

  .main {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    min-height: 0;
    min-width: 0;
  }

  .editor-pane,
  .response-pane,
  .drawer {
    min-height: 0;
    min-width: 0;
    overflow: auto;
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
  }

  .response-pane {
    border-left: 1px solid #243041;
    background: #0d1218;
  }

  .drawer {
    border-left: 1px solid #243041;
    background: #10161e;
    gap: 10px;
  }

  .drawer-head {
    justify-content: space-between;
  }

  .pane-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    margin-bottom: 8px;
  }

  .pane-head .label {
    margin-bottom: 0;
  }

  .label {
    display: block;
    font-size: 12px;
    opacity: 0.72;
    margin-bottom: 6px;
  }

  .response-meta {
    flex: 0 0 auto;
    margin-bottom: 8px;
  }

  .meta-title {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .meta-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 8px;
  }

  .chip {
    display: inline-flex;
    align-items: center;
    border-radius: 999px;
    padding: 3px 10px;
    font-size: 12px;
    border: 1px solid #334155;
    background: #182230;
  }

  .chip.ok {
    border-color: #1f6a45;
    background: #123225;
    color: #8dffc2;
  }

  .chip.fail {
    border-color: #8a2f2f;
    background: #331818;
    color: #ff9d9d;
  }

  .response-tabs,
  .tabs {
    display: flex;
    gap: 6px;
    margin-bottom: 8px;
  }

  .tabs button.active {
    background: #243447;
  }

  .history-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    overflow: auto;
    min-height: 0;
  }

  .history-item {
    text-align: left;
    display: flex;
    justify-content: space-between;
    gap: 8px;
    background: transparent;
    border: 1px solid transparent;
  }

  .history-item:hover {
    border-color: #334155;
    background: #182230;
  }

  .muted {
    opacity: 0.68;
    font-size: 12px;
  }

  .empty {
    margin: 8px 0 0;
  }

  @media (max-width: 900px) {
    .workspace.drawer-open {
      grid-template-columns: 1fr;
      grid-template-rows: minmax(0, 1fr) minmax(240px, 40vh);
    }

    .drawer {
      border-left: 0;
      border-top: 1px solid #243041;
    }

    .main {
      grid-template-columns: 1fr;
    }

    .response-pane {
      border-left: 0;
      border-top: 1px solid #243041;
    }
  }
</style>
