# Record AI chat fallback and tool support direction

## Objective and initial state

Record the user's report of an unhelpful chat response and decide whether Rawrarr needs a separate MCP server for useful AI tools. At the start, the dedicated `codex/ai-request-inbox` worktree was clean at `d8f8ab2`; the shared `main` checkout was separate. The current implementation is documented in [the chat and voice implementation runbook](2026-10-18-improve-ai-chat-voice-tabs.md) and [the original request-review plan](2026-10-18-plan-ai-chat-request-review.md).

## Observed response

The user asked that this response be kept as a product issue:

> I will check the available library records for this question.
>
> I saved that question for review. I can search your movies by year or genre right now. What would you like to try?

The first sentence is a streamed OpenAI introduction. The second is Rawrarr's local fallback for an intent it does not support. The chat can classify the current narrow set of requests, but it has no general tool loop for an agent to ask Rawrarr to perform a supported operation.

## Direction recorded

- Do not expect a model change alone to fix this behavior. Add an application-owned tool registry using Responses API function calling so the model can select from named, JSON-schema-bounded Rawrarr functions and roarlay can validate and execute each call.
- Start with read-only movie search and library statistics. Keep decrypted catalog filtering and result display in the browser. Do not send movie records to the model by default; if a future response requires cloud reasoning over private results, disclose and authorize the specific fields first.
- Keep configuration changes, downloads, and sharing edits behind a proposal and explicit owner approval. Never grant direct SQL, shell, or unrestricted configuration access to the model.
- Rawrarr does not need an MCP server for its own first-party chat. Once the same scoped operations are stable, an MCP adapter can make a deliberately limited subset available to a separate owner-run agent. It must reuse Rawrarr authorization and operation handlers rather than expose PostgreSQL directly.
- Revisit the model when implementing tools. OpenAI's current model documentation lists GPT-4o mini as supporting function calling but marks it deprecated.

## References and verification

- OpenAI documents function calling as a multi-step flow where the application receives tool calls, executes its own code, and returns tool outputs: [Function calling](https://developers.openai.com/api/docs/guides/function-calling).
- OpenAI describes MCP as a connection to an external tool server and documents remote server reachability and approval controls: [MCP servers](https://developers.openai.com/api/docs/guides/tools-connectors-mcp).
- OpenAI's model page records GPT-4o mini's function-calling support and deprecated status: [GPT-4o mini](https://developers.openai.com/api/docs/models/gpt-4o-mini).
- Updated `docs/DESIGN.md`, `ROADMAP.md`, and this runbook index. This is design documentation only; no model, endpoint, database, or deployment was changed.
- `git diff --check` passed. No application test suite was run for this documentation-only task.

## Remaining work

Implement the scoped function tool loop, audit each requested and executed tool call in the PostgreSQL inbox, and ensure unsupported requests produce a useful clarification or honest explanation instead of the recorded dead-end response. Consider MCP only after the operation and authorization layer has a stable contract.
