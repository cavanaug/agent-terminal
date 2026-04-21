# M009 Handoff

## Purpose

M008 standardizes the preferred public shell lifecycle without changing runtime or protocol semantics. The repo now teaches one honest path end to end:

1. spawn a named shell with an explicit prompt
2. wait for readiness with `agent-terminal wait`
3. capture a baseline `content_hash`
4. `type` a command and `press Enter`
5. use `snapshot --await-change --settle` after the visible change
6. clean up with `kill` then `stop`

## Deferred grammar work

The broader command-grammar cleanup belongs in M009, not M008. M009 should review whether the public CLI should further simplify or rename compatibility surfaces while preserving honest migration guidance.

Topics to evaluate in M009:

- whether the `key` compatibility alias should remain indefinitely or move toward stronger de-emphasis/deprecation guidance
- whether `wait-for` should stay only as a compatibility alias or eventually converge on a different public grammar
- whether internal names like `Commands::Key` and `Commands::WaitFor` should be renamed after deciding the external migration story
- whether more lifecycle-oriented helper docs/examples should be generated automatically from one shared source

## Guardrails carried forward from M008

- keep the current runtime and protocol semantics unchanged in M008
- do not change daemon behavior or the request/response protocol as part of the wording cleanup
- keep compatibility spellings documented and tested while `press` and `wait` remain the preferred public verbs
- any future renaming work must preserve executable migration notes for existing scripts

## Non-goals for this handoff

This handoff does not authorize changing the current M008 implementation of `Commands::Key`, `Commands::WaitFor`, daemon lifecycle behavior, or shell-session runtime semantics. It records the boundary so M009 can revisit grammar deliberately instead of by accident.
