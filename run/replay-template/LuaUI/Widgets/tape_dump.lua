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
local ANGLE = 1024 -- an attitude's angle is whole 1024ths of a turn (0.35 degrees)

local spGetUnitPosition = Spring.GetUnitPosition
local spGetUnitDirection = Spring.GetUnitDirection
local spGetUnitHealth = Spring.GetUnitHealth
local spGetUnitDefID = Spring.GetUnitDefID
local spGetUnitTeam = Spring.GetUnitTeam
local spGetUnitIsCloaked = Spring.GetUnitIsCloaked
local spGetUnitCurrentCommand = Spring.GetUnitCurrentCommand
local spGetCommandQueue = Spring.GetCommandQueue
local spGetUnitIsBuilding = Spring.GetUnitIsBuilding
local spGetUnitCurrentBuildPower = Spring.GetUnitCurrentBuildPower
local spGetFactoryCommands = Spring.GetFactoryCommands
local spGetUnitExperience = Spring.GetUnitExperience
local spGetFeatureResources = Spring.GetFeatureResources
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
local spGetGameFrame = Spring.GetGameFrame
local spGetUnitIsDead = Spring.GetUnitIsDead
local spGetUnitPieceMatrix = Spring.GetUnitPieceMatrix
local spDiffTimers = Spring.DiffTimers
local packU16, packU32, packS16, packS32, packF32 = VFS.PackU16, VFS.PackU32, VFS.PackS16, VFS.PackS32, VFS.PackF32
local floor, atan2, asin, sqrt = math.floor, math.atan2, math.asin, math.sqrt
local TURN = 2 * math.pi

local out
local piecesOn = false -- the moving pieces (`K` rows), with the marker tape.pieces
local skipOn = false
local frameNow = 0 -- the frame of the last GameFrame; a call-in's row is stamped with the engine's own (spGetGameFrame)
local bytes = 0

