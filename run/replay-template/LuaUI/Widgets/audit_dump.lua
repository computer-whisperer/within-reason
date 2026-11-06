-- Within Reason: the audit dump. While the engine plays a demo, write one file of every team's units as the engine
-- holds them, once a second, with what a waste audit needs and the match record does not carry: each unit's order
-- (the head of its queue and where it points), what a builder or a plant is building and at what share of its build
-- power, a frame's progress, each team's pull and excess beside its income, the damage each unit took and the
-- experience it holds (the engine's own tally of the damage it dealt: 0.1 x the game's experienceMult x the share of
-- the victim's health x the victim's power over its own; the game's widget handler hands a widget no attacker with
-- UnitDamaged, and a user widget may not replace that call-in), and the wrecks' metal.
-- Read by run/audit.py (docs/harness/audit.md). Loaded by run/replay_match.py --audit, which makes the marker file
-- this widget looks for; without the marker, and in a live game, it does nothing.
local widget = widget

function widget:GetInfo()
	return { name = "Audit Dump", desc = "writes every unit's state once a second for run/audit.py", layer = 1, enabled = true }
end

local SAMPLE = 30
local FEATURES = 450
local OUT = "LuaUI/Config/within-reason/"
local out
local teams = {}
local allyOf = {}
local enemyAlly = {} -- a team -> one ally team that is its enemy (a two-sided game's other side), for the seen bits
local damage = {} -- victim id -> damage taken since the last sample
local defSeen = {}

local function esc(s)
	return '"' .. tostring(s):gsub('[%c"\\]', function(c) return string.format("\\u%04x", c:byte()) end) .. '"'
end

local function json(v)
	local t = type(v)
	if t == "table" then
		if #v > 0 or next(v) == nil then
			local parts = {}
			for i = 1, #v do parts[i] = json(v[i]) end
			return "[" .. table.concat(parts, ",") .. "]"
		end
		local keys = {}
		for k in pairs(v) do keys[#keys + 1] = k end
		table.sort(keys, function(a, b) return tostring(a) < tostring(b) end)
		local parts = {}
		for i, k in ipairs(keys) do parts[i] = esc(k) .. ":" .. json(v[k]) end
		return "{" .. table.concat(parts, ",") .. "}"
	elseif t == "number" then
		if v ~= v or v == math.huge or v == -math.huge then return "null" end
		if v == math.floor(v) then return string.format("%d", v) end
		return string.format("%.5g", v)
	elseif t == "string" then
		return esc(v)
	elseif t == "boolean" then
		return v and "true" or "false"
	end
	return "null"
end

local function put(row)
	if out then out:write(json(row), "\n") end
end

local function defRow(id)
	local d = UnitDefs[id]
	local reach = 0
	for _, w in ipairs(d.weapons or {}) do
		local wd = WeaponDefs[w.weaponDef]
		if wd and (wd.range or 0) > reach and not wd.isShield and (wd.damages and (wd.damages[0] or 0) > 1 or wd.customParams) then reach = wd.range end
	end
	local options = {}
	for _, o in ipairs(d.buildOptions or {}) do options[#options + 1] = o end
	return {
		id = id, name = d.name, power = d.power or 0, human = d.translatedHumanName or d.humanName or d.name, metal = d.metalCost, energy = d.energyCost, build_time = d.buildTime,
		build_speed = d.buildSpeed or 0, build_distance = d.buildDistance or 0, speed = d.speed or 0, health = d.health, reach = reach,
		weapons = #(d.weapons or {}), building = d.isBuilding and 1 or 0, factory = d.isFactory and 1 or 0, air = d.canFly and 1 or 0,
		extracts = d.extractsMetal or 0, energy_make = d.energyMake or 0, energy_upkeep = d.energyUpkeep or 0, wind = d.windGenerator or 0, tidal = d.tidalGenerator or 0,
		metal_store = d.metalStorage or 0, energy_store = d.energyStorage or 0, can_assist = d.canAssist and 1 or 0, can_reclaim = d.canReclaim and 1 or 0,
		tech = tonumber(d.customParams and d.customParams.techlevel) or 1, sight = d.sightDistance or d.losRadius or 0, radar = d.radarDistance or d.radarRadius or 0,
		xsize = d.xsize, zsize = d.zsize, options = options, converter = tonumber(d.customParams and d.customParams.energyconv_capacity) or 0,
	}
end

function widget:Initialize()
	if not Spring.IsReplay() or not VFS.FileExists(OUT .. "audit.on") then
		widgetHandler:RemoveWidget(self)
		return
	end
	out = io.open(OUT .. "audit.jsonl", "w")
	local gaia = Spring.GetGaiaTeamID()
	local seats = {}
	for _, team in ipairs(Spring.GetTeamList()) do
		if team ~= gaia then
			local _, leader, _, isAI, side, ally = Spring.GetTeamInfo(team, false)
			teams[#teams + 1] = team
			allyOf[team] = ally
			local name = ""
			if isAI then
				local _, aiName, _, shortName, version = Spring.GetAIInfo(team)
				name = (shortName or aiName or "ai") .. " " .. (version or "")
			else
				name = Spring.GetPlayerInfo(leader, false) or ""
			end
			local sx, _, sz = Spring.GetTeamStartPosition(team)
			seats[#seats + 1] = { team = team, ally = ally, side = side or "", name = name, ai = isAI and 1 or 0, start = { math.floor(sx or 0), math.floor(sz or 0) } }
		end
	end
	for _, team in ipairs(teams) do
		for _, other in ipairs(teams) do
			if allyOf[other] ~= allyOf[team] then enemyAlly[team] = allyOf[other] end
		end
	end
	local spots = {}
	for i = 1, Spring.GetGameRulesParam("mex_count") or 0 do
		spots[#spots + 1] = { Spring.GetGameRulesParam("mex_x" .. i) or 0, Spring.GetGameRulesParam("mex_z" .. i) or 0, Spring.GetGameRulesParam("mex_metal" .. i) or 0 }
	end
	put({ t = "header", format = "within-reason-audit", version = 1, sample_frames = SAMPLE, map = { name = Game.mapName, width = Game.mapSizeX, height = Game.mapSizeZ }, seats = seats, spots = spots })
	Spring.Echo("<audit dump> " .. #teams .. " teams")
end

local function def(id)
	if not defSeen[id] then
		defSeen[id] = true
		put({ t = "def", d = defRow(id) })
	end
	return id
end

local function resources(team, kind)
	local cur, store, pull, income, expense, _, sent, received, excess = Spring.GetTeamResources(team, kind)
	return { cur or 0, store or 0, pull or 0, income or 0, expense or 0, excess or 0, sent or 0, received or 0 }
end

local function stats(team)
	local n = Spring.GetTeamStatsHistory(team)
	if not n or n < 1 then return {} end
	local h = Spring.GetTeamStatsHistory(team, n, n)
	local st = h and h[1]
	if not st then return {} end
	return {
		metal_used = st.metalUsed, metal_produced = st.metalProduced, metal_excess = st.metalExcess, energy_used = st.energyUsed, energy_produced = st.energyProduced,
		energy_excess = st.energyExcess, damage_dealt = st.damageDealt, damage_received = st.damageReceived, units_produced = st.unitsProduced, units_died = st.unitsDied, units_killed = st.unitsKilled,
	}
end

function widget:GameFrame(frame)
	if not out or frame % SAMPLE ~= 0 then return end
	local units = {}
	local sides = {}
	for _, team in ipairs(teams) do
		sides[#sides + 1] = { team = team, m = resources(team, "metal"), e = resources(team, "energy"), st = stats(team) }
		local seenBy = enemyAlly[team]
		for _, id in ipairs(Spring.GetTeamUnits(team)) do
			local defID = def(Spring.GetUnitDefID(id))
			local d = UnitDefs[defID]
			local x, _, z = Spring.GetUnitPosition(id)
			local health, maxHealth, _, _, progress = Spring.GetUnitHealth(id)
			local queue = Spring.GetCommandQueue(id, 0) or 0
			local cmd, _, _, p1, p2, p3 = Spring.GetUnitCurrentCommand(id)
			local tx, tz, target = 0, 0, 0
			if cmd and p3 then tx, tz = math.floor(p1), math.floor(p3) elseif cmd and p1 then target = math.floor(p1) end
			local builds, power, lined = 0, 0, 0
			if (d.buildSpeed or 0) > 0 then
				builds = Spring.GetUnitIsBuilding(id) or 0
				power = math.floor((Spring.GetUnitCurrentBuildPower(id) or 0) * 100 + 0.5)
			end
			if d.isFactory then lined = Spring.GetFactoryCommands(id, 0) or 0 end
			local seen = 0
			if seenBy then seen = Spring.GetUnitLosState(id, seenBy, true) or 0 end
			if type(seen) ~= "number" then seen = 0 end
			units[#units + 1] = {
				id, defID, team, math.floor(x or 0), math.floor(z or 0), (health and maxHealth and maxHealth > 0) and math.floor(health / maxHealth * 100) or 0,
				math.floor((progress or 1) * 100), queue, cmd or 0, tx, tz, target, builds, power, type(lined) == "number" and lined or 0, seen % 4, Spring.GetUnitExperience(id) or 0,
			}
		end
	end
	local hits = {}
	for victim, amount in pairs(damage) do hits[#hits + 1] = { victim, math.floor(amount + 0.5) } end
	damage = {}
	put({ t = "s", f = frame, sides = sides, u = units, hits = hits })
	if frame % FEATURES == 0 then
		local wrecks = {}
		for _, fid in ipairs(Spring.GetAllFeatures()) do
			local metal = Spring.GetFeatureResources(fid)
			if metal and metal >= 1 then
				local x, _, z = Spring.GetFeaturePosition(fid)
				local fd = FeatureDefs[Spring.GetFeatureDefID(fid)]
				wrecks[#wrecks + 1] = { fid, fd and fd.name or "", math.floor(x or 0), math.floor(z or 0), math.floor(metal) }
			end
		end
		put({ t = "wrecks", f = frame, w = wrecks })
	end
	out:flush()
end

local function place(id)
	local x, _, z = Spring.GetUnitPosition(id)
	return math.floor(x or 0), math.floor(z or 0)
end

function widget:UnitCreated(id, defID, team, builder)
	if not out then return end
	local x, z = place(id)
	put({ t = "ev", f = Spring.GetGameFrame(), k = "created", u = id, d = def(defID), team = team, x = x, z = z, by = builder or 0 })
end

function widget:UnitFinished(id, defID, team)
	if not out then return end
	local x, z = place(id)
	put({ t = "ev", f = Spring.GetGameFrame(), k = "finished", u = id, d = def(defID), team = team, x = x, z = z })
end

function widget:UnitDestroyed(id, defID, team, attacker, attackerDef, attackerTeam)
	if not out then return end
	local x, z = place(id)
	local _, _, _, _, progress = Spring.GetUnitHealth(id)
	put({ t = "ev", f = Spring.GetGameFrame(), k = "destroyed", u = id, d = def(defID), team = team, x = x, z = z, by = attacker or 0, by_d = attackerDef or 0, by_team = attackerTeam or -1, progress = math.floor((progress or 1) * 100) })
end

function widget:UnitGiven(id, defID, newTeam, oldTeam)
	if not out then return end
	put({ t = "ev", f = Spring.GetGameFrame(), k = "given", u = id, d = def(defID), team = newTeam, from = oldTeam })
end

function widget:UnitDamaged(id, defID, team, amount, paralyzer)
	if not out or paralyzer then return end
	damage[id] = (damage[id] or 0) + amount
end

function widget:UnitCommand(id, defID, team, cmdID, params, opts)
	if not out then return end
	local row = { t = "cmd", f = Spring.GetGameFrame(), u = id, team = team, c = cmdID, shift = (opts and opts.shift) and 1 or 0 }
	if params then
		if #params >= 3 then
			row.x, row.z = math.floor(params[1]), math.floor(params[3])
		elseif #params >= 1 then
			row.target = math.floor(params[1])
		end
	end
	put(row)
end

function widget:GameOver(winners)
	put({ t = "over", f = Spring.GetGameFrame(), winners = winners or {} })
	if out then out:close() end
	out = nil
end

function widget:Shutdown()
	if out then out:close() end
	out = nil
end
