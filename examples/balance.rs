//! Plays every champion against every champion many times (both sides with the CPU brain) and prints win rates and how
//! each move is used: how often, how much damage it does in all and per use.
//!
//!     cargo run --release --example balance -- [matches per pairing] [report.md]

use std::collections::BTreeMap;
use std::sync::Mutex;

use t2f::brain::Brain;
use t2f::sim::{CharacterData, Fighter, FighterState, GameState, Phase};

const CHAMPIONS: [&str; 7] = ["ayz", "anaran", "cowboy", "gray_persona", "junkhead", "katmata", "scribe"];

#[derive(Default, Clone)]
struct MoveStats {
    uses: u32,
    hits: u32,
    damage: i64,
}

#[derive(Default)]
struct Totals {
    /// Matches played (a mirror match counts for both sides), for the per-match averages.
    matches: u32,
    moves: BTreeMap<String, MoveStats>,
    /// Damage taken, by "attacker: move".
    taken: BTreeMap<String, i64>,
}

fn load(name: &str) -> CharacterData {
    CharacterData::from_ron(&std::fs::read_to_string(format!("assets/characters/{name}.ron")).unwrap()).unwrap()
}

/// One match; returns who won (0 or 1, or None if it ran out of time) and updates both champions' move stats.
fn play(chars: [&CharacterData; 2], seed: u32, stats: &mut [BTreeMap<String, MoveStats>; 2], rounds: &mut [u32; 2], taken: &mut [BTreeMap<String, i64>; 2]) -> Option<usize> {
    let mut s = GameState::new();
    s.frame = seed * 37;
    s.set_health([0, 1].map(|i| chars[i].health.unwrap_or(Fighter::MAX_HEALTH)));
    let mut brains = [Brain::new(), Brain::new()];
    let mut prev = s.clone();
    for _ in 0..60_000 {
        let inputs = [brains[0].input(&s, 0, chars), brains[1].input(&s, 1, chars)];
        s.step(inputs, chars);
        for me in 0..2 {
            let f = &s.fighters[me];
            let p = &prev.fighters[me];
            // a move starts
            if f.state == FighterState::Attack && (p.state != FighterState::Attack || p.attack != f.attack || f.state_frame < p.state_frame) {
                stats[me].entry(chars[me].moves[f.attack].name.clone()).or_default().uses += 1;
            }
            // damage the other one took this step, and which of my moves did it
            let foe = 1 - me;
            let lost = prev.fighters[foe].health - s.fighters[foe].health;
            if lost > 0 && s.round == prev.round {
                let index = if f.state == FighterState::Attack {
                    Some(f.attack)
                } else {
                    let near = |x: i32| (x - prev.fighters[foe].x).abs();
                    let shot = prev.projectiles.iter().filter(|q| q.owner == me).min_by_key(|q| near(q.x)).map(|q| (near(q.x), q.move_index));
                    let minion = prev.minions.iter().filter(|m| m.owner == me).min_by_key(|m| near(m.x)).map(|m| (near(m.x), m.move_index));
                    let zone = prev.zones.iter().filter(|z| z.owner == me).map(|z| (0, z.move_index)).next();
                    [shot, minion, zone].into_iter().flatten().min_by_key(|c| c.0).map(|c| c.1)
                };
                let name = index.and_then(|i| chars[me].moves.get(i)).map_or("(other: chip, costs)".to_string(), |m| m.name.clone());
                *taken[foe].entry(format!("{}: {}", chars[me].name, name)).or_default() += lost as i64;
                let e = stats[me].entry(name).or_default();
                e.hits += 1;
                e.damage += lost as i64;
            }
        }
        if s.round != prev.round {
            // somebody won the round
            let winner = if s.wins[0] > prev.wins[0] { 0 } else { 1 };
            rounds[winner] += 1;
        }
        prev = s.clone();
        if s.phase == Phase::MatchOver {
            return Some(if s.wins[0] > s.wins[1] { 0 } else { 1 });
        }
    }
    None
}

