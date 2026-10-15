# Kettle sprig patches

This branch starts at upstream `cc528dbe0538cff0357b7644a9466ad689a1874f`.
It carries only the patches needed for customer-scoped managed agents:

| Patch | Upstream | Drop condition |
| --- | --- | --- |
| Admit explicitly allowlisted authors in verified DMs (`7499688ee`) | [block/buzz#7642](https://github.com/block/buzz/pull/7642) | Rebase onto an upstream sprig image that includes #7642. |
| Opt-in strict author allowlist (`7ea58b403`, `737a8d411`) | [block/buzz#8278](https://github.com/block/buzz/pull/8278) | Rebase onto an upstream sprig image that includes the strict policy and its required gate construction. |
| Treat private channels as channels at the author gate (acceptance follow-up) | [block/buzz#8278](https://github.com/block/buzz/pull/8278) | Rebase onto the upstream gate once #8278 includes the private-channel fix. |
| Cap consecutive teammate triggers per thread on admitted authors, including edits; configure permission-mode setup timeout (`9f2c44e9b`, `56f160cc4`, `831114907`) | [block/buzz#8348](https://github.com/block/buzz/pull/8348) | Rebase onto upstream once #8348 ships; remove the Kettle fork patch and keep Kettle's opt-in env only. |

The default author policy remains unchanged. Kettle opts managed agents into strict mode through `BUZZ_ACP_STRICT_ALLOWLIST=true`; BYO agents do not set it.
For the loop cap, top-level teammate posts share a channel counter and replies
use their thread root. Only an admitted owner or allowlisted member without a
`bot` role or NIP-OA profile tag resets it. A Kettle agent that a customer
explicitly allowlists and that appears as a plain `member` therefore counts as
human for resets; customers should allowlist only agents they trust for that.
