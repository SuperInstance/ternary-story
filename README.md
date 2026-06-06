# ternary-story

A narrative engine where every decision branches three ways — negative (−1), neutral (0), positive (+1) — and accumulated moral weight shapes story outcomes beyond simple good/evil binaries.

## Why This Exists

Most interactive narratives force binary choices: fight or flee, save or destroy, yes or no. But real decisions exist on a spectrum. Sometimes you do nothing. Sometimes you act selflessly. Sometimes you're cruel. Ternary stories model this with three-way branches at every narrative node, and a `MoralTracker` that records the accumulated weight of every choice. The result isn't "good ending" or "bad ending" — it's a *moral alignment* (Dark → Shady → Neutral → Good → Virtuous) that reflects the texture of your decisions, not just their count.

The core primitive is `Trit`: −1, 0, or +1. Every choice produces one of three consequences, each with a moral weight. The story becomes a ternary decision tree, and the path through it — the `ternary_signature` — is a compact representation of the player's moral trajectory.

## Architecture

```text
StoryEngine (HashMap<String, StoryNode>)
    │
    ▼
StoryNode
├── narrative: String
├── consequences: [Consequence; 3]  ← Negative, Neutral, Positive
└── next: [Option<String>; 3]       ← Branch targets

Playback:
┌──────────┐     ┌──────────┐     ┌──────────┐
│ choose() │────►│ ArcStep  │────►│ StoryArc │
└──────────┘     └──────────┘     └──────────┘
                                          │
                                          ▼
                                   MoralTracker
                                   ├── total_weight: i64
                                   ├── choices_made: usize
                                   ├── choice_counts: [usize; 3]
                                   └── alignment() → MoralAlignment
```

### Key Types

- **`Trit`** — The fundamental ternary value: `Negative` (−1), `Neutral` (0), `Positive` (+1)
- **`Consequence`** — One outcome of a choice: trit value, description, moral weight
- **`StoryNode`** — A narrative moment with three consequences and three branch targets
- **`StoryEngine`** — Holds all nodes, drives playback
- **`StoryArc`** — The complete path through a story: sequence of `ArcStep`s
- **`MoralTracker`** — Accumulates moral weight, computes alignment and choice distributions
- **`MoralAlignment`** — Dark / Shady / Neutral / Good / Virtuous (based on average weight)
- **`StoryGenerator`** — Procedural story generation from ternary patterns

### Alignment Thresholds

| Alignment | Average Moral Weight |
|-----------|---------------------|
| Dark | < −1.0 |
| Shady | −1.0 to −0.3 |
| Neutral | −0.3 to +0.3 |
| Good | +0.3 to +1.0 |
| Virtuous | > +1.0 |

## Usage

### Manual Story Building

```rust
use ternary_story::*;

let mut engine = StoryEngine::new();

let root = StoryNode::new(
    "start",
    "You stand at a crossroads. A stranger approaches.",
    [
        Consequence::new(Trit::Negative, "You scowl and walk away.", -2),
        Consequence::new(Trit::Neutral, "You nod politely and continue.", 0),
        Consequence::new(Trit::Positive, "You greet them warmly.", 2),
    ],
    [Some("fight".into()), Some("town".into()), Some("ally".into())],
);

let ending = StoryNode::ending(
    "ending_dark",
    "The shadows close in.",
    [
        Consequence::new(Trit::Negative, "Embrace the dark.", -2),
        Consequence::new(Trit::Neutral, "Accept your fate.", 0),
        Consequence::new(Trit::Positive, "Seek redemption.", 1),
    ],
);

engine.add_node(root);
engine.add_node(ending);

// Play with explicit choices
let arc = engine.play("start", &[Trit::Positive, Trit::Positive])?;
let tracker = arc.moral_tracker();
assert_eq!(tracker.alignment(), MoralAlignment::Virtuous);

// Play with a choice function
let arc = engine.play_with("start", |_step, node| {
    // Always choose the most virtuous path
    Trit::Positive
})?;
```

### Procedural Generation

