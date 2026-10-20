<script>
  import { onDestroy, onMount } from "svelte";
  import { basicSetup, EditorView } from "codemirror";
  import { oneDark } from "@codemirror/theme-one-dark";
  import { graphql, updateSchema } from "cm6-graphql";
  import { schemaFromIntrospection } from "../lib/graphqlSchema.js";

  let {
    value = "",
    schema = null,
    onChange = (_next) => {},
    onBlur = () => {},
  } = $props();

  let host = $state(null);
  /** @type {EditorView | null} */
  let view = $state(null);
  let applyingExternal = false;

  onMount(() => {
    const gqlSchema = schemaFromIntrospection(schema);
    view = new EditorView({
      doc: value || "",
      parent: host,
      extensions: [
        basicSetup,
        oneDark,
        graphql(gqlSchema),
        EditorView.lineWrapping,
        EditorView.updateListener.of((update) => {
          if (!update.docChanged || applyingExternal) return;
          onChange(update.state.doc.toString());
        }),
        EditorView.domEventHandlers({
          blur() {
            onBlur();
          },
        }),
        EditorView.theme({
          "&": {
            height: "100%",
            fontSize: "13px",
            border: "1px solid #2a3648",
            borderRadius: "10px",
            overflow: "hidden",
          },
          "&.cm-focused": {
            outline: "1px solid #3d8bfd",
          },
          ".cm-scroller": {
            fontFamily:
              '"IBM Plex Mono", ui-monospace, SFMono-Regular, Menlo, monospace',
            lineHeight: "1.45",
          },
          ".cm-content": {
            padding: "10px 0",
          },
          ".cm-gutters": {
            background: "#0b1016",
            borderRight: "1px solid #243041",
            color: "#64748b",
          },
          ".cm-activeLineGutter": {
            background: "#151c26",
          },
        }),
      ],
    });
  });

  $effect(() => {
    if (!view) return;
    const next = value ?? "";
    const current = view.state.doc.toString();
    if (next === current) return;
    applyingExternal = true;
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: next },
    });
    applyingExternal = false;
  });

  $effect(() => {
    if (!view) return;
    updateSchema(view, schemaFromIntrospection(schema));
  });

  onDestroy(() => {
    view?.destroy();
    view = null;
  });
</script>

<div class="editor-host" bind:this={host}></div>

<style>
  .editor-host {
    flex: 1 1 auto;
    min-height: 180px;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .editor-host :global(.cm-editor) {
    flex: 1;
    width: 100%;
  }
</style>