fn main() {
    let mut args = std::env::args().skip(1);
    let reps: u32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(6);
    let report = args.next();
    // PAIR=scribe,gray_persona limits it to those champions (to look at one matchup)
    let only: Option<Vec<String>> = std::env::var("PAIR").ok().map(|v| v.split(',').map(String::from).collect());
    let chars: Vec<CharacterData> = CHAMPIONS.iter().filter(|n| only.as_ref().map_or(true, |o| o.iter().any(|x| x == **n))).map(|n| load(n)).collect();
    let n = chars.len();
    // matrix[a][b] = (a's match wins, matches)
    let matrix = Mutex::new(vec![vec![(0u32, 0u32); n]; n]);
    let totals: Mutex<Vec<Totals>> = Mutex::new((0..n).map(|_| Totals::default()).collect());
    // (no mirror matches: a champion is only ever played against the others)
    let jobs: Vec<(usize, usize)> = (0..n).flat_map(|a| (a + 1..n).map(move |b| (a, b))).collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let j = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let Some(&(a, b)) = jobs.get(j) else { return };
                for rep in 0..reps {
                    // alternate who starts on the left
                    let (l, r) = if rep % 2 == 0 { (a, b) } else { (b, a) };
                    let mut stats = [BTreeMap::new(), BTreeMap::new()];
                    let mut rounds = [0u32; 2];
                    let mut taken = [BTreeMap::new(), BTreeMap::new()];
                    let result = play([&chars[l], &chars[r]], rep + 1, &mut stats, &mut rounds, &mut taken);
                    let mut m = matrix.lock().unwrap();
                    let mut t = totals.lock().unwrap();
                    for (side, who) in [(0usize, l), (1, r)] {
                        let other = if side == 0 { r } else { l };
                        // (the win rates leave the mirror matches out)
                        if a != b {
                            if result == Some(side) {
                                m[who][other].0 += 1;
                            }
                            m[who][other].1 += 1;
                        }
                        t[who].matches += 1;
                        for (name, d) in &taken[side] {
                            *t[who].taken.entry(name.clone()).or_default() += d;
                        }
                        for (name, st) in &stats[side] {
                            let e = t[who].moves.entry(name.clone()).or_default();
                            e.uses += st.uses;
                            e.hits += st.hits;
                            e.damage += st.damage;
                        }
                    }
                }
            });
        }
    });
    let matrix = matrix.into_inner().unwrap();
    let totals = totals.into_inner().unwrap();
    let mut out = String::new();
    let short = |n: &str| n.chars().take(9).collect::<String>();
    out += "## Match win rate (row beats column), %\n\n| |";
    for c in &chars {
        out += &format!(" {} |", short(&c.name));
    }
    out += " overall |\n|---|";
    for _ in 0..=n {
        out += "---|";
    }
    out += "\n";
    for a in 0..n {
        out += &format!("| **{}** |", short(&chars[a].name));
        let (mut w, mut g) = (0, 0);
        for b in 0..n {
            if a == b {
                out += " - |";
                continue;
            }
            let (x, y) = matrix[a][b];
            out += &format!(" {} |", if y > 0 { 100 * x / y } else { 0 });
            w += x;
            g += y;
        }
        out += &format!(" **{}** |\n", if g > 0 { 100 * w / g } else { 0 });
    }
    out += "\n## Moves (all matches, per champion)\n";
    let mut all: Vec<(String, String, f32, f32, f32, i64, u32)> = Vec::new();
    for (i, c) in chars.iter().enumerate() {
        let t = &totals[i];
        let matches = t.matches.max(1) as f32;
        let total_damage: i64 = t.moves.values().map(|m| m.damage).sum();
        out += &format!("\n### {} ({} matches)\n\n| move | uses / match | hits / match | damage / match | damage per use | share of his damage |\n|---|---|---|---|---|---|\n", c.name, t.matches);
        let mut rows: Vec<(&String, &MoveStats)> = t.moves.iter().collect();
        rows.sort_by_key(|(_, m)| -(m.damage));
        for (name, m) in rows {
            let per_use = if m.uses > 0 { m.damage as f32 / m.uses as f32 } else { 0.0 };
            out += &format!(
                "| {name} | {:.1} | {:.1} | {:.0} | {:.0} | {}% |\n",
                m.uses as f32 / matches,
                m.hits as f32 / matches,
                m.damage as f32 / matches,
                per_use,
                if total_damage > 0 { 100 * m.damage / total_damage } else { 0 }
            );
            if m.uses >= t.matches && !name.starts_with("(other") {
                all.push((c.name.clone(), name.clone(), m.uses as f32 / matches, m.damage as f32 / matches, per_use, m.damage, m.uses));
            }
        }
    }
    let top = |out: &mut String, title: &str, rows: &mut Vec<(String, String, f32, f32, f32, i64, u32)>, key: &dyn Fn(&(String, String, f32, f32, f32, i64, u32)) -> f32, desc: bool| {
        rows.sort_by(|a, b| if desc { key(b).total_cmp(&key(a)) } else { key(a).total_cmp(&key(b)) });
        *out += &format!("\n### {title}\n\n| champion | move | uses / match | damage / match | damage per use |\n|---|---|---|---|---|\n");
        for r in rows.iter().take(8) {
            *out += &format!("| {} | {} | {:.1} | {:.0} | {:.0} |\n", r.0, r.1, r.2, r.3, r.4);
        }
    };
    out += "\n## Rankings across the whole roster (moves used at least once a match)\n";
    top(&mut out, "Used the most", &mut all, &|r| r.2, true);
    top(&mut out, "Used the least", &mut all, &|r| r.2, false);
    top(&mut out, "Most damage per match", &mut all, &|r| r.3, true);
    top(&mut out, "Least damage per match", &mut all, &|r| r.3, false);
    top(&mut out, "Most damage per use", &mut all, &|r| r.4, true);
    top(&mut out, "Least damage per use", &mut all, &|r| r.4, false);
    // damage taken, by attacking move, for each champion (top ten)
    out += "
## Damage taken, by the move that did it (per match)
";
    for (i, c) in chars.iter().enumerate() {
        let t = &totals[i];
        let matches = t.matches.max(1) as f32;
        let total: i64 = t.taken.values().sum();
        out += &format!("
### {} takes {:.0} per match

| attacker: move | damage / match | share |
|---|---|---|
", c.name, total as f32 / matches);
        let mut rows: Vec<_> = t.taken.iter().collect();
        rows.sort_by_key(|r| -*r.1);
        for (name, d) in rows.into_iter().take(10) {
            out += &format!("| {name} | {:.0} | {}% |
", *d as f32 / matches, if total > 0 { 100 * d / total } else { 0 });
        }
    }
    println!("{out}");
    if let Some(path) = report {
        std::fs::write(path, &out).unwrap();
    }
}
