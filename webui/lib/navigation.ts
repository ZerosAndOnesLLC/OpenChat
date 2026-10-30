/**
 * Full document load of the app root, replacing the current history entry.
 *
 * Used where client-side routing is the wrong tool: after the session has been
 * cleared (stale in-memory state must be dropped and `/` restarts the auth
 * flow) and from the error boundary (the React tree itself may be broken).
 */
export function reloadAppRoot(): void {
  window.location.replace('/');
}
