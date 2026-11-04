-- Within Reason's chat relay (docs/design/2026-11-04-chat-permissions.md). The engine hands an AI every chat line
-- of the game with no destination; this client's console shows each line it is allowed to see with a label that
-- says where it went. The widget reads the console and forwards each line to every Within Reason AI hosted on this
-- client as a Lua message, "wr-chat\t<to>\t<player>\t<text>", where <to> is one of: everyone (a player's all-chat),
-- allies (a player's team chat), watching (a spectator's all-chat the players see), spectators (the spectators'
-- channel, or a spectator's all-chat while spectator chat is off), private (a whisper). The AI's shim pairs it with
-- the engine's own message event. Without the widget the AI hears every line unlabelled and keeps to allied players.
local widget = widget
function widget:GetInfo()
	return { name = "WR chat relay", desc = "tells Within Reason AIs hosted here where each chat line went", author = "Within Reason", layer = 0, enabled = true }
end

local spGetPlayerList, spGetPlayerInfo, spGetTeamList, spGetAIInfo = Spring.GetPlayerList, Spring.GetPlayerInfo, Spring.GetTeamList, Spring.GetAIInfo
local spGetMyPlayerID, spSendSkirmishAIMessage = Spring.GetMyPlayerID, Spring.SendSkirmishAIMessage

-- The teams of the Within Reason AIs this client hosts (a Lua message reaches an AI on this client only).
local function ourTeams()
	local teams, me = {}, spGetMyPlayerID()
	for _, teamID in ipairs(spGetTeamList()) do
		local _, _, host, shortName = spGetAIInfo(teamID)
		if host == me and type(shortName) == "string" and shortName:sub(1, 2) == "WR" then
			teams[#teams + 1] = teamID
		end
	end
	return teams
end

-- The console label of each player: "<name> " for a seat, "[name] " for a spectator (a name may hold brackets).
local function labels()
	local out = {}
	for _, playerID in ipairs(spGetPlayerList()) do
		local name, _, spectator = spGetPlayerInfo(playerID, false)
		if name then
			out[#out + 1] = { label = spectator and ("[" .. name .. "] ") or ("<" .. name .. "> "), player = playerID, spectator = spectator }
		end
	end
	return out
end

-- Where a console line went, by its label and channel prefix, as `CGame::HandleChatMsg` writes them.
local function parse(line)
	local best
	for _, l in ipairs(labels()) do
		if line:sub(1, #l.label) == l.label and (not best or #l.label > #best.label) then
			best = l
		end
	end
	if not best then
		return nil
	end
	local rest = line:sub(#best.label + 1)
	local to, text
	if rest:sub(1, 8) == "Allies: " then
		-- A spectator's team line is shown to spectators only (`CGame::HandleChatMsg`).
		to, text = best.spectator and "spectators" or "allies", rest:sub(9)
	elseif rest:sub(1, 12) == "Spectators: " then
		to, text = "spectators", rest:sub(13)
	elseif rest:sub(1, 9) == "Private: " then
		to, text = "private", rest:sub(10)
	else
		to, text = best.spectator and "watching" or "everyone", rest
	end
	return to, best.player, text
end

function widget:AddConsoleLine(lines, priority)
	lines = lines:match("^%[f=[0-9]+%] (.*)$") or lines
	for line in lines:gmatch("[^\n]+") do
		local to, player, text = parse(line)
		if to then
			for _, teamID in ipairs(ourTeams()) do
				spSendSkirmishAIMessage(teamID, "wr-chat\t" .. to .. "\t" .. player .. "\t" .. text)
			end
		end
	end
end