-- Units. `mobile` is the list sampled every SAMPLE frames; a building is written once, when it appears.
local known = {} -- unit id -> def id
local mobile, mobileAt, mobileCount = {}, {}, 0
local all, allAt, allCount = {}, {}, 0
local last = {} -- unit id -> { x, y, z, yaw, pitch, roll } as last written
local vital = {} -- unit id -> { health, build, paralyze, flags } as last written
local sight = {} -- unit id -> { in sight, on radar } as last written (the `S` rows)
local doing = {} -- unit id -> { cmd, a, b, c, queue, builds } as last written
local work = {} -- builder id -> { build power in hundredths, a factory's line } as last written (the `W` rows)
local rank = {} -- unit id -> { experience, full health } as last written (the `R` rows)
local hurt = {} -- unit id -> damage taken since the last `H` row
local featureMetal = {} -- feature id -> its metal (whole) as last written
local early = {} -- unit id -> the commands it was given before the tape met it (`UnitCommand`)
local pieceState = {} -- unit id -> its pieces' last model-space matrices and keyframes (`samplePieces`)
local dirty = {} -- unit ids hit since the last sample: their health is read at the next
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

local stats = { units = 0, vitals = 0, activity = 0, sight = 0, projectiles = 0, events = 0, sight_rows = 0, marks = 0, unit_rows = 0, vital_rows = 0,
	work_rows = 0, rank_rows = 0, hurt_rows = 0, pieces = 0, key_rows = 0,
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

-- A type's reach as the audit counts it: its longest weapon that is no shield and does damage.
local function auditReach(d)
	local reach = 0
	for _, w in ipairs(d.weapons or {}) do
		local wd = WeaponDefs[w.weaponDef]
		if wd and (wd.range or 0) > reach and not wd.isShield and (wd.damages and (wd.damages[0] or 0) > 1 or wd.customParams) then reach = wd.range end
	end
	return reach
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
		-- (for the audit, as audit_dump.lua writes them)
		power = d.power or 0, energy_make = d.energyMake or 0, energy_upkeep = d.energyUpkeep or 0, wind = d.windGenerator or 0,
		tidal = d.tidalGenerator or 0, metal_store = d.metalStorage or 0, energy_store = d.energyStorage or 0, can_assist = d.canAssist and 1 or 0,
		can_reclaim = d.canReclaim and 1 or 0, converter = tonumber(cp.energyconv_capacity) or 0, reach = auditReach(d),
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
		record("G", packU32(spGetGameFrame(), x1, row, width, rows) .. packF32(heights))
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
	local x, y, z = spGetUnitPosition(id)
	local fx, fy, fz, _, _, _, ux, uy, uz = spGetUnitDirection(id)
	local health, maxHealth, _, _, build = spGetUnitHealth(id)
	local yaw, pitch, roll = attitude(fx or 0, fy or 0, fz or 1, ux or 0, uy or 1, uz or 0)
	put({ t = "ev", k = "created", f = spGetGameFrame(), u = id, d = defID, team = team or spGetUnitTeam(id), by = builder or -1,
		p = { round((x or 0) * POS), round((y or 0) * POS), round((z or 0) * POS) }, rot = { yaw, pitch, roll },
		health = health or 0, max_health = maxHealth or 0, build = build or 1 })
	for _, row in ipairs(early[id] or {}) do put(row) end
	early[id] = nil
	-- (a unit can be made with experience: then its `R` row now, else the reader takes none)
	local xp = spGetUnitExperience(id) or 0
	rank[id] = { xp, round(maxHealth or 0) }
	if xp > 0 then record("R", packU32(spGetGameFrame(), 1) .. packF32({ id, xp, round(maxHealth or 0) })) end
	dirty[id] = true
end

local function forget(id)
	known[id] = nil
	allCount = drop(all, allAt, allCount, id)
	mobileCount = drop(mobile, mobileAt, mobileCount, id)
	last[id], vital[id], doing[id], dirty[id], sight[id] = nil, nil, nil, nil, nil
	work[id], rank[id], hurt[id], early[id], pieceState[id] = nil, nil, nil, nil, nil
end

------------------------------------------------------------------------------------------------------------------------
-- The moving pieces (raw version 4, with the marker tape.pieces; docs/design/2026-11-15-game-tape.md section 9). A unit
-- type's pieces as the engine has them, once, in a `pieces` row: names, each one's parent (its index from 1, 0 for the
-- root) and offset from its parent at rest. Then every SAMPLE frames, for each piece whose model-space matrix
-- (`GetUnitPieceMatrix`) or whose parent's changed, its turn and place against its parent: the script's own angles
-- about x, y and z (the engine builds a piece's rotation as `CQuaternion::FromEulerYPR` of them, and
-- `CQuaternion::ToEulerYPR` gives them back: pitch = asin(-R12), yaw = atan2(R02, R22), roll = atan2(R10, R11), R the
-- rotation against the parent), unwound so that a spin keeps counting, and the place (the offset plus the script's
-- move). A unit script turns and moves at set speeds, so each number runs in straight stretches: a keyframe is written
-- only where one bends, the sample before lying off the straight line from the last keyframe to the new sample by more
-- than A_TOL or T_TOL. A unit's first sample writes a keyframe for each piece not at rest. Every PIECE_SAMPLE frames.
-- `K`: u32 n, then n frames (u32), n unit ids (u16), n piece numbers (u16, from 1), 3n angles (s32, A_SCALE a radian:
-- pitch, yaw, roll) and 3n place components (s16, T_SCALE an elmo). A ninth of the tolerance below and a quarter of an
-- elmo: finer numbers only made the tape bigger (docs/design/2026-11-15-game-tape.md section 10).
local A_SCALE, T_SCALE = 1024, 4
local A_TOL, T_TOL = 0.0087, 0.25 -- half a degree, a quarter of an elmo
local PIECE_SAMPLE = 6 -- frames between looks at the pieces: a walking unit's legs bend at every look
local SLICE = 300 -- the tape's slice (crates/tape SLICE_FRAMES): every piece that moved is a keyframe at its end
local TWO_PI = 2 * math.pi
local pieceTypes = {} -- def id -> { n, parent = {}, ox = {}, oy = {}, oz = {} }
local keyRows = { f = {}, u = {}, p = {}, a = {}, t = {}, n = 0 }
local function piecesOf(id, defID)
	local t = pieceTypes[defID]
	if t then return t end
	local names = Spring.GetUnitPieceList(id) or {}
	local index = {}
	for i, name in ipairs(names) do index[name] = i end
	t = { n = #names, parent = {}, ox = {}, oy = {}, oz = {} }
	local parents, offsets = {}, {}
	for i = 1, #names do
		local info = Spring.GetUnitPieceInfo(id, i) or {}
		local o = info.offset or { 0, 0, 0 }
		t.parent[i] = index[info.parent] or 0
		t.ox[i], t.oy[i], t.oz[i] = o[1] or 0, o[2] or 0, o[3] or 0
		parents[i], offsets[i] = t.parent[i], { o[1] or 0, o[2] or 0, o[3] or 0 }
	end
	pieceTypes[defID] = t
	put({ t = "pieces", d = defID, names = names, parents = parents, offsets = offsets })
	return t
end
local function keyRow(frame, id, p, ax, ay, az, x, y, z)
	local k = keyRows
	local n = k.n + 1
	k.n = n
	k.f[n], k.u[n], k.p[n] = frame, id, p
	local i = n * 3
	k.a[i - 2], k.a[i - 1], k.a[i] = round(ax * A_SCALE), round(ay * A_SCALE), round(az * A_SCALE)
	k.t[i - 2], k.t[i - 1], k.t[i] = round(x * T_SCALE), round(y * T_SCALE), round(z * T_SCALE)
end
-- Whether two of a piece's six numbers as written differ (`kq` its last keyframe's, `sq` its last sample's: a piece
-- looked at only because its parent moved may not have moved itself).
local function apart(a, b, Q)
	for j = 1, 6 do
		if a[Q + j] ~= b[Q + j] then return true end
	end
	return false
end
-- A keyframe of a piece, after the one its stretch starts from when that was not written (a piece at rest when its
-- unit was first looked at, which moves later): the reader draws a straight stretch between written keyframes.
local function bend(st, id, p, frame, ax, ay, az, x, y, z)
	if not st.kw[p] then
		local K = (p - 1) * 7
		local ks = st.k
		keyRow(ks[K + 1], id, p, ks[K + 2], ks[K + 3], ks[K + 4], ks[K + 5], ks[K + 6], ks[K + 7])
		st.kw[p] = true
	end
	keyRow(frame, id, p, ax, ay, az, x, y, z)
end
local abs, asin, atan2q = math.abs, math.asin, math.atan2
-- An angle from atan2 brought within a half turn of the last one: a spin keeps counting.
local function unwound(raw, last)
	local d = raw - last
	if d > math.pi or d < -math.pi then raw = raw - TWO_PI * floor((d + math.pi) / TWO_PI) end
	return raw
end
-- Each piece's last sample as a keyframe where it is not the last one (a unit's death, the capture's end): the reader
-- holds a piece at its last keyframe.
local function flushPieces(id)
	local st = pieceState[id]
	local t = st and pieceTypes[known[id]]
	if not t then return end
	local ks, ss = st.k, st.s
	for p = 1, t.n do
		local K = (p - 1) * 7
		if ss[K + 1] and ss[K + 1] > ks[K + 1] and apart(st.kq, st.sq, (p - 1) * 6) then
			bend(st, id, p, ss[K + 1], ss[K + 2], ss[K + 3], ss[K + 4], ss[K + 5], ss[K + 6], ss[K + 7])
		end
	end
end
local function writeKeys()
	local k = keyRows
	if k.n > 0 then
		record("K", packU32(k.n) .. packU32(k.f) .. packU16(k.u) .. packU16(k.p) .. packS32(k.a) .. packS16(k.t))
		stats.key_rows = stats.key_rows + k.n
		keyRows = { f = {}, u = {}, p = {}, a = {}, t = {}, n = 0 }
	end
end
local function samplePieces(frame)
	for i = 1, allCount do
		local id = all[i]
		local defID = known[id]
		local t = pieceTypes[defID] or piecesOf(id, defID)
		local n = t.n
		local st = pieceState[id]
		local fresh = not st
		if fresh then
			st = { ms = {}, k = {}, s = {}, c = {}, w = {}, kw = {}, kq = {}, sq = {} }
			pieceState[id] = st
		end
		local ms, ks, ss, ch, was, kq, sq = st.ms, st.k, st.s, st.c, st.w, st.kq, st.sq
		-- Which pieces moved in model space.
		local any = fresh
		for p = 1, n do
			local m1, m2, m3, _, m5, m6, m7, _, m9, m10, m11, _, m13, m14, m15 = spGetUnitPieceMatrix(id, p)
			local b = (p - 1) * 12
			if m1 and (fresh or ms[b + 1] ~= m1 or ms[b + 2] ~= m2 or ms[b + 3] ~= m3 or ms[b + 4] ~= m5 or ms[b + 5] ~= m6 or ms[b + 6] ~= m7
				or ms[b + 7] ~= m9 or ms[b + 8] ~= m10 or ms[b + 9] ~= m11 or ms[b + 10] ~= m13 or ms[b + 11] ~= m14 or ms[b + 12] ~= m15) then
				ms[b + 1], ms[b + 2], ms[b + 3], ms[b + 4], ms[b + 5], ms[b + 6], ms[b + 7], ms[b + 8], ms[b + 9], ms[b + 10], ms[b + 11], ms[b + 12] =
					m1, m2, m3, m5, m6, m7, m9, m10, m11, m13, m14, m15
				ch[p] = true
				any = true
			else
				ch[p] = false
			end
		end
		-- (only a unit that had a piece moving at its last look can have one that stopped since)
		for p = 1, st.moving and n or 0 do
			local q = t.parent[p]
			local moved = fresh or ch[p] or (q > 0 and ch[q])
			if not moved and was[p] then
				-- It stopped: its last sample is where it stands, a keyframe unless it is the last one (or written as
				-- the same numbers): what follows a stop never reaches back past it.
				local K, Q = (p - 1) * 7, (p - 1) * 6
				if ss[K + 1] > ks[K + 1] and apart(kq, sq, Q) then
					bend(st, id, p, ss[K + 1], ss[K + 2], ss[K + 3], ss[K + 4], ss[K + 5], ss[K + 6], ss[K + 7])
					for j = 1, 7 do ks[K + j] = ss[K + j] end
					for j = 1, 6 do kq[Q + j] = sq[Q + j] end
				end
			end
			was[p] = moved and not fresh
		end
		if any and not st.moving then
			for p = 1, n do
				local q = t.parent[p]
				was[p] = not fresh and (ch[p] or (q > 0 and ch[q]))
			end
		end
		st.moving = any and not fresh
		if any then
			for p = 1, n do
				local q = t.parent[p]
				if fresh or ch[p] or (q > 0 and ch[q]) then
					-- The piece against its parent: R = Rp^T R, t = Rp^T (t - tp). Columns (a, bb, c), (d, e, f), (g, h, ii).
					local b = (p - 1) * 12
					local a, bb, c, d, e, f, g, h, ii, x, y, z = ms[b + 1], ms[b + 2], ms[b + 3], ms[b + 4], ms[b + 5], ms[b + 6], ms[b + 7], ms[b + 8], ms[b + 9], ms[b + 10], ms[b + 11], ms[b + 12]
					if q > 0 then
						local P = (q - 1) * 12
						local p1, p2, p3, p5, p6, p7, p9, p10, p11 = ms[P + 1], ms[P + 2], ms[P + 3], ms[P + 4], ms[P + 5], ms[P + 6], ms[P + 7], ms[P + 8], ms[P + 9]
						local dx, dy, dz = x - ms[P + 10], y - ms[P + 11], z - ms[P + 12]
						a, bb, c, d, e, f, g, h, ii, x, y, z =
							p1 * a + p2 * bb + p3 * c, p5 * a + p6 * bb + p7 * c, p9 * a + p10 * bb + p11 * c,
							p1 * d + p2 * e + p3 * f, p5 * d + p6 * e + p7 * f, p9 * d + p10 * e + p11 * f,
							p1 * g + p2 * h + p3 * ii, p5 * g + p6 * h + p7 * ii, p9 * g + p10 * h + p11 * ii,
							p1 * dx + p2 * dy + p3 * dz, p5 * dx + p6 * dy + p7 * dz, p9 * dx + p10 * dy + p11 * dz
					end
					-- R12 is the second row's third entry (h), R02 the first's (g), R22 (ii), R10 (bb), R11 (e).
					if h > 1 then h = 1 elseif h < -1 then h = -1 end
					local ax, ay, az = asin(-h), atan2q(g, ii), atan2q(bb, e)
					local K, Q = (p - 1) * 7, (p - 1) * 6
					if not fresh then ay, az = unwound(ay, ss[K + 3]), unwound(az, ss[K + 4]) end
					-- (its six numbers as written: `keyRow`'s rounding)
					local q1, q2, q3 = floor(ax * A_SCALE + 0.5), floor(ay * A_SCALE + 0.5), floor(az * A_SCALE + 0.5)
					local q4, q5, q6 = floor(x * T_SCALE + 0.5), floor(y * T_SCALE + 0.5), floor(z * T_SCALE + 0.5)
					if fresh then
						ks[K + 1], ks[K + 2], ks[K + 3], ks[K + 4], ks[K + 5], ks[K + 6], ks[K + 7] = frame, ax, ay, az, x, y, z
						ss[K + 1], ss[K + 2], ss[K + 3], ss[K + 4], ss[K + 5], ss[K + 6], ss[K + 7] = frame, ax, ay, az, x, y, z
						kq[Q + 1], kq[Q + 2], kq[Q + 3], kq[Q + 4], kq[Q + 5], kq[Q + 6] = q1, q2, q3, q4, q5, q6
						sq[Q + 1], sq[Q + 2], sq[Q + 3], sq[Q + 4], sq[Q + 5], sq[Q + 6] = q1, q2, q3, q4, q5, q6
						st.kw[p] = false
						-- (a piece with no keyframe is read at rest: one not exactly there is a keyframe at once)
						if q1 ~= 0 or q2 ~= 0 or q3 ~= 0 or q4 ~= round(t.ox[p] * T_SCALE) or q5 ~= round(t.oy[p] * T_SCALE) or q6 ~= round(t.oz[p] * T_SCALE) then
							keyRow(frame, id, p, ax, ay, az, x, y, z)
							st.kw[p] = true
						end
					else
						-- The path since the last keyframe: the last sample at its frame and, when the piece was not
						-- looked at since, the same value one look ago; each is a keyframe when it is off the straight
						-- line from the last keyframe to this sample (a bend), and always where a piece that stood
						-- began to move again (standing at its last keyframe, or not looked at), so that a stretch
						-- never spans a stand: a slice cut while the game is played, with the piece standing, would
						-- read it standing where the later keyframe made it creep. A piece standing at its last
						-- keyframe that has not moved (a parent's child, looked at for its parent) has nothing to test.
						local same = q1 == sq[Q + 1] and q2 == sq[Q + 2] and q3 == sq[Q + 3] and q4 == sq[Q + 4] and q5 == sq[Q + 5] and q6 == sq[Q + 6]
						local standing = not apart(kq, sq, Q)
						if not (same and standing) then
							local fs = ss[K + 1]
							local hold = frame - PIECE_SAMPLE
							for pass = 1, fs < hold and 2 or 1 do
								local at = pass == 1 and fs or hold
								local fk = ks[K + 1]
								if at > fk then
									local u = (at - fk) / (frame - fk)
									if not same and (pass == 2 or standing) or abs(ss[K + 2] - (ks[K + 2] + u * (ax - ks[K + 2]))) > A_TOL or abs(ss[K + 3] - (ks[K + 3] + u * (ay - ks[K + 3]))) > A_TOL
										or abs(ss[K + 4] - (ks[K + 4] + u * (az - ks[K + 4]))) > A_TOL or abs(ss[K + 5] - (ks[K + 5] + u * (x - ks[K + 5]))) > T_TOL
										or abs(ss[K + 6] - (ks[K + 6] + u * (y - ks[K + 6]))) > T_TOL or abs(ss[K + 7] - (ks[K + 7] + u * (z - ks[K + 7]))) > T_TOL then
										bend(st, id, p, at, ss[K + 2], ss[K + 3], ss[K + 4], ss[K + 5], ss[K + 6], ss[K + 7])
										ks[K + 1], ks[K + 2], ks[K + 3], ks[K + 4], ks[K + 5], ks[K + 6], ks[K + 7] = at, ss[K + 2], ss[K + 3], ss[K + 4], ss[K + 5], ss[K + 6], ss[K + 7]
										for j = 1, 6 do kq[Q + j] = sq[Q + j] end
									end
								end
							end
						end
						ss[K + 1], ss[K + 2], ss[K + 3], ss[K + 4], ss[K + 5], ss[K + 6], ss[K + 7] = frame, ax, ay, az, x, y, z
						sq[Q + 1], sq[Q + 2], sq[Q + 3], sq[Q + 4], sq[Q + 5], sq[Q + 6] = q1, q2, q3, q4, q5, q6
						-- At a slice's end a piece that moved since its last keyframe is one, so that nothing in a slice
						-- hangs on a keyframe after it: a tape that grows while the game is played cuts a slice as
						-- soon as the stream has passed its end, and a slow turn's next keyframe may be far off.
						if frame % SLICE == 0 and ks[K + 1] < frame and apart(kq, sq, Q) then
							bend(st, id, p, frame, ax, ay, az, x, y, z)
							ks[K + 1], ks[K + 2], ks[K + 3], ks[K + 4], ks[K + 5], ks[K + 6], ks[K + 7] = frame, ax, ay, az, x, y, z
							for j = 1, 6 do kq[Q + j] = sq[Q + j] end
						end
					end
				end
			end
		end
	end
	writeKeys()
end

-- `U`: u32 frame, u32 n, then n unit ids (u16), 3n position components (s32, quarters of an elmo) and 3n angles
-- (s16: heading, pitch, roll, 1024ths of a turn). Only the units that moved or turned since their last row.
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

-- What the waste audit reads (run/audit.py, docs/harness/audit.md) beside the head order, every ACTIVITY frames, a row
-- for each unit whose numbers changed (raw version 3).
-- `W`: u32 frame, u32 n, then n rows of three u16: a builder's id, its build power in use in hundredths of its build
-- speed (`GetUnitCurrentBuildPower`: what the economy lets it spend) and a factory's line (the units it is told to
-- build, `GetFactoryCommands`).
local function sampleWork(frame)
	local rows, n = {}, 0
	for i = 1, allCount do
		local id = all[i]
		local d = UnitDefs[known[id]]
		if d.buildSpeed > 0 then
			local power = round((spGetUnitCurrentBuildPower(id) or 0) * 100)
			local line = d.isFactory and spGetFactoryCommands(id, 0) or 0
			if type(line) ~= "number" then line = 0 end
			if line > 65535 then line = 65535 end
			local w = work[id]
			if not w or w[1] ~= power or w[2] ~= line then
				work[id] = { power, line }
				rows[n * 3 + 1], rows[n * 3 + 2], rows[n * 3 + 3] = id, power, line
				n = n + 1
			end
		end
	end
	if n > 0 then
		record("W", packU32(frame, n) .. packU16(rows))
		stats.work_rows = stats.work_rows + n
	end
end

-- `R`: u32 frame, u32 n, then n rows of three f32: unit id, its experience (`GetUnitExperience`) and its full health
-- now (BAR raises it with experience; the unit's `created` row has the one it was made with).
local function sampleRank(frame)
	local rows, n = {}, 0
	for i = 1, allCount do
		local id = all[i]
		local xp = spGetUnitExperience(id) or 0
		local _, full = spGetUnitHealth(id)
		full = round(full or 0)
		local r = rank[id]
		if not r or r[1] ~= xp or r[2] ~= full then
			rank[id] = { xp, full }
			rows[n * 3 + 1], rows[n * 3 + 2], rows[n * 3 + 3] = id, xp, full
			n = n + 1
		end
	end
	if n > 0 then
		record("R", packU32(frame, n) .. packF32(rows))
		stats.rank_rows = stats.rank_rows + n
	end
end

-- `H`: u32 frame, u32 n, then n rows of two f32: unit id and the damage it took since the last row, paralysis left out,
-- as the engine tells a widget (`UnitDamaged`).
local function flushHurt(frame)
	local rows, n = {}, 0
	for id, amount in pairs(hurt) do
		rows[n * 2 + 1], rows[n * 2 + 2] = id, amount
		n = n + 1
	end
	hurt = {}
	if n > 0 then
		record("H", packU32(frame, n) .. packF32(rows))
		stats.hurt_rows = stats.hurt_rows + n
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
	put({ t = "ev", k = kind, f = spGetGameFrame(), id = id, d = defID, team = Spring.GetFeatureTeam(id) or -1, p = { x or 0, y or 0, z or 0 },
		front = { fx or 0, fy or 0, fz or 1 }, up = { ux or 0, uy or 1, uz or 0 }, metal = metal or 0, energy = energy or 0 })
	featureMetal[id] = floor(metal or 0)
end

-- The features whose metal changed (a wreck reclaimed in part), whole metal, every WRECKS frames (the audit's wrecks).
local WRECKS = 450
local function sampleWrecks(frame)
	local changed = {}
	for id, was in pairs(featureMetal) do
		local metal = floor(spGetFeatureResources(id) or 0)
		if metal ~= was then
			featureMetal[id] = metal
			changed[#changed + 1] = { id, metal }
		end
	end
	if #changed > 0 then put({ t = "fmetal", f = frame, w = changed }) end
end

function widget:FeatureCreated(id)
	if out then featureRow(id, "feature") end
end

function widget:FeatureDestroyed(id)
	if out then put({ t = "ev", k = "feature_gone", f = spGetGameFrame(), id = id }) end
	featureMetal[id] = nil
end

------------------------------------------------------------------------------------------------------------------------

local function timed(name, started)
	stats[name] = stats[name] + spDiffTimers(spGetTimer(), started)
end

-- Each team's metal and energy: the store, its size, pull, income and spending, and from raw version 3 for the audit
-- the excess, sent and received of each; and the engine's own statistics of the team (`GetTeamStatsHistory`'s last
-- entry: metal used, produced and wasted, the same of energy, damage dealt and received, units made, lost and killed)
-- whenever the engine has added an entry.
local statsLast = {} -- team -> the statistics last written, as one string
local function resources(frame)
	local rows, statRows = {}, {}
	for _, team in ipairs(Spring.GetTeamList()) do
		local m, ms, mp, mi, me, _, msent, mrecv, mx = Spring.GetTeamResources(team, "metal")
		local e, es, ep, ei, ee, _, esent, erecv, ex = Spring.GetTeamResources(team, "energy")
		local tenth = function(v) return round((v or 0) * 10) / 10 end
		if m then
			rows[#rows + 1] = { team, round(m), round(ms), tenth(mp), tenth(mi), tenth(me), round(e), round(es), tenth(ep), tenth(ei), tenth(ee),
				tenth(mx), tenth(msent), tenth(mrecv), tenth(ex), tenth(esent), tenth(erecv) }
		end
		-- (the last entry is the one the engine is filling: it changes between entries, so it is compared, not counted)
		local n = Spring.GetTeamStatsHistory(team)
		local h = n and n > 0 and Spring.GetTeamStatsHistory(team, n, n)
		local st = h and h[1]
		if st then
			local row = { team, st.metalUsed or 0, st.metalProduced or 0, st.metalExcess or 0, st.energyUsed or 0, st.energyProduced or 0,
				st.energyExcess or 0, st.damageDealt or 0, st.damageReceived or 0, st.unitsProduced or 0, st.unitsDied or 0, st.unitsKilled or 0 }
			local key = table.concat(row, ",")
			if key ~= statsLast[team] then
				statsLast[team] = key
				statRows[#statRows + 1] = row
			end
		end
	end
	put({ t = "res", f = frame, teams = rows, stats = statRows })
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
	piecesOn = VFS.FileExists(OUT .. "tape.pieces")
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
		t = "header", format = "within-reason-tape-raw", version = 5, live = Spring.IsReplay() and 0 or 1, max_units = Game.maxUnits, ally_teams = sightTeams, engine = Engine and Engine.version or "", game = Game.gameName .. " " .. Game.gameVersion,
		sample_frames = SAMPLE, vitals_frames = VITALS, activity_frames = ACTIVITY, check_frames = CHECK, position_scale = POS, angle_scale = ANGLE,
		pieces = piecesOn and 1 or 0, piece_angle_scale = A_SCALE, piece_place_scale = T_SCALE,
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
		-- (not one dying: the engine lists a unit while it dies, after its UnitDestroyed; met again it would stand to
		-- the game's end)
		for _, id in ipairs(Spring.GetAllUnits()) do
			if not known[id] and not spGetUnitIsDead(id) then meet(id) end
		end
	end
	if frame % SAMPLE == 0 then sampleUnits(frame) end
	timed("units", t)
	if piecesOn and frame % PIECE_SAMPLE == 0 then
		t = spGetTimer()
		samplePieces(frame)
		timed("pieces", t)
	end
	t = spGetTimer()
	if frame % SAMPLE == 0 then sampleVitals(frame, frame % VITALS == 0) end
	timed("vitals", t)
	t = spGetTimer()
	if frame % ACTIVITY == 0 then
		sampleActivity(frame)
		sampleWork(frame)
		sampleRank(frame)
		flushHurt(frame)
	end
	timed("activity", t)
	if frame % VITALS == 0 then
		t = spGetTimer()
		sampleSight(frame)
		timed("sight", t)
	end
	t = spGetTimer()
	sampleProjectiles(frame)
	flushBursts(frame)
	timed("projectiles", t)
	if frame % RESOURCES == 0 then resources(frame) end
	if frame % WRECKS == 0 then sampleWrecks(frame) end
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
	put({ t = "ev", k = "finished", f = spGetGameFrame(), u = id })
	dirty[id] = true
end

function widget:UnitDestroyed(id, defID, team, attacker, attackerDef, attackerTeam)
	if not out then return end
	-- The damage it took since the last `H` row, the killing blow with it, and its experience as it dies (what it dealt
	-- in its last second), before its id is let go.
	if hurt[id] then
		record("H", packU32(spGetGameFrame(), 1) .. packF32({ id, hurt[id] }))
		hurt[id] = nil
	end
	local xp = spGetUnitExperience(id) or 0
	local _, full = spGetUnitHealth(id)
	local r = rank[id]
	if r and (r[1] ~= xp or (full and r[2] ~= round(full))) then
		record("R", packU32(spGetGameFrame(), 1) .. packF32({ id, xp, round(full or r[2]) }))
	end
	if piecesOn then
		flushPieces(id)
		writeKeys()
	end
	local x, y, z = spGetUnitPosition(id)
	local _, _, _, _, build = spGetUnitHealth(id)
	put({ t = "ev", k = "destroyed", f = spGetGameFrame(), u = id, by = attacker or -1, by_team = attackerTeam or -1,
		p = { round((x or 0) * POS), round((y or 0) * POS), round((z or 0) * POS) }, build = build or 1 })
	forget(id)
end

function widget:UnitGiven(id, defID, newTeam, oldTeam)
	if not out then return end
	meet(id, defID, newTeam)
	put({ t = "ev", k = "given", f = spGetGameFrame(), u = id, team = newTeam, from = oldTeam })
end

function widget:UnitDamaged(id, defID, team, damage, paralyzer)
	if not out then return end
	dirty[id] = true
	if not paralyzer and damage and damage > 0 then hurt[id] = (hurt[id] or 0) + damage end
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
	local row = { t = "cmd", f = spGetGameFrame(), u = id, c = cmdID, p = p, shift = (opts and opts.shift) and 1 or 0 }
	-- A unit's first settings come before the call-in that tells of its making (which names its builder): held, and
	-- written just after its `created` row.
	if known[id] then
		put(row)
	else
		early[id] = early[id] or {}
		early[id][#early[id] + 1] = row
	end
end

function widget:UnsyncedHeightMapUpdate(x1, z1, x2, z2)
	if out and frameNow > 0 then ground(x1, z1, x2, z2) end
end

function widget:AddConsoleLine(line)
	if not out then return end
	local name, text = line:match("^<([^>]+)> (.*)$")
	if name then put({ t = "ev", k = "chat", f = spGetGameFrame(), player = name, text = text }) end
end

local function close(winners)
	if not out then return end
	if piecesOn then
		for i = 1, allCount do flushPieces(all[i]) end
		writeKeys()
	end
	stats.t = "stats"
	stats.frames = frameNow
	stats.bytes = bytes
	stats.wall = spDiffTimers(spGetTimer(), wallStart)
	put(stats)
	put({ t = "over", f = spGetGameFrame(), winners = winners or {} })
	out:close()
	out = nil
	Spring.Echo(string.format("<tape dump> closed at f=%d: %d bytes; seconds in units %.1f, pieces %.1f (%d keyframes), vitals %.1f, activity %.1f, sight %.1f, projectiles %.1f; %d marks",
		frameNow, bytes, stats.units, stats.pieces, stats.key_rows, stats.vitals, stats.activity, stats.sight, stats.projectiles, stats.marks))
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
