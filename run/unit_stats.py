#!/usr/bin/env python3
"""Generate the numbers of the unit glossary, crates/bot/data/units.json, from the game checkout.

Reads the unit definition files (units/**/*.lua) and the English language file
(language/en/units.json) of the Beyond All Reason checkout at upstream/Beyond-All-Reason,
follows the build lists from the two commanders, and writes one entry per reachable unit:
name, faction, tier, made_by, class, the numbers, and the rule flags. The hand-written
fields of an entry already in the file (gloss, prose, anything else this script does not
generate) are kept. The checkout's commit is recorded in the top-level "source" field.

    run/unit_stats.py                    # regenerate in place
    run/unit_stats.py --check            # only report what would change
    run/unit_stats.py --checkout <dir> --out <file>

Needs lua5.4 (or lua) on the path: the unit files are Lua tables, a few of which call
helper functions, so they are evaluated in a stub environment rather than parsed as text.
"""

import argparse
import json
import math
import os
import shutil
import subprocess
import sys
import tempfile
from collections import OrderedDict

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DEFAULT_CHECKOUT = os.path.join(REPO, "upstream", "Beyond-All-Reason")
DEFAULT_OUT = os.path.join(REPO, "crates", "bot", "data", "units.json")
ROOTS = ("armcom", "corcom")
GAME_FPS = 30  # the engine rounds reload times to whole frames

# Loads every units/**/*.lua file (Legion and Scavengers left out) in a tolerant sandbox
# and prints {"defs": {...}, "errors": [...]} as JSON. Unknown globals answer any field
# with a function returning {} and can be called, which is enough for the few files
# that reach for Spring.* or VFS.* helpers; the ones that still fail are not units the
# commanders can reach, and they are reported.
LUA_LOADER = r"""
local root = arg[1]
local function list_lua(dir)
  local out = {}
  local p = io.popen('find "' .. dir .. '" -name "*.lua" | sort')
  for line in p:lines() do out[#out + 1] = line end
  p:close()
  return out
end
local function is_array(t)
  for k in pairs(t) do if type(k) ~= "number" then return false end end
  return true
end
local function array_values(t)
  local keys = {}
  for k in pairs(t) do keys[#keys + 1] = k end
  table.sort(keys)
  local vals = {}
  for i, k in ipairs(keys) do vals[i] = t[k] end
  return vals
end
local function esc(s)
  return '"' .. s:gsub('[%c"\\]', function(c)
    if c == '"' then return '\\"' elseif c == '\\' then return '\\\\'
    elseif c == '\n' then return '\\n' elseif c == '\t' then return '\\t'
    elseif c == '\r' then return '\\r' else return string.format('\\u%04x', c:byte()) end
  end) .. '"'
end
local function tojson(v, out)
  local t = type(v)
  if t == "table" then
    if next(v) == nil then out[#out + 1] = "{}"
    elseif is_array(v) then
      out[#out + 1] = "["
      local vals = array_values(v)
      for i = 1, #vals do
        if i > 1 then out[#out + 1] = "," end
        tojson(vals[i], out)
      end
      out[#out + 1] = "]"
    else
      out[#out + 1] = "{"
      local keys = {}
      for k in pairs(v) do keys[#keys + 1] = tostring(k) end
      table.sort(keys)
      for i, k in ipairs(keys) do
        if i > 1 then out[#out + 1] = "," end
        out[#out + 1] = esc(k) .. ":"
        local val = v[k]
        if val == nil then val = v[tonumber(k)] end
        tojson(val, out)
      end
      out[#out + 1] = "}"
    end
  elseif t == "string" then out[#out + 1] = esc(v)
  elseif t == "number" then
    if v ~= v or v == math.huge or v == -math.huge then out[#out + 1] = "null"
    elseif math.type(v) == "integer" then out[#out + 1] = tostring(v)
    else out[#out + 1] = string.format("%.10g", v) end
  elseif t == "boolean" then out[#out + 1] = tostring(v)
  else out[#out + 1] = "null" end
end
local function make_env()
  local env = {}
  env._G = env
  env.string = string; env.table = table; env.math = math; env.pairs = pairs
  env.ipairs = ipairs; env.tostring = tostring; env.tonumber = tonumber; env.type = type
  env.next = next; env.select = select; env.unpack = table.unpack; env.setmetatable = setmetatable
  env.lowerkeys = function(t) return t end
  env.Shared = {}
  env.GetFilename = function() return "" end
  local stub = setmetatable({}, { __index = function() return function() return {} end end,
                                  __call = function() return {} end })
  setmetatable(env, { __index = function() return stub end })
  return env
end
local defs, errors = {}, {}
for _, f in ipairs(list_lua(root .. "/units")) do
  if not f:find("/Legion/") and not f:find("/Scavengers/") then
    local chunk, err = loadfile(f, "t", make_env())
    if not chunk then
      errors[#errors + 1] = f:sub(#root + 2) .. ": " .. tostring(err)
    else
      local ok, res = pcall(chunk)
      if not ok then
        errors[#errors + 1] = f:sub(#root + 2) .. ": " .. tostring(res)
      elseif type(res) == "table" then
        for name, def in pairs(res) do
          if type(name) == "string" and type(def) == "table" then
            def._file = f:sub(#root + 2)
            defs[name] = def
          end
        end
      end
    end
  end
end
local out = {}
tojson({ defs = defs, errors = errors }, out)
io.write(table.concat(out))
"""


