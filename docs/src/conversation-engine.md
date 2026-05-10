# Conversation Engine

The conversation engine lives in `src/agents/` and is driven from `Arpsci::run_graph()`.

## Run setup

Starting a run now does more than just spawn a pair loop.

`run_graph()` first stops any existing run, bumps the stop epoch, clears all sidecar and chat state, and marks the graph as running. It then:

1. creates a fresh agent-to-chat channel,
2. builds and persists a `RunManifest`,
3. opens `events.jsonl` for the active run and writes `system.run_started`,
4. resolves eligible worker nodes from the current graph,
5. builds evaluator/researcher sidecar config from the node graph,
6. groups workers into conversations and spawns one async loop per group.

Grouping behavior is driven by `ARPSCI_CONVERSATION_GROUP_SIZE`.

- When the value is `2`, explicit `partner_worker` links are preserved where possible and the remaining workers are paired by row order.
- When the value is `3` or higher, workers are chunked by sorted row order into larger conversation groups.

## Turn loop

Each spawned loop in `agent_conversation_loop.rs` manages one conversation session. The loop:

1. checks the cooperative stop flag,
2. rotates the current speaker through the participant list,
3. resolves the effective topic for the turn,
4. optionally runs researcher sidecars before the main turn,
5. builds a bounded memory block from dialogue history,
6. assembles the system prompt and user prompt,
7. records turn timing,
8. calls Ollama with streaming enabled,
9. writes ledger events and forwards the result to the Overview chat bridge,
10. optionally runs evaluator sidecars after the message.

The loop writes a `dialogue.start` event at the beginning and `dialogue.turn` events for each completed reply. Conversation start and turn messages can also be mirrored externally over HTTP and internally into the Overview chat room, depending on runtime flags.

## Prompt memory strategy

`DialogueSessionState` keeps three pieces of memory:

- `per_agent_last` for targeted grounding,
- `recent_exchanges` as a bounded deque,
- a rolling summary for messages that fall out of the recent window.

It also tracks recent token usage so the prompt can include a lightweight "last/average total" token budget line. The history window is controlled from the Settings UI through `conversation_history_size`, which defaults to `5`.

## Sidecars

Sidecars are first-class parts of the conversation engine now.

- Researcher sidecars run before a worker turn and inject references into that worker's prompt.
- Evaluator sidecars run after dialogue messages and can execute every turn or on a batch cadence.

Scheduling is controlled by environment variables:

- `ARPSCI_RESEARCH_POLICY`: `off`, `inline`, or `background`
- `ARPSCI_EVALUATOR_POLICY`: `off`, `inline`, or `batched:N`

Research sidecars are targeted to specific workers through the graph wiring, and evaluator/researcher HTTP posts go through the same outbound HTTP policy as conversation streaming.

## Stop semantics

Stopping is cooperative, not preemptive.

- `stop_graph()` clears each loop's active flag.
- `stop_graph()` also increments `ollama_run_epoch`.
- live Ollama streaming checks that epoch before and during stream consumption.

That combination lets an in-flight run stop promptly without requiring process-level cancellation.