```rust
use ternary_story::*;

let gen = StoryGenerator::new();
let engine = gen.generate(&[
    PatternTrit::P,  // Light opening
    PatternTrit::N,  // Dark twist
    PatternTrit::Z,  // Ambiguous resolution
    PatternTrit::P,  // Hopeful ending
]);

let arc = engine.play("node_0", &[
    Trit::Positive,   // Follow the light
    Trit::Negative,   // Embrace darkness
    Trit::Neutral,    // Hesitate
    Trit::Positive,   // Seize hope
])?;

let sig = arc.ternary_signature();
// [+1, -1, 0, +1]
let tracker = arc.moral_tracker();
println!("Alignment: {}", tracker.alignment());
println!("Fraction positive: {:.1}%", tracker.fraction(Trit::Positive) * 100.0);
```

## API Reference

### `Trit`
- `Negative` (−1), `Neutral` (0), `Positive` (+1)
- `from_i8(v)` → `Option<Trit>`
- `.as_i8()` → `i8`

### `Consequence`
- `new(trit, description, moral_weight)`
- Fields: `trit`, `description`, `moral_weight` (i32)

### `StoryNode`
- `new(id, narrative, consequences, next)` — Full node
- `ending(id, narrative, consequences)` — Terminal node (no branches)
- `.choose(trit)` → `(&Consequence, Option<&str>)` — Select a path

### `StoryEngine`
- `new()` — Empty engine
- `.add_node(node)` — Register a node
- `.get_node(id)` → `Option<&StoryNode>`
- `.play(start_id, choices)` → `Result<StoryArc, StoryError>` — Play with explicit choices
- `.play_with(start_id, choose_fn)` → `Result<StoryArc, StoryError>` — Play with choice function
- `.node_count()` → `usize`

### `StoryArc`
- `new()` — Empty arc
- `.add_step(step)` — Append a step
- `.total_moral_weight()` → `i64`
- `.ternary_signature()` → `Vec<Trit>` — The sequence of all choices
- `.moral_tracker()` → `MoralTracker` — Reconstruct tracker from arc
- `.len()`, `.is_empty()`

### `MoralTracker`
- `new()` — Fresh tracker
- `.record(trit, weight)` — Record a choice
- `.average_weight()` → `f64`
- `.alignment()` → `MoralAlignment`
- `.fraction(trit)` → `f64` — What fraction of choices were this trit

### `MoralAlignment`
- `Dark`, `Shady`, `Neutral`, `Good`, `Virtuous`
- Implements `Display`

### `StoryGenerator`
- `new()` — With built-in narrative templates
- `.generate(pattern: &[PatternTrit])` → `StoryEngine` — Generate from pattern

### `PatternTrit`
- `N` (Negative), `Z` (Neutral), `P` (Positive) — Pattern elements

### `StoryError`
- `NodeNotFound(id)`, `InvalidChoice`
- Implements `Display` + `Error`

## The Deeper Idea

Ternary narratives capture what binary ones can't: *the weight of inaction*. Choosing "neutral" in a ternary story isn't the same as not choosing — it's an active decision with its own consequences and moral weight. This models real moral reasoning more accurately than good/evil binaries.

The `ternary_signature` is a compact moral DNA: `[-1, +1, 0, +1, -1]` tells you the shape of someone's journey without the prose. Two players who make the same moral choices get the same signature, regardless of what narrative text they read.

The `StoryGenerator` uses patterns (`[P, P, N, Z]`) to generate stories with designed moral trajectories. A `[N, N, N]` story starts dark and stays dark. A `[P, N, P]` story has hope, then crisis, then redemption. The pattern determines tone; the player's choices determine the actual path.

## Related Crates

- [`ternary-cuda-kernels`](../ternary-cuda-kernels) — GPU-accelerated ternary operations for large-scale narrative simulation
- [`ternary-auto-vectorizer`](../ternary-auto-vectorizer) — Formal verification of ternary computation equivalence
- [`character-encounter`](../character-encounter) — RPG encounter system that uses similar stat-based resolution