def load_defs(checkout):
    lua = shutil.which("lua5.4") or shutil.which("lua")
    if not lua:
        sys.exit("lua5.4 not found on the path")
    with tempfile.NamedTemporaryFile("w", suffix=".lua", delete=False) as f:
        f.write(LUA_LOADER)
        path = f.name
    try:
        out = subprocess.run([lua, path, checkout], check=True, capture_output=True, text=True).stdout
    finally:
        os.unlink(path)
    data = json.loads(out)
    return data["defs"], data["errors"]


def checkout_commit(checkout):
    try:
        return subprocess.run(["git", "-C", checkout, "rev-parse", "HEAD"], check=True,
                              capture_output=True, text=True).stdout.strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return None


# ---------------------------------------------------------------- reachability and tiers

def reachable(defs, root):
    seen = {root}
    stack = [root]
    while stack:
        n = stack.pop()
        for b in defs.get(n, {}).get("buildoptions") or []:
            if b in defs and b not in seen:
                seen.add(b)
                stack.append(b)
    return seen


def is_factory(u):
    return bool(u.get("builder")) and mobility(u) is None and bool(u.get("buildoptions"))


def declared_tier(u):
    t = (u.get("customparams") or {}).get("techlevel")
    try:
        return int(math.floor(float(t))) if t is not None else None
    except (TypeError, ValueError):
        return None


def tiers(defs, units, made_by):
    """Tier by the maker: a unit made by a factory gets the factory's production tier
    (its declared techlevel, 1.5 counting as 1), a building gets the tier of the builder
    that makes it, the commander's buildings tier 1; the lowest over all makers. A unit
    the game itself labels tier 2 or 3 (customparams.techlevel) is never put lower than
    that, which is what makes the advanced factories tier 2 although a tier-1
    constructor builds them."""
    tier = {r: 1 for r in ROOTS if r in units}
    changed = True
    while changed:
        changed = False
        for n in units:
            best = None
            for m in made_by.get(n, []):
                if m not in tier:
                    continue
                t = (declared_tier(defs[m]) or 1) if is_factory(defs[m]) else tier[m]
                best = t if best is None else min(best, t)
            if best is None:
                continue
            d = declared_tier(defs[n])
            if d in (2, 3):
                best = max(best, d)
            if tier.get(n) != best:
                tier[n] = best
                changed = True
    return tier


