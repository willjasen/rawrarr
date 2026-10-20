# Plan one-line subscriber node deployment

## Objective

Plan a curl-based command that downloads a script containing the user's generated Docker Compose definition, prepares Docker prerequisites, and starts that deployment on the destination node. Add the work to the roadmap without implementing or deploying it.

## Evidence and decisions

- Reviewed the current subscriber Compose generator, setup preview, deployment guide, device-linking plan, design guide, and roadmap.
- The current generator supports fossil and boneyard only, uses `rawrarr:local`, named data/shared volumes, loopback 7297, and a boneyard-only read-only media bind. It does not install Docker or bypass owner approval.
- Added the [deployment plan](../docs/one-line-node-deployment-plan.md), covering embedded Compose generation, restricted HTTPS delivery, local media-path handling, consent-based Docker setup, supported-platform gates, persistent reruns, failures, and release acceptance evidence.
- Linked the plan from Packaging and subscriptions on the roadmap, the documentation index, deployment guide, and design guide. All installer behavior remains explicitly planned.

## Scope and remaining work

Documentation only. No installer endpoint, script, application behavior, host package, container, runtime data, or secret was changed. No live Docker/bootstrap, enrollment, or platform-support verification was performed. Implementation must publish images and satisfy the plan's acceptance criteria before exposing a working install command.
