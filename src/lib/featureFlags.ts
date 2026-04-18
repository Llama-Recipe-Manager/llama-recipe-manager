/**
 * Feature flags for in-progress work. Flip to `true` to surface the related
 * UI; the underlying code (components, types, routes, plan doc) is kept
 * intact regardless so flipping is a one-line change.
 *
 * Currently disabled while we focus on the local-only experience. See
 * `docs/PLAN.md` for the rollout plan.
 */
export const FEATURES = {
  /** Community recipes browse / fork / validate section. */
  community: false,
  /** Account / profile / sign-in affordance in the nav rail. */
  account: false,
} as const;