# ---------------------------------------------------------------- weapons

def used_weapons(u):
    """The weapon definitions the unit actually mounts, with the dummies dropped: a
    weapon fired only by hand (the D-gun), an interceptor, a zero-damage targeting helper
    (the anti-air 'bogus_missile', the smart-trajectory dummies, footsteps, roars)."""
    if (u.get("customparams") or {}).get("unitgroup") == "explo":
        return []  # a mine or crawling bomb: its damage is the self-destruct explosion
    wdefs = u.get("weapondefs") or {}
    out = []
    for w in u.get("weapons") or []:
        key = (w.get("def") or "").lower()
        wd = wdefs.get(key)
        if not wd or not wd.get("weapontype") or wd.get("weapontype") == "Shield":
            continue
        # The target categories live on the mount, not the definition: carried over for `targets`.
        wd = dict(wd, _only=w.get("onlytargetcategory"), _bad=w.get("badtargetcategory"))
        if wd.get("commandfire") or wd.get("interceptor"):
            continue
        dmg = wd.get("damage") or {}
        if not wd.get("paralyzer") and "default" in dmg and float(dmg["default"] or 0) <= 0:
            continue
        if not wd.get("range"):
            continue
        if "UNDERWATER" in str(w.get("onlytargetcategory") or ""):
            continue  # a depth charge: only fired at submerged targets
        if key.endswith("_high") and key[:-5] in wdefs:
            continue  # the high-trajectory mode of a gun already counted
        out.append(wd)
    # A unit with both a water weapon and a dry one (the commander's underwater laser,
    # the amphibious bots' torpedoes) is summarised by its dry weapons.
    dry = [wd for wd in out if not wd.get("waterweapon")]
    return dry if dry else out


def reload_seconds(wd):
    r = wd.get("reloadtime")
    if r is None:
        return None
    frames = max(1, math.floor((float(r) + 1e-3) * GAME_FPS))
    return frames / GAME_FPS


def weapon_dps(wd, armour="default"):
    """Damage a second against an armour class ('default' for ground, 'vtol' for
    aircraft: the game gives every weapon a damage per class, and a class not listed
    takes 'default'): damage x burst x projectiles over the frame-rounded reload. A
    weapon with no 'default' entry does the engine's default of 1 (anti-air missiles are
    the usual case). Stockpiled weapons (nukes, tactical missiles, the EMP launcher) have
    no rate of fire and give None."""
    if wd.get("paralyzer") or wd.get("stockpile"):
        return None
    reload = reload_seconds(wd)
    if not reload:
        return None
    dmg = wd.get("damage") or {}
    per = float(dmg.get(armour, dmg.get("default", 1.0)) or 0)
    return per * float(wd.get("burst") or 1) * float(wd.get("projectiles") or 1) / reload


def targets(wd):
    """(hits ground, hits air, air is a bad target) from the weapon's target categories:
    'onlytargetcategory' names what it may fire at (VTOL is aircraft; SURFACE and NOTAIR
    exclude them; NOTSUB and an empty value include them), 'badtargetcategory' what it
    fires at only when nothing better is in range (the Blitz and the Stout name VTOL)."""
    only = str(wd.get("_only") or "").upper().split()
    bad = str(wd.get("_bad") or "").upper().split()
    air = not only or any(c in ("VTOL", "NOTSUB", "NOTSHIP", "NOTHOVER", "MOBILE", "ALL") for c in only)
    ground = not only or any(c not in ("VTOL",) for c in only)
    return ground, air, "VTOL" in bad


def air_summary(u):
    """(dps against ground, dps against aircraft, aircraft a bad target) over the unit's
    usable weapons, each counted only against what it may fire at."""
    ws = used_weapons(u)
    ground = air = 0.0
    reluctant = False
    for w in ws:
        g, a, bad = targets(w)
        if g:
            ground += weapon_dps(w) or 0
        if a:
            air += weapon_dps(w, "vtol") or 0
            reluctant = reluctant or bad
    return round(ground, 1), round(air, 1), reluctant


