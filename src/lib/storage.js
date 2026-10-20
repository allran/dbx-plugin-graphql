const HISTORY_KEY = "graphql.history.v1";
const DRAFT_KEY = "graphql.draft.v2";

function storageAvailable() {
  return Boolean(window.dbxPlugin?.storage && window.dbxPlugin?.capabilities?.storage);
}

export async function loadHistory(connectionId) {
  if (!storageAvailable() || !connectionId) return [];
  try {
    const all = (await window.dbxPlugin.storage.get(HISTORY_KEY)) || {};
    const list = all[connectionId];
    return Array.isArray(list) ? list : [];
  } catch {
    return [];
  }
}

export async function saveHistory(connectionId, items) {
  if (!storageAvailable() || !connectionId) return;
  const all = (await window.dbxPlugin.storage.get(HISTORY_KEY)) || {};
  all[connectionId] = items.slice(0, 50);
  await window.dbxPlugin.storage.set(HISTORY_KEY, all);
}

export async function loadDraft(connectionId) {
  if (!storageAvailable() || !connectionId) return null;
  try {
    const all = (await window.dbxPlugin.storage.get(DRAFT_KEY)) || {};
    return all[connectionId] || null;
  } catch {
    return null;
  }
}

export async function saveDraft(connectionId, draft) {
  if (!storageAvailable() || !connectionId) return;
  const all = (await window.dbxPlugin.storage.get(DRAFT_KEY)) || {};
  all[connectionId] = draft;
  await window.dbxPlugin.storage.set(DRAFT_KEY, all);
}
