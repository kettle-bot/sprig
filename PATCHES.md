# Kettle sprig patches

This branch starts at upstream `cc528dbe0538cff0357b7644a9466ad689a1874f`.
It carries only the patches needed for customer-scoped managed agents:

| Patch | Upstream | Drop condition |
| --- | --- | --- |
| Admit explicitly allowlisted authors in verified DMs (`7499688ee`) | [block/buzz#7642](https://github.com/block/buzz/pull/7642) | Rebase onto an upstream sprig image that includes #7642. |
| Opt-in strict author allowlist (`7ea58b403`, `737a8d411`) | [block/buzz#8278](https://github.com/block/buzz/pull/8278) | Rebase onto an upstream sprig image that includes the strict policy and its required gate construction. |
| Treat private channels as channels at the author gate (acceptance follow-up) | [block/buzz#8278](https://github.com/block/buzz/pull/8278) | Rebase onto the upstream gate once #8278 includes the private-channel fix. |

The default author policy remains unchanged. Kettle opts managed agents into strict mode through `BUZZ_ACP_STRICT_ALLOWLIST=true`; BYO agents do not set it.