def air_words(ground, air, reluctant):
    """One clause for the glossary line: what the unit does to aircraft."""
    if not air:
        return "does nothing to aircraft" if ground else ""
    if not ground:
        return f"anti-air only: {air:.0f} dps against aircraft"
    share = air / ground if ground else 1
    tail = ", and only when nothing on the ground is in range" if reluctant else ""
    if share >= 0.8:
        return f"{air:.0f} dps against aircraft, its full damage{tail}"
    return f"{air:.0f} dps against aircraft ({share:.0%} of its ground damage{tail})"


def weapon_summary(u):
    ws = used_weapons(u)
    if not ws:
        return None, None, False, False
    rng = max(float(w["range"]) for w in ws)
    parts = [weapon_dps(w) for w in ws]
    real = [p for p in parts if p is not None]
    dps = round(sum(real), 1) if real else None
    paralyzer = any(w.get("paralyzer") for w in ws)
    energy = any(float(w.get("energypershot") or 0) > 0 for w in ws)
    return rng, dps, paralyzer, energy


# ---------------------------------------------------------------- classes

HOVER = ("HOVER",)
AMPHIBIOUS_CLASSES = {"VBOT6", "COMMANDERBOT", "ATANK3", "ABOT3", "HABOT5", "ABOTBOMB2", "AHOVER2"}
JAMMER_MIN = 100  # pop-up turrets and minelayers carry a jam radius of 8-64 that only hides themselves


def mobility(u):
    """What kind of thing moves: bot, vehicle, hovercraft, ship, submarine, aircraft,
    seaplane; None for a building."""
    if u.get("canfly"):
        return "seaplane" if "Seaplanes" in u.get("_file", "") else "aircraft"
    mc = (u.get("movementclass") or "").upper()
    if not u.get("canmove") or not mc or mc == "NANO":
        return None
    if mc.startswith("UBOAT") or mc == "EPICSUBMARINE":
        return "submarine"
    if "BOAT" in mc or mc == "EPICSHIP":
        return "ship"
    if mc.startswith("AHOVER"):
        return "bot"  # the amphibious hover bot (Platypus) is a bot that crosses water
    if "HOVER" in mc:
        return "hovercraft"
    if "TANK" in mc:
        return "vehicle"
    return "bot"


