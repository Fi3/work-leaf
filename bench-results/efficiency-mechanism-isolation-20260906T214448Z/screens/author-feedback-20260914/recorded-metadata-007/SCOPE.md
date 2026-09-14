# P09 exact recorded-message input restoration

Prospective scope: 2026-09-14 23:07 UTC. One distinct remaining P09 setup gate.
The input006 natural plugin toggle produced no repeated instruction and is
parked. This alternative uses the documented additional developer-instruction
setting, in a private benchmark configuration only, to deliver the exact saved
1,014-byte plugin-usage message at its recorded boundary. It does not claim to
reproduce the original loader decision, and contains no operator-authored
guidance, altered author policy, fake feedback or changed task requirement.

Source: the isolated plugins.usage_instructions developer item in the saved
third-author first review-fix turn, SHA-256
81d37acb6feff552b562f712b1e854339d90a0b19aa9f271047cca787f77a51a.
Official setting: [developer_instructions](https://learn.chatgpt.com/docs/config-file/config-reference).
Additional message metadata/provenance must remain explicit; exact model-visible
role/content and future delivery are the required qualification, not just its presence.

First use a no-generation private prompt preview and exact parsed-config diff.
Then at most one sixty-second/100,000-raw/three-turn read-only subscription
diagnostic: ordinary short launch, same-thread resume with the saved message,
same-thread resume with the normal short profile again. Predict one additional
exact message on the second turn and no extra message on the third. Reject
rewritten roles/text, extra instructions, omitted base input or repeat injection.
Use the existing tested scheduler/driver, no new provider or workflow host.
No completed or historical native thread is resumed. Local preparation cap:
ten minutes, recorded separately from prior qualification allocations.

Preserve the outcome and once-only accounting. A failed gate parks this
alternative without an automatic retry or a full-batch admission. Source/input
drift and resource/monitor failures stop this diagnostic, not the investigation.
The latest no-routine-pause authority governs continued useful in-scope work.
