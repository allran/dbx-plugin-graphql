export function flattenTypes(schema) {
  const types = Array.isArray(schema?.types) ? schema.types : [];
  return types
    .filter((type) => type?.name && !String(type.name).startsWith("__"))
    .sort((a, b) => String(a.name).localeCompare(String(b.name)));
}

export function rootTypeNames(schema) {
  return {
    query: schema?.queryType?.name || null,
    mutation: schema?.mutationType?.name || null,
    subscription: schema?.subscriptionType?.name || null,
  };
}

export function findType(schema, name) {
  if (!schema || !name) return null;
  const types = Array.isArray(schema.types) ? schema.types : [];
  return types.find((type) => type?.name === name) || null;
}

/** Unwrap NON_NULL / LIST wrappers to the named type. */
export function namedType(type) {
  let current = type;
  while (current && (current.kind === "NON_NULL" || current.kind === "LIST")) {
    current = current.ofType;
  }
  return current;
}

export function formatTypeRef(type) {
  if (!type) return "";
  if (type.kind === "NON_NULL") return `${formatTypeRef(type.ofType)}!`;
  if (type.kind === "LIST") return `[${formatTypeRef(type.ofType)}]`;
  return type.name || "";
}

export function isRequiredType(type) {
  return type?.kind === "NON_NULL";
}

/**
 * Search types and fields like Altair docs search.
 * @returns {{ kind: "type"|"field", typeName: string, fieldName?: string, label: string, detail: string }[]}
 */
export function searchSchema(schema, needle) {
  const q = String(needle || "").trim().toLowerCase();
  if (!q || !schema) return [];

  const results = [];
  for (const type of flattenTypes(schema)) {
    const typeHit = String(type.name).toLowerCase().includes(q);
    const descHit = String(type.description || "")
      .toLowerCase()
      .includes(q);
    if (typeHit || descHit) {
      results.push({
        kind: "type",
        typeName: type.name,
        label: type.name,
        detail: type.kind,
      });
    }

    const fields = [
      ...(type.fields || []),
      ...(type.inputFields || []),
    ];
    for (const field of fields) {
      const name = String(field.name || "");
      if (
        name.toLowerCase().includes(q) ||
        String(field.description || "")
          .toLowerCase()
          .includes(q)
      ) {
        results.push({
          kind: "field",
          typeName: type.name,
          fieldName: name,
          label: `${type.name}.${name}`,
          detail: formatTypeRef(field.type),
        });
      }
    }
  }
  return results.slice(0, 80);
}

export function sortFields(fields, mode = "alpha") {
  const list = Array.isArray(fields) ? [...fields] : [];
  if (mode === "alpha") {
    list.sort((a, b) => String(a.name).localeCompare(String(b.name)));
  }
  return list;
}

/** Build a minimal operation document for a root field (Altair “add from docs”). */
export function buildRootFieldDocument(operation, field) {
  const op = operation || "query";
  const name = field?.name;
  if (!name) return "";

  const leaf = namedType(field.type);
  const needsSelection =
    leaf &&
    (leaf.kind === "OBJECT" ||
      leaf.kind === "INTERFACE" ||
      leaf.kind === "UNION");

  // No Variables panel: emit the field call without $variables; user edits args inline.
  if (needsSelection) {
    return `${op} {\n  ${name} {\n    __typename\n  }\n}\n`;
  }
  return `${op} {\n  ${name}\n}\n`;
}

export function safeParseJson(text, fallback) {
  const trimmed = String(text || "").trim();
  if (!trimmed) return { ok: true, value: fallback };
  try {
    return { ok: true, value: JSON.parse(trimmed) };
  } catch (error) {
    return { ok: false, error: error.message };
  }
}