# Units whose one-line description does not say what they are for, or says it in a way
# the keyword rules below read wrongly. Everything else is derived.
CLASS_OVERRIDES = {
    "armfark": "assist bot", "cormando": "commando bot", "armspy": "spy bot", "corspy": "spy bot",
    "armdecom": "decoy commander", "cordecom": "decoy commander", "armdf": "decoy building",
    "armgremlin": "stealth tank", "armdfly": "EMP transport aircraft",
    "armspid": "EMP bot", "corbw": "EMP drone", "armstil": "EMP bomber", "armthor": "EMP assault mech",
    "armemp": "EMP missile launcher", "cortron": "tactical missile launcher",
    "armjuno": "juno launcher", "corjuno": "juno launcher",
    "armeyes": "camera tower", "coreyes": "camera tower",
    "armfatf": "targeting facility", "armtarg": "targeting facility",
    "corfatf": "targeting facility", "cortarg": "targeting facility",
    "armsd": "targeting facility", "corsd": "targeting facility",
    "armmlv": "minelayer vehicle", "cormlv": "minelayer vehicle",
    "armvader": "crawling bomb", "corroach": "crawling bomb", "corsktl": "crawling bomb",
    "armgate": "shield generator", "corgate": "shield generator",
    "corexp": "armed metal extractor", "cormexp": "armed metal extractor",
    "corbhmth": "armed geothermal plant",
    "armsnipe": "sniper bot", "armmav": "skirmisher bot", "armfido": "skirmisher bot",
    "armfast": "raider bot", "corpyro": "raider bot", "corgator": "raider tank", "armflash": "raider tank",
    "armsh": "raider hovercraft", "corsh": "raider hovercraft", "armdecade": "raider ship",
    "coresupp": "raider ship", "armlship": "raider ship",
    "armham": "line bot", "corthud": "line bot", "armstump": "line tank", "corraid": "line tank",
    "armwar": "riot bot", "corlevlr": "riot tank", "corfship": "riot ship", "corsala": "riot tank",
    "armlatnk": "riot tank", "corhal": "assault hovercraft", "armanac": "line hovercraft",
    "corsnap": "line hovercraft", "armpincer": "line tank", "corgarp": "line tank",
    "armzeus": "assault bot", "corcan": "assault bot", "cortermite": "assault bot", "corsumo": "assault bot",
    "armfboy": "artillery bot", "cormort": "artillery bot", "corhrk": "artillery bot",
    "armrock": "skirmisher bot", "corstorm": "skirmisher bot", "armsptk": "skirmisher bot",
    "armjanus": "riot tank", "armsam": "skirmisher tank", "cormist": "skirmisher tank",
    "armmh": "artillery hovercraft", "cormh": "artillery hovercraft",
    "armmerl": "artillery tank", "corvroc": "artillery tank", "corban": "assault tank",
    "armmanni": "sniper tank", "corgol": "assault tank", "armbull": "assault tank",
    "correap": "assault tank", "armcroc": "assault tank", "corparrow": "assault tank",
    "armlun": "assault hovercraft", "corsok": "assault hovercraft",
    "armvang": "artillery mech", "corcat": "artillery mech", "corshiva": "assault mech",
    "armmar": "assault mech", "armraz": "assault mech", "corkarg": "assault mech", "cordemon": "assault mech",
    "armbanth": "assault mech", "corkorg": "assault mech", "corjugg": "assault mech",
    "armpt": "patrol boat", "corpt": "patrol boat", "armamph": "amphibious assault bot",
    "coramph": "amphibious assault bot",
    "armmship": "missile ship", "cormship": "missile ship",
    "armtl": "torpedo launcher", "cortl": "torpedo launcher", "armdl": "torpedo launcher",
    "cordl": "torpedo launcher", "armatl": "torpedo launcher", "coratl": "torpedo launcher",
    "armkraken": "defence turret", "cordoom": "defence turret", "corfdoom": "defence turret",
    "armanni": "defence turret", "armpb": "defence turret", "corvipe": "defence turret",
    "armclaw": "defence turret", "cormaw": "defence turret",
    "armamb": "artillery turret", "cortoast": "artillery turret", "armguard": "artillery turret",
    "corpun": "artillery turret", "cortrem": "artillery tank", "armmart": "artillery tank",
    "cormart": "artillery tank", "armart": "artillery tank", "corwolv": "artillery tank",
    "armdrag": "wall", "cordrag": "wall", "armfort": "wall", "corfort": "wall",
    "armfdrag": "wall", "corfdrag": "wall",
    "armliche": "atomic bomber", "corcrwh": "gunship",
    "armamsub": "amphibious factory", "coramsub": "amphibious factory",
}

SHIP_WORDS = ("destroyer", "cruiser", "battleship", "flagship", "corvette", "frigate", "submarine")


