#!/usr/bin/env python3
"""What a demo's network stream says about lag: every client's distance behind the server and its sim load, the
server's game speed, and the bytes of the stream every client receives, by sender, minute by minute.
Written for the lobby 10v8 on Erebos Lakes (run/matches/1794293029-human/00), where the people called "laggy".

    run/demo_net.py <demo.sdfz>

Read from the engine's own messages in the demo (rts/Net/Protocol/NetMessageTypes.h):
- `behind` is the server's PLAYERINFO ping: (server frame - the client's last answered keyframe) in ms, so it is how
  far the client's game runs behind the server, not a network round trip. It growing steadily is a client falling
  behind; a client that keeps up stays at a few hundred ms.
- `sim` is the client's own CPU report: the share of wall time in its "Sim" timer (Game.cpp SimFrame). Executing the
  commands it receives happens in ClientReadNet, outside that timer, so it does not count them.
- no INTERNAL_SPEED message means the server never slowed the game (speed control, GameServer.cpp).
- stream bytes are message bytes as the client receives them, before the link's chunk and packet headers and resends.
  AI command messages (AICOMMAND, AICOMMAND_TRACKED, AICOMMANDS) are attributed to the client hosting the AI
  (`ai@p11` is player 11's AIs, named in the seat list); `people` is the people's orders, their widgets' included.
"""
import collections, gzip, re, struct, sys

raw = gzip.open(sys.argv[1]).read()
header_size = struct.unpack_from('<i', raw, 20)[0]
script_size, stream_size, game_time, wall_time = struct.unpack_from('<4i', raw, 16 + 8 + 256 + 16 + 8)
script = raw[header_size:header_size + script_size].decode('latin1')

players = {}; hosted = collections.defaultdict(list)
for m in re.finditer(r'\[(player|ai)(\d+)\]\s*\{([^}]*)\}', script, re.I):
    kv = {k.lower(): v for k, v in re.findall(r'(\w+)\s*=\s*([^;]*);', m.group(3))}
    if m.group(1).lower() == 'player':
        players[int(m.group(2))] = kv
    else:
        hosted[int(kv.get('host', -1))].append('%s t%s' % (kv.get('shortname', kv.get('name', '?')), kv.get('team', '?')))
host = re.search(r'autohostname=([^;]*)', script, re.I)
print('host %s; game %d s, wall clock %d s (the wall clock counts from before the start)' % (host.group(1) if host else '?', game_time, wall_time))
def label(p):
    kv = players.get(p, {})
    who = kv.get('name', 'p%d' % p)
    tag = 'spec' if kv.get('spectator', '0') == '1' else 't%s' % kv.get('team', '?')
    return '%s %s %s' % (who, kv.get('countrycode', '?'), tag)
for p in sorted(players):
    print('  p%-2d %s%s' % (p, label(p), ('  hosts ' + ', '.join(hosted[p])) if hosted[p] else ''))

pos = header_size + script_size; end = pos + stream_size
frame = 0; speeds = []
behind = collections.defaultdict(lambda: collections.defaultdict(list))
sim = collections.defaultdict(lambda: collections.defaultdict(list))
stream = collections.defaultdict(collections.Counter)
while pos < end:
    _, length = struct.unpack_from('<fI', raw, pos); pos += 8
    msg = raw[pos:pos + length]; pos += length
    if not msg: continue
    kind = msg[0]; minute = frame // 1800
    if kind == 1: frame = struct.unpack_from('<i', msg, 1)[0]
    elif kind == 2: frame += 1
    elif kind == 38:
        p, cpu, ping = struct.unpack_from('<BfI', msg, 1)
        behind[minute][p].append(ping); sim[minute][p].append(cpu)
    elif kind == 20: speeds.append((frame, struct.unpack_from('<f', msg, 1)[0]))
    # a person's widgets send unit orders as AI command messages too: those count with the people's
    if kind in (14, 15, 76) and msg[3] in hosted: stream[minute]['ai@p%d' % msg[3]] += length
    elif kind in (11, 14, 15, 76): stream[minute]['people'] += length
    elif kind == 50: stream[minute]['lua'] += length
    else: stream[minute]['other'] += length

print('server speed changes: %s' % (', '.join('%d:%02d %.2f' % (f // 1800, f // 30 % 60, s) for f, s in speeds) or 'none'))
senders = sorted({k for m in stream.values() for k in m if k.startswith('ai@')})
print()
print('stream every client receives, KB/s by game minute')
print('min  total  ' + '  '.join('%7s' % s for s in senders) + '  people    lua  other')
for minute in sorted(stream):
    s = stream[minute]; kb = lambda n: n / 60 / 1024
    print('%3d %6.1f  ' % (minute, kb(sum(s.values()))) + '  '.join('%7.1f' % kb(s[k]) for k in senders) + '  %6.2f %6.2f %6.2f' % (kb(s['people']), kb(s['lua']), kb(s['other'])))
clients = sorted(players)
print()
print('each client: largest `behind` in the minute (s) / mean `sim` (%)')
print('min ' + ' '.join('%13s' % (players[p].get('name', 'p%d' % p)[:12]) for p in clients))
print('    ' + ' '.join('%13s' % ('%s %s' % (players[p].get('countrycode', '?'), 'spec' if players[p].get('spectator', '0') == '1' else 't' + players[p].get('team', '?'))) for p in clients))
for minute in sorted(behind):
    cells = []
    for p in clients:
        b = behind[minute][p]; c = sim[minute][p]
        cells.append('%13s' % ('%.1f / %.0f' % (max(b) / 1000, 100 * sum(c) / len(c)) if b else '-'))
    print('%3d ' % minute + ' '.join(cells))
