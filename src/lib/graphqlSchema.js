import { buildClientSchema } from "graphql";

/**
 * Convert introspection `__schema` JSON (from our Sidecar) into a
 * graphql-js GraphQLSchema for cm6-graphql autocomplete / lint.
 */
export function schemaFromIntrospection(introspectionSchema) {
  if (!introspectionSchema || typeof introspectionSchema !== "object") {
    return undefined;
  }
  try {
    return buildClientSchema({ __schema: introspectionSchema });
  } catch {
    return undefined;
  }
}
