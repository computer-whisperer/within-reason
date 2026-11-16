-- Within Reason: the tape dump. While the headless engine plays a demo, or plays an arena game with this widget in
-- its spectating host, write everything a viewer needs to show the game as the engine would have: every unit's place and attitude ten times a second, health and build progress, what
-- each unit is doing, every projectile (where it began, how it flew, when it ended), the explosions and muzzle fire
-- the game reports, the features (trees, rocks, wrecks), the ground's heights and every change to them, the orders of
-- both sides and the teams' resources. One raw stream, `tape.raw`, which `target/release/tape pack` turns into the
-- seekable `game.tape` (docs/design/2026-11-15-game-tape.md, docs/harness/tape-format.md).
--
-- Loaded by run/replay_match.py --tape (and run/tape.py) and by the arena (this file is the only copy: the arena copies
-- it into each match), which make the marker file this widget looks for; without the marker it does nothing. In a
-- live game it needs a spectator's full view, as a replay's: anything less would be one side's picture.
--
-- The raw stream is a run of records: one byte of kind, a little-endian u32 of length, the payload. Kind `J` is a
-- JSON object whose `t` names it (header, unitdef, weapondef, featuredef, ev, cmd, res, over, stats); the others are
-- packed numbers, laid out where each is written below.
local widget = widget

function widget:GetInfo()
	return { name = "Tape Dump", desc = "writes the raw stream of a game for the tape (run/tape.py)", layer = 2, enabled = true }
end

local OUT = "LuaUI/Config/within-reason/"
local SAMPLE = 3 -- frames between unit samples: the bot's own tick
local VITALS = 15 -- frames between looks at every unit's health and build progress
local ACTIVITY = 30 -- frames between looks at every unit's head order
local RESOURCES = 30
local CHECK = 15 -- frames between positions of the projectiles the reader works out itself (the check of that working)
local POS = 4 -- a position is whole quarters of an elmo
local ANGLE = 4096 -- an attitude's angle is whole 4096ths of a turn (0.09 degrees)
local AIM = 1024 -- a weapon's direction is whole 1024ths of a turn

local spGetUnitPosition = Spring.GetUnitPosition
local spGetUnitDirection = Spring.GetUnitDirection
local spGetUnitHealth = Spring.GetUnitHealth
local spGetUnitDefID = Spring.GetUnitDefID
local spGetUnitTeam = Spring.GetUnitTeam
local spGetUnitIsCloaked = Spring.GetUnitIsCloaked
local spGetUnitCurrentCommand = Spring.GetUnitCurrentCommand
local spGetCommandQueue = Spring.GetCommandQueue
local spGetUnitIsBuilding = Spring.GetUnitIsBuilding
local spGetUnitWeaponVectors = Spring.GetUnitWeaponVectors
local spGetProjectilesInRectangle = Spring.GetProjectilesInRectangle
local spGetProjectilePosition = Spring.GetProjectilePosition
local spGetProjectileVelocity = Spring.GetProjectileVelocity
local spGetProjectileDefID = Spring.GetProjectileDefID
local spGetProjectileType = Spring.GetProjectileType
local spGetProjectileOwnerID = Spring.GetProjectileOwnerID
local spGetProjectileTeamID = Spring.GetProjectileTeamID
local spGetProjectileTarget = Spring.GetProjectileTarget
local spGetProjectileTimeToLive = Spring.GetProjectileTimeToLive
local spGetProjectileGravity = Spring.GetProjectileGravity
local spGetGroundHeight = Spring.GetGroundHeight
local spGetTimer = Spring.GetTimer
local spDiffTimers = Spring.DiffTimers
local packU16, packU32, packS16, packS32, packF32 = VFS.PackU16, VFS.PackU32, VFS.PackS16, VFS.PackS32, VFS.PackF32
local floor, atan2, asin, sqrt = math.floor, math.atan2, math.asin, math.sqrt
local TURN = 2 * math.pi

local out
local aimOn = false
local skipOn = false
local frameNow = 0
local bytes = 0

-- Units. `mobile` is the list sampled every SAMPLE frames; a building is written once, when it appears.
local known = {} -- unit id -> def id
local mobile, mobileAt, mobileCount = {}, {}, 0
local all, allAt, allCount = {}, {}, 0
local last = {} -- unit id -> { x, y, z, yaw, pitch, roll } as last written
local vital = {} -- unit id -> { health, build, paralyze, flags } as last written
local sight = {} -- unit id -> { in sight, on radar } as last written (the `S` rows)
local doing = {} -- unit id -> { cmd, a, b, c, queue, builds } as last written
local dirty = {} -- unit ids hit since the last sample: their health is read at the next
local armed = {} -- unit id -> number of weapons, for the units whose aim is sampled
local aimLast = {} -- unit id * 16 + weapon -> packed direction as last written
local defSeen, weaponSeen, featureSeen = {}, {}, {}
local cloaks = {} -- def id -> true for the types that can cloak
local mobileDef = {} -- def id -> true for the types that move

-- Projectiles. `flown[wdef]` says the reader cannot work the path out from the start alone (WORKED_OUT below).
local live, liveAt, liveCount = {}, {}, 0
local seen = {} -- projectile id -> the poll that last listed it
local flies = {} -- projectile id -> true when its place is written every SAMPLE frames
local endedAt = {} -- projectile id -> the frame it was last seen to end, for the count of quick reuse
local madeBy = {} -- projectile id -> its owner and weapon type as one number: a change is another projectile
local flown = {}
local poll = 0
local explosions, barrels = {}, {}