def classify(name, u, english, desc):
    if name in CLASS_OVERRIDES:
        return CLASS_OVERRIDES[name]
    cp = u.get("customparams") or {}
    ug = cp.get("unitgroup")
    d = ((english or "") + " " + (desc or "")).lower()
    mob = mobility(u)
    has_weapon = bool(used_weapons(u))
    if cp.get("iscommander"):
        return "commander"
    if mob is None:
        if is_factory(u):
            if ug == "buildert3":
                return "experimental gantry"
            kinds = [mobility(u2) for u2 in u["_products"]]
            top = max(set(kinds), key=kinds.count) if kinds else None
            return {"bot": "bot factory", "vehicle": "vehicle factory", "aircraft": "aircraft factory",
                    "seaplane": "seaplane factory", "ship": "shipyard", "submarine": "amphibious factory",
                    "hovercraft": "hovercraft factory"}.get(top, "factory")
        if u.get("builder"):
            return "construction turret"
        if float(u.get("extractsmetal") or 0) > 0:
            return "armed metal extractor" if has_weapon else "metal extractor"
        if cp.get("energyconv_capacity"):
            return "energy converter"
        if u.get("windgenerator"):
            return "wind generator"
        if u.get("tidalgenerator"):
            return "tidal generator"
        if "solar" in d:
            return "solar collector"
        if "geothermal" in d:
            return "geothermal plant"
        if "fusion" in d:
            return "fusion reactor"
        if float(u.get("energymake") or 0) > 0 or float(u.get("energyupkeep") or 0) < 0:
            return "energy building"
        if ug == "antinuke":
            return "anti-nuke"
        if ug == "nuke":
            return "nuclear silo"
        if ug == "explo":
            return "mine"
        if float(u.get("energystorage") or 0) >= 1000 and not has_weapon:
            return "energy storage"
        if float(u.get("metalstorage") or 0) >= 1000 and not has_weapon:
            return "metal storage"
        if float(u.get("radardistancejam") or 0) >= JAMMER_MIN:
            return "jammer tower"
        if float(u.get("radardistance") or 0) > 0 and float(u.get("sonardistance") or 0) > 0:
            return "radar and sonar tower"
        if float(u.get("radardistance") or 0) > 0:
            return "radar tower"
        if float(u.get("sonardistance") or 0) > 0:
            return "sonar station"
        if ug == "aa":
            return "anti-air turret"
        if has_weapon:
            rng, _, _, _ = weapon_summary(u)
            if rng and rng >= 3000:
                return "long-range cannon"
            if "artillery" in d:
                return "artillery turret"
            if "torpedo" in d:
                return "torpedo launcher"
            return "defence turret"
        return "building"
    # mobile
    if ug == "antinuke":
        return "mobile anti-nuke"
    if u.get("builder") and u.get("buildoptions"):
        return "constructor " + mob
    if u.get("canresurrect"):
        return "resurrection " + mob
    if u.get("builder"):
        return "assist " + mob
    if float(u.get("transportcapacity") or 0) > 0:
        return "transport aircraft" if mob == "aircraft" else "transport " + mob
    if ug == "explo":
        return "crawling bomb"
    if float(u.get("radardistancejam") or 0) >= JAMMER_MIN:
        return "jammer " + mob
    if float(u.get("radardistance") or 0) > 0 and not has_weapon:
        return "radar plane" if mob in ("aircraft", "seaplane") else "radar " + mob
    if ug == "aa":
        return "fighter aircraft" if mob in ("aircraft", "seaplane") else "anti-air " + mob
    if ug == "emp":
        return "EMP " + mob
    if mob in ("aircraft", "seaplane"):
        if "scout" in d:
            return "scout aircraft"
        if "torpedo" in d:
            return "torpedo bomber"
        if "bomber" in d:
            return "bomber"
        if "gunship" in d:
            return "gunship"
        return "aircraft"
    if mob == "submarine":
        return "submarine"
    if mob == "ship":
        for w in SHIP_WORDS:
            if w in d:
                return w
        return "warship"
    if "scout" in d:
        return "scout " + mob
    if "raider" in d or "fast" in d:
        return "raider " + mob
    if "artillery" in d or "mortar" in d:
        return "artillery " + mob
    if "skirmish" in d or "rocket" in d or "missile" in d:
        return "skirmisher " + mob
    if "anti-swarm" in d or "riot" in d:
        return "riot " + mob
    if "sniper" in d:
        return "sniper " + mob
    if "assault" in d or "heavy" in d:
        return "assault " + mob
    if has_weapon:
        return "line " + mob
    return mob


