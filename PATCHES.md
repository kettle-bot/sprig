# Kettle sprig patches

This branch starts at upstream `18e4b37d9` (block/buzz `main`, 2026-11-23). It was
rebased from `cc528dbe0538cff0357b7644a9466ad689a1874f`; the prior reviewed tip was `131ff12a5`.
It carries only the patches needed for customer-scoped managed agents:

| Patch | Upstream | Drop condition |
| --- | --- | --- |
| Admit explicitly allowlisted authors in verified DMs (`da8da904f`) | [block/buzz#7642](https://github.com/block/buzz/pull/7642) | Rebase onto an upstream sprig image that includes #7642. |
| Opt-in strict author allowlist (`c7820a7eb`, `895d7d4f7`) | [block/buzz#8278](https://github.com/block/buzz/pull/8278) | Rebase onto an upstream sprig image that includes the strict policy and its required gate construction. |
| Treat private channels as channels at the author gate (acceptance follow-up) | [block/buzz#8278](https://github.com/block/buzz/pull/8278) | Rebase onto the upstream gate once #8278 includes the private-channel fix. |
| Cap consecutive teammate triggers on admitted authors (top-level by channel, replies by thread), including edits; configure permission-mode setup timeout (`f828e8b98`, `5db0725bd`, `c3709023a`, `7048ff25f`, `9c0625e91`), and count every teammate-authored event that matched a subscription rule, whatever its kind | [block/buzz#8348](https://github.com/block/buzz/pull/8348) | Rebase onto upstream once #8348 ships; remove the Kettle fork patch and keep Kettle's opt-in env only. |

The default author policy remains unchanged. Kettle opts managed agents into strict mode through `BUZZ_ACP_STRICT_ALLOWLIST=true`; BYO agents do not set it.
For the loop cap, top-level teammate posts share a channel counter and replies
use their thread root. Only an admitted owner or allowlisted member without a
`bot` role or NIP-OA profile tag resets it. A Kettle agent that a customer
explicitly allowlists and that appears as a plain `member` therefore counts as
human for resets; customers should allowlist only agents they trust for that.

The cap names no event kinds at the point where a matched event is queued. An
earlier fixed list (`9`, `40003`, `45001`, `45003`) missed stream reminders
(`40007`) and workflow approval requests (`46010`), both admitted by the
default mention rule, so a teammate could still start or steer a turn with one
after the cap was spent. Resets still come only from kinds `9`, `45001` and
`45003` by an admitted human.
