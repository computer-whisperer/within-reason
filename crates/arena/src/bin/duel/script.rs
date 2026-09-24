//! Start script for a duel match: both teams are our AI, which only spawns and orders what the director tells it.

/// A start box as fractions of the map: left, top, right, bottom.
pub type StartBox = [f32; 4];

/// The corners of the map, 30% squares: north-west, north-east, south-west, south-east.
pub const CORNERS: [StartBox; 4] = [[0.0, 0.0, 0.3, 0.3], [0.7, 0.0, 1.0, 0.3], [0.0, 0.7, 0.3, 1.0], [0.7, 0.7, 1.0, 1.0]];

/// The duels' boxes: north-west against south-east.
pub const DUEL_BOXES: [StartBox; 2] = [CORNERS[0], CORNERS[3]];

/// The two corners farthest from a point (fractions of the map): where the commanders stand out of a scenario's way.
pub fn farthest_corners(x: f32, z: f32) -> [StartBox; 2] {
    let mut corners = CORNERS;
    let distance = |b: &StartBox| ((b[0] + b[2]) / 2.0 - x).hypot((b[1] + b[3]) / 2.0 - z);
    corners.sort_by(|a, b| distance(b).total_cmp(&distance(a)));
    [corners[0], corners[1]]
}

pub fn render(game: &str, map: &str, host_port: u16, autohost_port: u16, seed: u32, boxes: [StartBox; 2]) -> String {
    let [a, b] = boxes;
    format!(
        "[GAME]
{{
	GameType={game};
	MapName={map};
	IsHost=1;
	HostIP=127.0.0.1;
	HostPort={host_port};
	AutohostIP=127.0.0.1;
	AutohostPort={autohost_port};
	MyPlayerName=arena;
	StartPosType=2;
	GameStartDelay=0;
	FixedRNGSeed={seed};
	NumPlayers=1;
	NumTeams=2;
	NumAllyTeams=2;
	[PLAYER0] {{ Name=arena; Spectator=1; }}
	[AI0] {{ Name=ai0; Team=0; Host=0; ShortName=WReason; Version=0.1; }}
	[AI1] {{ Name=ai1; Team=1; Host=0; ShortName=WReason; Version=0.1; }}
	[TEAM0] {{ TeamLeader=0; AllyTeam=0; Side=Armada; }}
	[TEAM1] {{ TeamLeader=0; AllyTeam=1; Side=Cortex; }}
	[ALLYTEAM0] {{ NumAllies=0; StartRectLeft={}; StartRectTop={}; StartRectRight={}; StartRectBottom={}; }}
	[ALLYTEAM1] {{ NumAllies=0; StartRectLeft={}; StartRectTop={}; StartRectRight={}; StartRectBottom={}; }}
}}
",
        a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3]
    )
}