local stats = { units = 0, vitals = 0, activity = 0, sight = 0, aim = 0, projectiles = 0, events = 0, sight_rows = 0, marks = 0, unit_rows = 0, vital_rows = 0, aim_rows = 0,
	projectile_new = 0, projectile_rows = 0, projectile_ended = 0, projectile_peak = 0, quick_reuse = 0, same_frame_reuse = 0, explosions = 0, barrels = 0, cmds = 0, piece_projectiles = 0 }
local wallStart

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
		if v == floor(v) then return string.format("%d", v) end
		return string.format("%.6g", v)
	elseif t == "string" then
		return esc(v)
	elseif t == "boolean" then
		return v and "true" or "false"
	end
	return "null"
end

local function record(kind, payload)
	out:write(kind, packU32(#payload), payload)
	bytes = bytes + 5 + #payload
end

local function put(row)
	if out then record("J", json(row)) end
end

local function round(v)
	return floor(v + 0.5)
end

-- An angle in radians as whole `scale`ths of a turn, from -scale/2 up to scale/2 - 1.
local function turns(angle, scale)
	return (floor(angle / TURN * scale + 0.5) + scale / 2) % scale - scale / 2
end

-- A unit's attitude from its front and up directions, as three angles a reader turns back into the two: the heading
-- (about the vertical, 0 facing south, +z), the pitch (the front's rise) and the roll (the up's turn about the front
-- from where it would be with no roll). Six numbers to three, which a column of small differences packs far better
-- (measured on the Erebos 10v8: 10.1 MB against 2.5).
local function attitude(fx, fy, fz, ux, uy, uz)
	local yaw = atan2(fx, fz)
	local pitch = asin(fy > 1 and 1 or (fy < -1 and -1 or fy))
	-- the up with no roll: the vertical less its part along the front
	local ax, ay, az = -fx * fy, 1 - fy * fy, -fz * fy
	local n = sqrt(ax * ax + ay * ay + az * az)
	local roll = 0
	if n > 1e-4 then
		ax, ay, az = ax / n, ay / n, az / n
		local cx, cy, cz = ay * uz - az * uy, az * ux - ax * uz, ax * uy - ay * ux
		roll = atan2(cx * fx + cy * fy + cz * fz, ax * ux + ay * uy + az * uz)
	end
	return turns(yaw, ANGLE), turns(pitch, ANGLE), turns(roll, ANGLE)
end

------------------------------------------------------------------------------------------------------------------------
-- The files a viewer draws with (models, their textures, the map's own files), copied out of the engine's archives as
-- they are: the engine's archive layer finds each one whatever holds it (the game's pool, the map's archive), so the
-- copy is the file the game used. A copy is named by its content (the base64 of its MD5, made safe for a file name)
-- and its extension; `assets.have` lists the names the cache already holds (run/replay_match.py writes it and moves
-- the new copies into run/data/tape-assets afterwards), and an `asset` row says which copy a path is.
local have = {}
local assetOf = {} -- lower-case path -> the copy's name, or false when the file could not be read

local function asset(path)
	if not path or path == "" then return nil end
	local key = string.lower(path)
	if assetOf[key] ~= nil then return assetOf[key] or nil end
	local data = VFS.LoadFile(path, VFS.ZIP) or VFS.LoadFile(path)
	if not data or #data == 0 then
		assetOf[key] = false
		return nil
	end
	local hash = (VFS.CalculateHash(data, 0) or ""):gsub("%+", "-"):gsub("/", "_"):gsub("=", "")
	local name = hash .. "." .. string.lower(path:match("%.(%w+)$") or "bin")
	if not have[name] then
		local f = io.open(OUT .. "assets/" .. name, "wb")
		if f then
			f:write(data)
			f:close()
			have[name] = true
		end
	end
	assetOf[key] = name
	put({ t = "asset", path = key, file = name, bytes = #data })
	return name, data
end

-- A model and the two textures its file names (an S3O's header holds the offsets of their names at bytes 44 and 48).
local function modelAssets(path)
	local _, data = asset(path)
	if not data or data:sub(1, 11) ~= "Spring unit" then return end
	for _, at in ipairs({ 45, 49 }) do
		local offset = VFS.UnpackU32(data, at)
		if offset and offset > 0 and offset < #data then
			local stop = data:find("\0", offset + 1, true) or (#data + 1)
			local texture = data:sub(offset + 1, stop - 1)
			if texture ~= "" then asset("unittextures/" .. texture) end
		end
	end
end

------------------------------------------------------------------------------------------------------------------------
-- The tables: a type's row is written before the first row that names it.

local function weaponRow(id)
	local w = WeaponDefs[id]
	local v = w.visuals or {}
	return {
		t = "weapondef", id = id, name = w.name, kind = w.type, range = w.range, speed = w.projectilespeed, start_speed = w.startvelocity,
		gravity = w.myGravity, aoe = w.damageAreaOfEffect, size = w.size, beamtime = w.beamtime, beam_ttl = w.beamTTL, duration = w.duration,
		intensity = w.intensity, reload = w.reload, salvo = w.salvoSize, tracks = w.tracks and 1 or 0, turn_rate = w.turnRate, wobble = w.wobble,
		dance = w.dance, trajectory_height = w.trajectoryHeight, flight_time = w.flightTime, uptime = w.uptime, large_beam = w.largeBeamLaser and 1 or 0,
		shield = w.isShield and 1 or 0, shield_radius = w.shieldRadius, water = w.waterWeapon and 1 or 0, interceptor = w.interceptor,
		damage = w.damages and w.damages[0] or 0, paralyzer = w.paralyzer and 1 or 0,
		model = v.modelName or "", color = { v.colorR or 1, v.colorG or 1, v.colorB or 1 }, color2 = { v.color2R or 1, v.color2G or 1, v.color2B or 1 },
		thickness = v.thickness or 0, core_thickness = v.coreThickness or 0, flare = v.laserFlareSize or 0, smoke_trail = v.smokeTrail and 1 or 0,
	}
end

-- The kinds whose flight a reader works out from the start alone: a shell and a bomb by the formula (measured against
-- the engine's own places: under a tenth of an elmo), a beam as a line to what it was aimed at. Every other kind is
-- sampled: missiles, torpedoes, flame, the D-gun's ground-hugging ball, and laser bolts (a bolt stops where it hits
-- and lingers while it fades: the formula flew on, 3,700 elmos past on the Erebos 10v8).
local WORKED_OUT = { Cannon = true, AircraftBomb = true, BeamLaser = true, LightningCannon = true }

local function weapon(id)
	if id and id >= 0 and not weaponSeen[id] and WeaponDefs[id] then
		weaponSeen[id] = true
		local row = weaponRow(id)
		flown[id] = not WORKED_OUT[row.kind]
		if row.model ~= "" then modelAssets(row.model) end
		put(row)
	end
	return id
end

local function modelOf(d)
	local m = d.model or {}
	local textures = m.textures or {}
	return { path = d.modelpath or "", tex1 = textures.tex1 or "", tex2 = textures.tex2 or "", min = { m.minx or 0, m.miny or 0, m.minz or 0 }, max = { m.maxx or 0, m.maxy or 0, m.maxz or 0 } }
end

local function unitDef(id)
	if defSeen[id] then return id end
	defSeen[id] = true
	local d = UnitDefs[id]
	local weapons = {}
	for i, w in ipairs(d.weapons or {}) do
		weapons[i] = weapon(w.weaponDef)
	end
	local options = {}
	for _, o in ipairs(d.buildOptions or {}) do options[#options + 1] = o end
	local cp = d.customParams or {}
	local blast = WeaponDefNames and d.deathExplosion and WeaponDefNames[string.lower(d.deathExplosion)]
	mobileDef[id] = not d.isImmobile
	cloaks[id] = d.canCloak or false
	modelAssets(d.modelpath)
	put({
		t = "unitdef", id = id, name = d.name, human = d.translatedHumanName or d.humanName or d.name, model = modelOf(d),
		xsize = d.xsize, zsize = d.zsize, height = d.height, radius = d.radius, speed = d.speed or 0, mobile = mobileDef[id] and 1 or 0,
		air = d.canFly and 1 or 0, building = d.isBuilding and 1 or 0, factory = d.isFactory and 1 or 0, builder = (d.buildSpeed or 0) > 0 and 1 or 0,
		health = d.health, metal = d.metalCost, energy = d.energyCost, build_time = d.buildTime, build_speed = d.buildSpeed or 0,
		build_distance = d.buildDistance or 0, extracts = d.extractsMetal or 0, sight = d.sightDistance or 0, radar = d.radarDistance or 0,
		sonar = d.sonarDistance or 0, jammer = d.radarDistanceJam or 0, cloak = cloaks[id] and 1 or 0, tech = tonumber(cp.techlevel) or 1,
		icon = d.iconType or "", weapons = weapons, options = options, wreck = d.corpse or "", death = weapon(blast and blast.id or -1),
	})
	return id
end

local function featureDef(id)
	if featureSeen[id] then return id end
	featureSeen[id] = true
	local d = FeatureDefs[id]
	modelAssets(d.modelpath)
	put({
		t = "featuredef", id = id, name = d.name, model = modelOf(d), xsize = d.xsize, zsize = d.zsize, height = d.height, radius = d.radius,
		metal = d.metal, energy = d.energy, reclaimable = d.reclaimable and 1 or 0, blocking = d.blocking and 1 or 0, geo = d.geoThermal and 1 or 0,
		tree = (d.drawTypeString == "tree" or (d.drawType or 0) > 0) and 1 or 0,
	})
	return id
end

------------------------------------------------------------------------------------------------------------------------
-- The ground: the heights of the engine's height map (one a corner of a square of 8 elmos, rows from the north), the
-- whole map at the start and each rectangle the game changes after. `G`, at most 64 rows a record: u32 frame, u32 first
-- column, u32 first row, u32 columns, u32 rows, then the heights as f32, row by row.

local function ground(x1, z1, x2, z2)
	local width = x2 - x1 + 1
	local row = z1
	while row <= z2 do
		local rows = math.min(64, z2 - row + 1)
		local heights, n = {}, 0
		for z = row, row + rows - 1 do
			for x = x1, x2 do
				n = n + 1
				heights[n] = spGetGroundHeight(x * 8, z * 8)
			end
		end
		record("G", packU32(frameNow, x1, row, width, rows) .. packF32(heights))
		row = row + rows
	end
end

------------------------------------------------------------------------------------------------------------------------
-- Units.

local function add(list, at, count, id)
	count = count + 1
	list[count] = id
	at[id] = count
	return count
end

local function drop(list, at, count, id)
	local i = at[id]
	if not i then return count end
	local moved = list[count]
	list[i] = moved
	at[moved] = i
	list[count] = nil
	at[id] = nil
	return count - 1
end

-- A unit's first row: `created` with where it stands and how it is turned, written for every unit, moving or not.
local function meet(id, defID, team, builder)
	if known[id] then return end
	defID = defID or spGetUnitDefID(id)
	if not defID then return end
	unitDef(defID)
	known[id] = defID
	allCount = add(all, allAt, allCount, id)
	if mobileDef[defID] then mobileCount = add(mobile, mobileAt, mobileCount, id) end
	local weapons = #(UnitDefs[defID].weapons or {})
	if weapons > 0 then armed[id] = math.min(weapons, 8) end
	local x, y, z = spGetUnitPosition(id)
	local fx, fy, fz, _, _, _, ux, uy, uz = spGetUnitDirection(id)
	local health, maxHealth, _, _, build = spGetUnitHealth(id)
	local yaw, pitch, roll = attitude(fx or 0, fy or 0, fz or 1, ux or 0, uy or 1, uz or 0)
	put({ t = "ev", k = "created", f = frameNow, u = id, d = defID, team = team or spGetUnitTeam(id), by = builder or -1,
		p = { round((x or 0) * POS), round((y or 0) * POS), round((z or 0) * POS) }, rot = { yaw, pitch, roll },
		health = health or 0, max_health = maxHealth or 0, build = build or 1 })
	dirty[id] = true
end

local function forget(id)
	known[id] = nil
	allCount = drop(all, allAt, allCount, id)
	mobileCount = drop(mobile, mobileAt, mobileCount, id)
	last[id], vital[id], doing[id], dirty[id], armed[id], sight[id] = nil, nil, nil, nil, nil, nil
end

-- `U`: u32 frame, u32 n, then n unit ids (u16), 3n position components (s32, quarters of an elmo) and 3n angles
-- (s16: heading, pitch, roll, 4096ths of a turn). Only the units that moved or turned since their last row.
local function sampleUnits(frame)
	local ids, pos, rot, n = {}, {}, {}, 0
	for i = 1, mobileCount do
		local id = mobile[i]
		local x, y, z = spGetUnitPosition(id)
		if x then
			local fx, fy, fz, _, _, _, ux, uy, uz = spGetUnitDirection(id)
			x, y, z = round(x * POS), round(y * POS), round(z * POS)
			local yaw, pitch, roll = attitude(fx, fy, fz, ux, uy, uz)
			local l = last[id]
			if not l then
				l = {}
				last[id] = l
			end
			if l[1] ~= x or l[2] ~= y or l[3] ~= z or l[4] ~= yaw or l[5] ~= pitch or l[6] ~= roll then
				l[1], l[2], l[3], l[4], l[5], l[6] = x, y, z, yaw, pitch, roll
				n = n + 1
				ids[n] = id
				local p = n * 3
				pos[p - 2], pos[p - 1], pos[p] = x, y, z
				rot[p - 2], rot[p - 1], rot[p] = yaw, pitch, roll
			end
		end
	end
	if n > 0 then
		record("U", packU32(frame, n) .. packU16(ids) .. packS32(pos) .. packS16(rot))
		stats.unit_rows = stats.unit_rows + n
	end
end

-- `V`: u32 frame, u32 n, then n rows of five f32: unit id, health, build progress (0 to 1), paralysis damage, flags
-- (1 cloaked). A unit's row is written when any of them changed; its full health is in its `created` row.
local function vitalRow(id, rows, n)
	local health, _, paralyze, _, build = spGetUnitHealth(id)
	if not health then return n end
	local flags = 0
	local defID = known[id]
	if cloaks[defID] and spGetUnitIsCloaked(id) then flags = 1 end
	health, paralyze = round(health), round(paralyze or 0)
	build = round((build or 1) * 1000) / 1000
	local v = vital[id]
	if not v then
		v = {}
		vital[id] = v
	end
	if v[1] ~= health or v[2] ~= build or v[3] ~= paralyze or v[4] ~= flags then
		v[1], v[2], v[3], v[4] = health, build, paralyze, flags
		local k = n * 5
		rows[k + 1], rows[k + 2], rows[k + 3], rows[k + 4], rows[k + 5] = id, health, build, paralyze, flags
		return n + 1
	end
	return n
end

local function sampleVitals(frame, everyone)
	local rows, n = {}, 0
	if everyone then
		for i = 1, allCount do n = vitalRow(all[i], rows, n) end
		dirty = {}
	else
		for id in pairs(dirty) do
			if known[id] then n = vitalRow(id, rows, n) end
		end
		dirty = {}
	end
	if n > 0 then
		record("V", packU32(frame, n) .. packF32(rows))
		stats.vital_rows = stats.vital_rows + n
	end
end

-- `S`: u32 frame, u32 n, then n rows of three u16: unit id, the ally teams that have it in sight (bit `a` for ally
-- team a) and those that have it on radar. Every VITALS frames, a unit's row when it changed. Two masks of sixteen
-- bits, not one of thirty-two: the engine's Lua numbers are 32-bit floats, exact to 2^24.
local spGetUnitLosState = Spring.GetUnitLosState
local sightTeams = {} -- the ally teams that play (gaia's sees nothing worth telling)
local function sampleSight(frame)
	local rows, n = {}, 0
	for i = 1, allCount do
		local id = all[i]
		local los, radar = 0, 0
		for k = 1, #sightTeams do
			local a = sightTeams[k]
			-- (the raw bits: 1 in sight, 2 on radar; this widget reads as a spectator does, so any ally team answers)
			local state = spGetUnitLosState(id, a, true) or 0
			if state % 2 == 1 then los = los + 2 ^ a end
			if floor(state / 2) % 2 == 1 then radar = radar + 2 ^ a end
		end
		local s = sight[id]
		if not s or s[1] ~= los or s[2] ~= radar then
			sight[id] = { los, radar }
			rows[n * 3 + 1], rows[n * 3 + 2], rows[n * 3 + 3] = id, los, radar
			n = n + 1
		end
	end
	if n > 0 then
		record("S", packU32(frame, n) .. packU16(rows))
		stats.sight_rows = stats.sight_rows + n
	end
end

-- `O`: u32 frame, u32 n, then n rows of eight f32: unit id, the head order's command id (0 none; negative: a build of
-- that type), how the order points (0 nowhere, 1 at the unit or feature `a`, 3 at the place a, b, c), a, b, c, the
-- unit it is building (0 none) and the queue's length. Written when it changed. (Until raw version 2 the last two
-- were one number, 65536 x the unit + the queue: the engine's Lua numbers are 32-bit floats, so the queue was lost
-- whenever the unit's id was over 127.)
--
-- Repair, reclaim, resurrect and capture take five numbers when a builder working an area is on one target: its id,
-- then the area (BuilderCAI.cpp). That is written as the target (`how` 1); until 2026-11-15 night it was written as
-- the place of its first three numbers, which the reader sorts out (crates/tape/src/read.rs, `aim_of`).
local AREA_TARGET = { [40] = true, [90] = true, [125] = true, [130] = true }
local function sampleActivity(frame)
	local rows, n = {}, 0
	for i = 1, allCount do
		local id = all[i]
		local cmd, _, _, p1, p2, p3, _, p5 = spGetUnitCurrentCommand(id)
		local queue = spGetCommandQueue(id, 0) or 0
		local how, a, b, c = 0, 0, 0, 0
		if cmd and p5 and AREA_TARGET[cmd] then
			how, a = 1, round(p1)
		elseif cmd and p3 then
			how, a, b, c = 3, round(p1), round(p2), round(p3)
		elseif cmd and p1 then
			how, a = 1, round(p1)
		end
		local builds = 0
		if UnitDefs[known[id]].buildSpeed > 0 then builds = spGetUnitIsBuilding(id) or 0 end
		cmd = cmd or 0
		if queue > 65535 then queue = 65535 end
		local d = doing[id]
		if not d then
			d = {}
			doing[id] = d
		end
		if d[1] ~= cmd or d[2] ~= a or d[3] ~= b or d[4] ~= c or d[5] ~= queue or d[6] ~= builds then
			d[1], d[2], d[3], d[4], d[5], d[6] = cmd, a, b, c, queue, builds
			local k = n * 8
			rows[k + 1], rows[k + 2], rows[k + 3], rows[k + 4], rows[k + 5], rows[k + 6], rows[k + 7], rows[k + 8] = id, cmd, how, a, b, c, builds, queue
			n = n + 1
		end
	end
	if n > 0 then record("O", packU32(frame, n) .. packF32(rows)) end
end

-- `A`: u32 frame, u32 n, then n rows of four s16: unit id, weapon number (from 1), and where the weapon points in
-- the world: its heading and its pitch, 1024ths of a turn. Written when either changed.
local function sampleAim(frame)
	local rows, n = {}, 0
	for id, weapons in pairs(armed) do
		for w = 1, weapons do
			local _, _, _, dx, dy, dz = spGetUnitWeaponVectors(id, w)
			if dx then
				local key = id * 16 + w
				local yaw, pitch = turns(atan2(dx, dz), AIM), turns(asin(dy > 1 and 1 or (dy < -1 and -1 or dy)), AIM)
				local packed = yaw * 4096 + pitch
				if aimLast[key] ~= packed then
					aimLast[key] = packed
					local k = n * 4
					rows[k + 1], rows[k + 2], rows[k + 3], rows[k + 4] = id, w, yaw, pitch
					n = n + 1
				end
			end
		end
	end
	if n > 0 then
		record("A", packU32(frame, n) .. packS16(rows))
		stats.aim_rows = stats.aim_rows + n
	end
end

------------------------------------------------------------------------------------------------------------------------
-- Projectiles, by asking the engine every frame which exist: the game hands a widget no event when one is made or ends.

-- `P`: u32 frame, u32 n, then n rows of sixteen f32: projectile id, weapon type (-1: a piece of a dying unit, -2
-- neither), owner unit (-1), team (-1), position x y z, velocity x y z (elmos a frame), gravity (elmos a frame a frame),
-- frames it has left to live (-1 unknown), how it is aimed (0 nothing, 1 the ground at a b c, 2 the unit a, 3 the
-- feature a, 4 the projectile a), a, b, c.
-- `Q`: u32 frame, u32 n, then n rows of four f32: projectile id and where it is. Every SAMPLE frames for the ones that
-- steer; every CHECK frames for the rest, whose path the reader works out from the start (the check of that working).
-- `E`: u32 frame, u32 n, then n projectile ids (u32): listed the frame before, gone this one.
local AIMED = { [string.byte("g")] = 1, [string.byte("u")] = 2, [string.byte("f")] = 3, [string.byte("p")] = 4 }

local function sampleProjectiles(frame)
	poll = poll + 1
	local list = spGetProjectilesInRectangle(-8192, -8192, Game.mapSizeX + 8192, Game.mapSizeZ + 8192)
	local fresh, f = {}, 0
	local reused, r = {}, 0
	for i = 1, #list do
		local id = list[i]
		-- The engine reuses a projectile's id as soon as it is gone (within two frames 335 times in the Erebos 10v8's
		-- 281,852), and one gone and made again between two looks is still on the list: told apart by its owner and
		-- weapon type, which the old one ends on and the new one begins from.
		local maker = (spGetProjectileOwnerID(id) or -1) * 65536 + (spGetProjectileDefID(id) or -1)
		if seen[id] and madeBy[id] ~= maker then
			r = r + 1
			reused[r] = id
			seen[id], flies[id] = nil, nil
			liveCount = drop(live, liveAt, liveCount, id)
			stats.same_frame_reuse = stats.same_frame_reuse + 1
		end
		if not seen[id] then
			madeBy[id] = maker
			liveCount = add(live, liveAt, liveCount, id)
			local isWeapon, isPiece = spGetProjectileType(id)
			local wdef = -2
			if isWeapon then
				wdef = weapon(spGetProjectileDefID(id) or -2)
			elseif isPiece then
				wdef = -1
				stats.piece_projectiles = stats.piece_projectiles + 1
			end
			flies[id] = flown[wdef] or nil
			local x, y, z = spGetProjectilePosition(id)
			local vx, vy, vz = spGetProjectileVelocity(id)
			local how, a, b, c = 0, 0, 0, 0
			local ttl = -1
			if isWeapon then
				local kind, at = spGetProjectileTarget(id)
				how = AIMED[kind] or 0
				if how == 1 and type(at) == "table" then
					a, b, c = at[1], at[2], at[3]
				elseif how > 1 then
					a = at or 0
				end
				ttl = spGetProjectileTimeToLive(id) or -1
			end
			local k = f * 16
			fresh[k + 1], fresh[k + 2], fresh[k + 3], fresh[k + 4] = id, wdef, spGetProjectileOwnerID(id) or -1, spGetProjectileTeamID(id) or -1
			fresh[k + 5], fresh[k + 6], fresh[k + 7], fresh[k + 8], fresh[k + 9], fresh[k + 10] = x or 0, y or 0, z or 0, vx or 0, vy or 0, vz or 0
			fresh[k + 11], fresh[k + 12], fresh[k + 13], fresh[k + 14], fresh[k + 15], fresh[k + 16] = spGetProjectileGravity(id) or 0, ttl, how, a, b, c
			f = f + 1
			if endedAt[id] and frame - endedAt[id] <= 2 then stats.quick_reuse = stats.quick_reuse + 1 end
		end
		seen[id] = poll
	end
	if r > 0 then
		-- before the new ones' rows: a reader ends the old projectile of an id, then meets the new
		record("E", packU32(frame, r) .. packU32(reused))
		stats.projectile_ended = stats.projectile_ended + r
	end
	if f > 0 then
		record("P", packU32(frame, f) .. packF32(fresh))
		stats.projectile_new = stats.projectile_new + f
	end
	local ended, e = {}, 0
	local rows, n = {}, 0
	local onSample, onCheck = frame % SAMPLE == 0, frame % CHECK == 0
	local i = 1
	while i <= liveCount do
		local id = live[i]
		if seen[id] ~= poll then
			e = e + 1
			ended[e] = id
			seen[id], flies[id], madeBy[id] = nil, nil, nil
			endedAt[id] = frame
			liveCount = drop(live, liveAt, liveCount, id)
		else
			if (onSample and flies[id]) or onCheck then
				local x, y, z = spGetProjectilePosition(id)
				if x then
					local k = n * 4
					rows[k + 1], rows[k + 2], rows[k + 3], rows[k + 4] = id, x, y, z
					n = n + 1
				end
			end
			i = i + 1
		end
	end
	if e > 0 then
		record("E", packU32(frame, e) .. packU32(ended))
		stats.projectile_ended = stats.projectile_ended + e
	end
	if n > 0 then
		record("Q", packU32(frame, n) .. packF32(rows))
		stats.projectile_rows = stats.projectile_rows + n
	end
	if liveCount > stats.projectile_peak then stats.projectile_peak = liveCount end
end

-- `X` (explosions) and `B` (muzzle fire), as BAR's explosion-lights gadget reports them to widgets: u32 frame, u32 n,
-- then n rows of five f32: x, y, z, weapon type, owner unit.
local function flushBursts(frame)
	if #explosions > 0 then
		record("X", packU32(frame, #explosions / 5) .. packF32(explosions))
		stats.explosions = stats.explosions + #explosions / 5
		explosions = {}
	end
	if #barrels > 0 then
		record("B", packU32(frame, #barrels / 5) .. packF32(barrels))
		stats.barrels = stats.barrels + #barrels / 5
		barrels = {}
	end
end

function widget:VisibleExplosion(x, y, z, weaponID, ownerID)
	if not out then return end
	local n = #explosions
	explosions[n + 1], explosions[n + 2], explosions[n + 3], explosions[n + 4], explosions[n + 5] = x, y, z, weapon(weaponID or -1) or -1, ownerID or -1
end

function widget:Barrelfire(x, y, z, weaponID, ownerID)
	if not out then return end
	local n = #barrels
	barrels[n + 1], barrels[n + 2], barrels[n + 3], barrels[n + 4], barrels[n + 5] = x, y, z, weapon(weaponID or -1) or -1, ownerID or -1
end

------------------------------------------------------------------------------------------------------------------------
-- Features: all of them at the start, then as they come and go.

local function featureRow(id, kind)
	local defID = Spring.GetFeatureDefID(id)
	if not defID then return end
	featureDef(defID)
	local x, y, z = Spring.GetFeaturePosition(id)
	local fx, fy, fz, _, _, _, ux, uy, uz = Spring.GetFeatureDirection(id)
	local metal, _, energy = Spring.GetFeatureResources(id)
	put({ t = "ev", k = kind, f = frameNow, id = id, d = defID, team = Spring.GetFeatureTeam(id) or -1, p = { x or 0, y or 0, z or 0 },
		front = { fx or 0, fy or 0, fz or 1 }, up = { ux or 0, uy or 1, uz or 0 }, metal = metal or 0, energy = energy or 0 })
end

function widget:FeatureCreated(id)
	if out then featureRow(id, "feature") end
end

function widget:FeatureDestroyed(id)
	if out then put({ t = "ev", k = "feature_gone", f = frameNow, id = id }) end
end

------------------------------------------------------------------------------------------------------------------------

local function timed(name, started)
	stats[name] = stats[name] + spDiffTimers(spGetTimer(), started)
end

local function resources(frame)
	local rows = {}
	for _, team in ipairs(Spring.GetTeamList()) do
		local m, ms, mp, mi, me = Spring.GetTeamResources(team, "metal")
		local e, es, ep, ei, ee = Spring.GetTeamResources(team, "energy")
		if m then rows[#rows + 1] = { team, round(m), round(ms), round(mp * 10) / 10, round(mi * 10) / 10, round(me * 10) / 10, round(e), round(es), round(ep * 10) / 10, round(ei * 10) / 10, round(ee * 10) / 10 } end
	end
	put({ t = "res", f = frame, teams = rows })
end

function widget:Initialize()
	if not VFS.FileExists(OUT .. "tape.on") then
		widgetHandler:RemoveWidget(self)
		return
	end
	local spectating, fullView = Spring.GetSpectatingState()
	if not (spectating and fullView) then
		Spring.Echo("<tape dump> not a spectator with full view: nothing written")
		widgetHandler:RemoveWidget(self)
		return
	end
	aimOn = VFS.FileExists(OUT .. "tape.aim")
	skipOn = VFS.FileExists(OUT .. "records.off")
	out = io.open(OUT .. "tape.raw", "wb")
	if not out then
		Spring.Echo("<tape dump> cannot open " .. OUT .. "tape.raw")
		widgetHandler:RemoveWidget(self)
		return
	end
	wallStart = spGetTimer()
	local listed = io.open(OUT .. "assets.have", "r")
	if listed then
		for name in listed:lines() do have[name] = true end
		listed:close()
	end
	-- A lobby demo opens with the minutes its players took to load, and the engine plays them at the server's speed.
	if skipOn then Spring.SendCommands("setmaxspeed 1000", "setminspeed 1000") end
	local gaia = Spring.GetGaiaTeamID()
	local gaiaAlly = select(6, Spring.GetTeamInfo(gaia, false))
	for _, a in ipairs(Spring.GetAllyTeamList()) do
		if a ~= gaiaAlly and a < 16 then sightTeams[#sightTeams + 1] = a end
	end
	local teams = {}
	for _, team in ipairs(Spring.GetTeamList()) do
		local _, leader, _, isAI, side, ally = Spring.GetTeamInfo(team, false)
		local name = ""
		if team == gaia then
			name = "gaia"
		elseif isAI then
			local _, aiName, _, shortName, version = Spring.GetAIInfo(team)
			name = (shortName or aiName or "ai") .. " " .. (version or "")
		else
			name = Spring.GetPlayerInfo(leader, false) or ""
		end
		local r, g, b = Spring.GetTeamColor(team)
		local sx, sy, sz = Spring.GetTeamStartPosition(team)
		teams[#teams + 1] = { team = team, ally = ally, side = side or "", name = name, ai = isAI and 1 or 0, gaia = team == gaia and 1 or 0,
			color = { r or 1, g or 1, b or 1 }, start = { sx or 0, sy or 0, sz or 0 } }
	end
	local spots = {}
	for i = 1, Spring.GetGameRulesParam("mex_count") or 0 do
		spots[#spots + 1] = { Spring.GetGameRulesParam("mex_x" .. i) or 0, Spring.GetGameRulesParam("mex_z" .. i) or 0, Spring.GetGameRulesParam("mex_metal" .. i) or 0 }
	end
	local low, high = Spring.GetGroundExtremes()
	put({
		t = "header", format = "within-reason-tape-raw", version = 2, live = Spring.IsReplay() and 0 or 1, max_units = Game.maxUnits, ally_teams = sightTeams, engine = Engine and Engine.version or "", game = Game.gameName .. " " .. Game.gameVersion,
		sample_frames = SAMPLE, vitals_frames = VITALS, activity_frames = ACTIVITY, check_frames = CHECK, position_scale = POS, angle_scale = ANGLE, aim_scale = AIM,
		aim = aimOn and 1 or 0,
		map = { name = Game.mapName, width = Game.mapSizeX, height = Game.mapSizeZ, gravity = Game.gravity, wind_min = Game.windMin, wind_max = Game.windMax,
			tidal = Game.tidal, low = low, high = high, water_damage = Game.waterDamage },
		teams = teams, spots = spots,
	})
	-- The map's own files: its height and tile-index file and the tile files that one names.
	local mapFiles = {}
	for _, pattern in ipairs({ "*.smf", "*.smt" }) do
		for _, path in ipairs(VFS.DirList("maps/", pattern, VFS.ZIP) or {}) do
			if asset(path) then mapFiles[#mapFiles + 1] = string.lower(path) end
		end
	end
	put({ t = "map_files", files = mapFiles })
	ground(0, 0, Game.mapSizeX / 8, Game.mapSizeZ / 8)
	for _, id in ipairs(Spring.GetAllFeatures()) do featureRow(id, "feature") end
	Spring.Echo("<tape dump> " .. #teams .. " teams, " .. bytes .. " bytes of ground and features")
end

function widget:GameStart()
	-- The replay dump widget asks for the skip when it runs; alone, this one does (run/replay_match.py's `records.off`).
	-- The speed first: a headless engine sleeps after every frame to hold the game's speed factor (Game.cpp, HEADLESS),
	-- so at the demo's factor of 1 a skip runs at twice game speed however idle the machine is.
	if out and skipOn then Spring.SendCommands("setmaxspeed 1000", "setminspeed 1000", "skip f100000000") end
end

function widget:GameFrame(frame)
	if not out then return end
	frameNow = frame
	local t = spGetTimer()
	if allCount == 0 or frame % 300 == 0 then
		-- a unit that stood before this widget loaded, or whose making no call-in told of
		for _, id in ipairs(Spring.GetAllUnits()) do
			if not known[id] then meet(id) end
		end
	end
	if frame % SAMPLE == 0 then sampleUnits(frame) end
	timed("units", t)
	t = spGetTimer()
	if frame % SAMPLE == 0 then sampleVitals(frame, frame % VITALS == 0) end
	timed("vitals", t)
	t = spGetTimer()
	if frame % ACTIVITY == 0 then sampleActivity(frame) end
	timed("activity", t)
	if frame % VITALS == 0 then
		t = spGetTimer()
		sampleSight(frame)
		timed("sight", t)
	end
	if aimOn and frame % SAMPLE == 0 then
		t = spGetTimer()
		sampleAim(frame)
		timed("aim", t)
	end
	t = spGetTimer()
	sampleProjectiles(frame)
	flushBursts(frame)
	timed("projectiles", t)
	if frame % RESOURCES == 0 then resources(frame) end
	-- Written out just past each slice's end, the frame that makes it whole for the arena's follower (the tape that
	-- grows while the game is played).
	if frame % 300 == 1 then out:flush() end
	if frame % 1800 == 0 then
		Spring.Echo(string.format("<tape dump> f=%d wall=%.0fs units=%d mobile=%d projectiles=%d bytes=%d", frame, spDiffTimers(spGetTimer(), wallStart), allCount, mobileCount, liveCount, bytes))
	end
end

function widget:UnitCreated(id, defID, team, builder)
	if out then meet(id, defID, team, builder) end
end

function widget:UnitFinished(id, defID, team)
	if not out then return end
	meet(id, defID, team)
	put({ t = "ev", k = "finished", f = frameNow, u = id })
	dirty[id] = true
end

function widget:UnitDestroyed(id, defID, team, attacker, attackerDef, attackerTeam)
	if not out then return end
	local x, y, z = spGetUnitPosition(id)
	local _, _, _, _, build = spGetUnitHealth(id)
	put({ t = "ev", k = "destroyed", f = frameNow, u = id, by = attacker or -1, by_team = attackerTeam or -1,
		p = { round((x or 0) * POS), round((y or 0) * POS), round((z or 0) * POS) }, build = build or 1 })
	forget(id)
end

function widget:UnitGiven(id, defID, newTeam, oldTeam)
	if not out then return end
	meet(id, defID, newTeam)
	put({ t = "ev", k = "given", f = frameNow, u = id, team = newTeam, from = oldTeam })
end

function widget:UnitDamaged(id)
	if out then dirty[id] = true end
end

function widget:UnitCloaked(id)
	if out then dirty[id] = true end
end

function widget:UnitDecloaked(id)
	if out then dirty[id] = true end
end

function widget:UnitCommand(id, defID, team, cmdID, params, opts)
	if not out then return end
	stats.cmds = stats.cmds + 1
	local p = {}
	if params then
		for i = 1, math.min(#params, 6) do p[i] = round(params[i]) end
	end
	put({ t = "cmd", f = frameNow, u = id, c = cmdID, p = p, shift = (opts and opts.shift) and 1 or 0 })
end

function widget:UnsyncedHeightMapUpdate(x1, z1, x2, z2)
	if out and frameNow > 0 then ground(x1, z1, x2, z2) end
end

function widget:AddConsoleLine(line)
	if not out then return end
	local name, text = line:match("^<([^>]+)> (.*)$")
	if name then put({ t = "ev", k = "chat", f = frameNow, player = name, text = text }) end
end

local function close(winners)
	if not out then return end
	stats.t = "stats"
	stats.frames = frameNow
	stats.bytes = bytes
	stats.wall = spDiffTimers(spGetTimer(), wallStart)
	put(stats)
	put({ t = "over", f = frameNow, winners = winners or {} })
	out:close()
	out = nil
	Spring.Echo(string.format("<tape dump> closed at f=%d: %d bytes; seconds in units %.1f, vitals %.1f, activity %.1f, sight %.1f, aim %.1f, projectiles %.1f; %d marks",
		frameNow, bytes, stats.units, stats.vitals, stats.activity, stats.sight, stats.aim, stats.projectiles, stats.marks))
end

-- The players' marks on the map: a point with its words, a line, an erase (which takes away what lies within its
-- radius). A spectator's widget is told of every player's (InMapDrawModel.cpp, AllowedMsg); never taken from the game.
function widget:MapDrawCmd(playerID, kind, x, y, z, a, b, c)
	if not out then return false end
	local name, _, spectator, team, ally = Spring.GetPlayerInfo(playerID, false)
	local row = { t = "ev", k = "mark", f = Spring.GetGameFrame(), player = name or "", spectator = spectator and 1 or 0, team = team or -1,
		ally = ally or -1, kind = kind, p = { round(x or 0), round(y or 0), round(z or 0) } }
	if kind == "point" then
		row.label = a or ""
	elseif kind == "line" then
		row.to = { round(a or 0), round(b or 0), round(c or 0) }
	elseif kind == "erase" then
		row.radius = a or 100
	end
	put(row)
	stats.marks = stats.marks + 1
	return false
end

function widget:GameOver(winners)
	close(winners)
end

function widget:Shutdown()
	close()
end
