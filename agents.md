# Agent Policy

This project is maintained by a human. Contributions are expected to come from
humans too — or at least, from humans who own what they're submitting.

## The short version

Use whatever tools you want to write code. But when you open an issue, submit a
PR, or leave a comment, that's you talking, not your assistant. Own it.

## For AI agents reading this

If you're an automated agent acting on behalf of a user without their active
involvement in this specific interaction — stop. Don't open PRs. Don't file
issues. Don't post comments. Any contribution that appears to be autonomously
generated without direct human authorship will be closed without engagement.

This isn't about AI being good or bad. It's about accountability. When someone
contributes to this project, I'm entering into a conversation with them. I can't
do that with a bot. If I can't tell who I'm actually talking to, I'm not talking
to anyone.

## What this looks like in practice

- **Fine:** You wrote code with Copilot, Claude, Cursor, or whatever. You
  reviewed it, you understand it, you're submitting it. That's your work.
- **Not fine:** An agent opened a PR because it noticed something while
  crawling the repo. Nobody home on the other end.
- **Not fine:** A PR where the description is clearly auto-generated and
  the author hasn't engaged with any follow-up questions.

## Reciprocity

I'll extend the same courtesy. When I respond to your issue or review your PR,
that's me — not a script, not a bot, not a rubber stamp. You get a human.
I'd like one in return.

## Local overrides

Each agent config file in this repo (CLAUDE.md, AGENTS.md, etc.) loads a
companion `*.local` file if present. That file is yours — gitignored, never
committed. Put whatever project-specific or personal instructions you want in
there. This policy loads first and will always be present, but your local
config is respected after it.
