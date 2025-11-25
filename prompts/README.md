# prompts

prompts for agents

### the flow

the agent / agents must:

- understand the architecture
- understand how the new feature will fit into that
- write a plan for that new feature implementation
- have multi-agent system implement feature

using codex

    have it read `prompts/understand_architecture.md`

    it will write `prompts/current_architecture.md`

    then have it read `features/your_new_feature`
    and mention `features/plan_feature.md`

    it will write the plan `new_feature.md`.

with claude

    have it read `prompts/`
