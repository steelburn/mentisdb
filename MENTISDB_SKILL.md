---
name: mentisdb
description: Durable semantic memory for AI agents — append-only, hash-chained, searchable.
triggers:
  - mentisdb
  - memory
  - remember
  - recall
  - chain
  - thought
---

# MentisDB Skill

## ⚡ MANDATORY STARTUP (DO THIS FIRST)

1. `mentisdb_list_chains` → pick your project's chain (don't create a new one)
2. `mentisdb_bootstrap(chain_key="<found>")` → open it (safe on existing chains)
3. `mentisdb_skill_md` → load THIS skill before reading memory
4. `mentisdb_list_agents` → reuse your existing agent identity
5. `mentisdb_recent_context` → load where you left off
6. Write `Summary` with `role: Checkpoint`

**READ BEFORE WRITE.** Never skip steps 1–5.

## ✍️ WRITE TRIGGERS

Write **immediately** when any becomes true:

| Trigger | Type | Role |
|---------|------|------|
| Non-obvious bug cause | LessonLearned | Retrospective |
| Architectural decision | Decision | Memory |
| Security boundary found | Constraint | Memory |
| Stable convention established | Decision | Memory |
| Dangerous assumption corrected | Correction | Memory |
| Restart point reached | Summary | Checkpoint |
| Framework/ecosystem trap | LessonLearned | Retrospective |
| Expensive operation ahead | Summary | Checkpoint |
| Tool call surprise | LessonLearned | Retrospective |

**One strong memory > many weak ones.** Link to prior thoughts with `refs` or `relations`.

## 📋 THOUGHT TYPES

| Type | Use for | Role |
|------|---------|------|
| Decision | Chosen direction | Memory |
| Constraint | Hard rule | Memory |
| LessonLearned | Lesson from failure/fix | Retrospective |
| Correction | Previous fact wrong (replaces) | Memory |
| Mistake | Wrong action (distinct from Correction) | Memory |
| Insight | Non-obvious realization | Memory |
| PreferenceUpdate | Stable preference | Memory |
| Summary | Compressed state | Checkpoint |

Use `refs: [index]` for positional refs. Use `relations` with `kind` (CausedBy, Corrects, Supersedes, DerivedFrom) for typed edges. 1–3 high-signal refs, not many weak ones.

## 🔍 RETRIEVAL

| Need | Tool |
|------|------|
| Topical search | `mentisdb_ranked_search` |
| Keyword match | `mentisdb_lexical_search` |
| Recent context | `mentisdb_recent_context(last_n=N)` |
| One thought | `mentisdb_get_thought` |
| First thought | `mentisdb_get_genesis_thought` |
| Page history | `mentisdb_traverse_thoughts` |
| Grouped context | `mentisdb_context_bundles` |
| Export markdown | `mentisdb_memory_markdown` |

**Always filter** — supply text, tags, concepts, types, or time window.

## 🏷️ SEARCHABILITY

- `tags`: `rust`, `security`, `api-design`
- `concepts`: `hybrid-retrieval`, `session-bootstrap`
- `importance`: 0.0–1.0 (user=0.8, assistant=0.2)
- `confidence`: 0.0–1.0

## ❌ ANTI-PATTERNS

- Writing raw logs instead of rules
- Creating new agent IDs for same role
- Skipping `recent_context` at start
- Vague summaries ("worked on X")
- Polluting chains with redundant bootstrap
- Loading entire chains without filters
- Forgetting to write checkpoint before context compaction