# ---------------------------------------------------------------- flags

def flags_of(name, u, tier_class):
    cp = u.get("customparams") or {}
    ug = cp.get("unitgroup")
    mob = mobility(u)
    mc = (u.get("movementclass") or "").upper()
    rng, dps, paralyzer, energy = weapon_summary(u)
    f = []
    if u.get("cancloak"):
        f.append("cloak")
    if paralyzer:
        f.append("paralyzer")
    if ug == "emp":
        f.append("emp")
    if float(u.get("transportcapacity") or 0) > 0:
        f.append("transport")
    if u.get("canresurrect"):
        f.append("resurrect")
    if mob == "submarine" or (mob is None and float(u.get("minwaterdepth") or 0) > 0 and not u.get("floater")):
        f.append("submerged")
    if u.get("stealth"):
        f.append("stealth")
    if energy:
        f.append("needs_energy_to_fire")
    if mob in ("bot", "vehicle") and (mc in AMPHIBIOUS_CLASSES or float(u.get("maxwaterdepth") or 0) >= 200):
        f.append("amphibious")
    if mob == "hovercraft":
        f.append("hover")
    if u.get("canfly"):
        f.append("flies")
    ground_dps, air_dps, _ = air_summary(u)
    if air_dps and (not ground_dps or air_dps >= 0.5 * ground_dps):
        f.append("anti_air")
    if float(u.get("radardistancejam") or 0) >= JAMMER_MIN:
        f.append("radar_jammer")
    if float(u.get("sonardistance") or 0) > 0:
        f.append("sonar")
    if ug == "nuke":
        f.append("nuke")
    if ug == "antinuke":
        f.append("antinuke")
    if "_nolrpc_" in str(cp.get("restrictions_inclusion") or ""):
        f.append("long_range_cannon")
    if ug == "buildert3" or "Gantry" in u.get("_file", ""):
        f.append("gantry")
    if mob is None and (float(u.get("minwaterdepth") or 0) > 0 or u.get("floater") is True):
        f.append("on_water")
    if mob in ("ship", "hovercraft") or (mob in ("bot", "vehicle") and u.get("floater") is True):
        f.append("floats")
    return f


# ---------------------------------------------------------------- entries

GENERATED = ["name", "faction", "tier", "made_by", "class", "metal", "energy", "build_time", "health",
             "speed", "range", "dps", "dps_air", "air", "sight", "build_power", "energy_make", "energy_upkeep",
             "energy_storage", "metal_storage", "flags"]


def num(v, digits=1):
    if v is None:
        return None
    v = float(v)
    if v == int(v):
        return int(v)
    return round(v, digits)


def entry(name, u, lang, tier, made_by):
    cp = u.get("customparams") or {}
    rng, dps, _, _ = weapon_summary(u)
    e = OrderedDict()
    e["name"] = lang["names"].get(name, name)
    e["faction"] = "armada" if name.startswith("arm") else "cortex" if name.startswith("cor") else None
    e["tier"] = tier
    e["made_by"] = sorted(made_by)
    e["class"] = classify(name, u, e["name"], lang["descriptions"].get(name))
    e["metal"] = num(u.get("metalcost"))
    e["energy"] = num(u.get("energycost"))
    e["build_time"] = num(u.get("buildtime"))
    e["health"] = num(u.get("health"))
    e["speed"] = num(u.get("speed")) if u.get("canmove") else None
    e["range"] = num(rng)
    e["dps"] = num(dps)
    ground_dps, air_dps, reluctant = air_summary(u)
    e["dps_air"] = num(air_dps) if air_dps else None
    e["air"] = air_words(ground_dps, air_dps, reluctant)
    e["sight"] = num(u.get("sightdistance"))
    if u.get("builder") and u.get("workertime"):
        e["build_power"] = num(u.get("workertime"))
    make = float(u.get("energymake") or 0)
    upkeep = float(u.get("energyupkeep") or 0)
    if upkeep < 0:
        make += -upkeep
        upkeep = 0
    if make > 0:
        e["energy_make"] = num(make)
    if upkeep > 0:
        e["energy_upkeep"] = num(upkeep)
    if float(u.get("energystorage") or 0) >= 1000:
        e["energy_storage"] = num(u.get("energystorage"))
    if float(u.get("metalstorage") or 0) >= 1000:
        e["metal_storage"] = num(u.get("metalstorage"))
    e["flags"] = flags_of(name, u, e["class"])
    return e


