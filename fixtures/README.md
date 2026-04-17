# fixtures/

Anonymisierte JSONL-Samples für Unit- und Integrationstests.

**Gewährleistung:** Alle `uuid`, `sessionId`, `parentUuid`, `requestId`, `message.id`
und `thinking.signature`-Werte sind synthetisch; `cwd` ist `C:\\example\\project`;
`gitBranch` ist `main`. Token-Zahlen entsprechen plausiblen Real-Werten, um Pricing-
Berechnungen sinnvoll zu testen.

| Datei                  | Szenario                                                 |
|------------------------|----------------------------------------------------------|
| `happy-path.jsonl`     | Eine vollständige Mini-Session: User-Prompt → Assistant (tool_use) → Tool-Result → Assistant (end_turn). |
| `error-synthetic.jsonl`| Eine `<synthetic>`-Error-Message (auth_failed). Muss von der Aggregation ausgeschlossen werden. |
| `sidechain.jsonl`      | Ein Sub-Agent-Assistant mit `isSidechain: true`. Muss in die Session-Aggregation einfließen. |

Schema-Referenz: `docs/jsonl-format.md`.
