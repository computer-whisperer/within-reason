# Chat by the game's permissions (2026-11-04)

The user, after the rove-2 arc: "let's fix our chat participation to match the game's intended permission system,
particularly with respect to spectators and whether or not players can see them", then "we should build this to
degrade gracefully if the plugin is disallowed in a lobby".

## 1. What the game intends (`CGame::HandleChatMsg`, engine 2026.07.04)

A player is shown: all-chat from players; team chat from players of their own ally team; all-chat from spectators
unless the host has turned spectator chat off (the engine's `/nospectatorchat`, SPADS's `noSpecChat`); whispers to
them from their own kind (player to player, spectator to spectator). Never: the enemy's team chat, the spectators'
own channel (`s:` lines, and every spectator's all-chat while spectator chat is off), other people's whispers. A
spectator is shown everything. BAR's code of conduct (B5) calls passing information into a running game through
spectator chat cheating. BAR's chat widget tags a spectator's line as a spectator's, and a player can hide those
(`HideSpecChat`).

## 2. What we did until this piece

- **Hearing.** The engine hands an AI every chat line of the game after the client's own filter, as `(player,
  text)` with no destination (`HandleChatMsg`'s last line; the server broadcasts every chat packet to every client).
  The bot relayed each line to the player marked by who spoke and which side they are on, so the enemy's team chat,
  the spectators' channel and everyone's whispers reached the player. In the twelve lobby games the lines heard were
  greetings and advice from thebluegecko and our own bots' banners; no leak is in the logs, but the channel was open.
- **Speaking.** In every lobby game the user hosted the AI as a spectator (all twelve start scripts), so every line
  the player said was a spectator's all-chat: both sides read it (game 1, 1:13, thebluegecko: "shhh dont say that,
  your oponent will know what faction you are playing and where you spawned"), a player could hide it, and under a
  host with spectator chat off nobody playing would see it while the tool said "said". A spectating host cannot
  reach its allies at all: a spectator's `a:` line goes to spectators, a spectator's whisper to a player is dropped.
  The brief told the player its lines went "to everyone playing".

## 3. The piece

The destination is visible in one place only: the host's client prints every chat line it is shown to its console
with a label (`<name> text` all-chat, `<name> Allies: text` team chat, `[name] text` a spectator's all-chat the
players see, `[name] Spectators: text` the spectators' channel or a muted all-chat, `Private:` a whisper), and a
spectating client is shown them all.

1. **The relay widget** `run/match-template/LuaUI/Widgets/wr_chat_relay.lua` on the host's client reads those
   console lines (`widget:AddConsoleLine`) and forwards each as a Lua message (`Spring.SendSkirmishAIMessage`, an
   unsynced call widgets may make) to every Within Reason AI hosted on that client: `wr-chat\t<to>\t<player>\t<text>`.
   The arena's match template ships it; `run/install_ai.sh` copies it into a BAR install's `LuaUI/Widgets/` beside
   the AI (never into the arena's own data dir, whose matches take it from the template). BAR loads user widgets when
   the game's `allowuserwidgets` option is on, and always for a spectating client (`barwidgets.lua`), which the host
   of our lobby games is; a lobby that forbids user widgets simply runs without it.
2. **The shim** pairs a relayed line with the engine's own message event and sends `Event::Chat { player, text,
   to }`. The event comes first: the console reaches widgets from `CGame::Update`, not from the `LOG` call (measured
   in chat-relay-2: the banner's event at frame 62, its label at 68), so the shim holds a line up to
   `CHAT_RELAY_WAIT` (30 frames) for its label and sends it `to: Unknown` after that (the widget is not running, or
   the host's client was not shown it; chat-relay-3-nowidget: unlabelled at frame 98 after the event at 68).
3. **The bot hears as a player on its side would** (`people.rs` `hears`): all-chat from players, team chat from
   our own allies, spectators' lines when the players can see them, nothing else. While the destinations are unknown
   (no widget) it hears allied players only, and says so once in the report, when the first echo of its own line comes
   back unlabelled; with the widget running an unlabelled line is one the host was not shown, and is dropped. Our
   own host's banners for other seats of ours are dropped too.
4. **`say` takes a target**, `everyone` (the default) or `allies` (`/say a:`, the engine's prefix). From a
   spectating host `allies` is refused with the reason; the `people` entry says the host watches, that both sides read
   every line, and that nothing reaches the allies alone. When the echo of our own line comes back labelled as the
   spectators' channel, the report says once that the players cannot see our lines in this game.
5. **The briefs** say "to everyone in the game, his side too" instead of "to everyone playing".

Rule H-CHAT-PERMISSIONS in `docs/heuristics.md`; the claims K-mech-the-engine-hands-an-ai-every-chat-line and
K-mech-a-spectating-hosts-ai-speaks-as-a-spectator in `docs/knowledge/mechanics.md`; the engine facts in §1 are
read from the source, the relay's labels are measured in the arena (its host is a spectator too: chat-relay-2's
banner came back `Watching`). The `a:` channel and a person's team line are read from the engine, not yet seen in a
game with people.

What it cannot give: a private line to our allies while the user hosts as a spectator. That is the game's rule.