def build(checkout):
    defs, errors = load_defs(checkout)
    with open(os.path.join(checkout, "language", "en", "units.json"), encoding="utf-8") as f:
        lang = json.load(f)["units"]
    units = set()
    for r in ROOTS:
        if r not in defs:
            sys.exit(f"{r} is not in the checkout's unit files")
        units |= reachable(defs, r)
    made_by = {}
    for n in units:
        for b in defs[n].get("buildoptions") or []:
            if b in units:
                made_by.setdefault(b, set()).add(n)
    for n in units:
        defs[n]["_products"] = [defs[b] for b in (defs[n].get("buildoptions") or []) if b in units]
    tier = tiers(defs, units, made_by)
    entries = OrderedDict()
    for n in sorted(units):
        entries[n] = entry(n, defs[n], lang, tier.get(n), made_by.get(n, set()))
    return entries, errors


def merge(existing, entries):
    """Generated fields replace the old ones; every other field of an existing entry
    (gloss, prose, whatever was added by hand) is kept. Entries for units no longer
    reachable are dropped and named."""
    out = OrderedDict()
    dropped = []
    old_units = (existing or {}).get("units") or {}
    for n, e in entries.items():
        old = old_units.get(n) or {}
        merged = OrderedDict(e)
        for k, v in old.items():
            if k not in GENERATED:
                merged[k] = v
        out[n] = merged
    for n in old_units:
        if n not in entries:
            dropped.append(n)
    return out, dropped


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--checkout", default=DEFAULT_CHECKOUT, help="the game checkout (default: upstream/Beyond-All-Reason)")
    ap.add_argument("--out", default=DEFAULT_OUT, help="the glossary file (default: crates/bot/data/units.json)")
    ap.add_argument("--check", action="store_true", help="report what would change without writing")
    args = ap.parse_args()

    entries, errors = build(args.checkout)
    for err in errors:
        print(f"note: could not evaluate {err}", file=sys.stderr)

    existing = None
    if os.path.exists(args.out):
        with open(args.out, encoding="utf-8") as f:
            existing = json.load(f)
    merged, dropped = merge(existing, entries)
    for n in dropped:
        print(f"warning: {n} is no longer reachable; its entry is dropped", file=sys.stderr)
    missing_prose = [n for n, e in merged.items() if not e.get("gloss") or not e.get("prose")]

    doc = OrderedDict()
    doc["source"] = checkout_commit(args.checkout)
    doc["units"] = merged
    text = json.dumps(doc, indent=1, ensure_ascii=False) + "\n"

    by_faction = {}
    for e in merged.values():
        by_faction[e["faction"]] = by_faction.get(e["faction"], 0) + 1
    summary = ", ".join(f"{k} {v}" for k, v in sorted(by_faction.items()))
    print(f"{len(merged)} units ({summary}); {len(missing_prose)} without gloss or prose; source {doc['source']}")

    if args.check:
        old_text = json.dumps(existing, indent=1, ensure_ascii=False) + "\n" if existing else ""
        print("up to date" if old_text == text else "would change")
        return
    os.makedirs(os.path.dirname(args.out), exist_ok=True)
    with open(args.out, "w", encoding="utf-8") as f:
        f.write(text)
    print(f"wrote {args.out}")


if __name__ == "__main__":
    main()
