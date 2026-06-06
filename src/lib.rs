//! # ternary-story
//!
//! A ternary narrative engine where every choice has three paths: negative (-1), neutral (0),
//! and positive (+1). Stories become ternary decision trees, and the accumulated moral weight
//! of choices shapes outcomes in ways binary narratives never could.
//!
//! The core insight: most stories force binary choices (yes/no, good/evil). But real decisions
//! exist on a spectrum. Ternary stories capture that nuance with three-way branching at every
//! narrative node, and a moral tracker that remembers the tendency of your choices.

use std::collections::HashMap;

/// A trit value: the fundamental unit of ternary choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trit {
    Negative = -1,
    Neutral = 0,
    Positive = 1,
}

impl Trit {
    /// Convert from an i8 value to a Trit.
    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Trit::Negative),
            0 => Some(Trit::Neutral),
            1 => Some(Trit::Positive),
            _ => None,
        }
    }

    /// Convert to i8.
    pub fn as_i8(self) -> i8 {
        self as i8
    }
}

impl std::fmt::Display for Trit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Trit::Negative => write!(f, "−1"),
            Trit::Neutral => write!(f, " 0"),
            Trit::Positive => write!(f, "+1"),
        }
    }
}

/// Consequence of a ternary choice — describes what happens for each branch.
#[derive(Debug, Clone)]
pub struct Consequence {
    pub trit: Trit,
    pub description: String,
    pub moral_weight: i32, // How much this shifts the moral compass
}

impl Consequence {
    pub fn new(trit: Trit, description: impl Into<String>, moral_weight: i32) -> Self {
        Self {
            trit,
            description: description.into(),
            moral_weight,
        }
    }
}

/// A node in the story — a moment with narrative text, three possible choices,
/// and consequences for each.
#[derive(Debug, Clone)]
pub struct StoryNode {
    pub id: String,
    pub narrative: String,
    pub consequences: [Consequence; 3],
    /// Next node IDs for Negative, Neutral, Positive choices.
    /// None means the story ends on that branch.
    pub next: [Option<String>; 3],
}

impl StoryNode {
    /// Create a new story node.
    pub fn new(
        id: impl Into<String>,
        narrative: impl Into<String>,
        consequences: [Consequence; 3],
        next: [Option<String>; 3],
    ) -> Self {
        Self {
            id: id.into(),
            narrative: narrative.into(),
            consequences,
            next,
        }
    }

    /// Create a leaf node (story ending) with no further branches.
    pub fn ending(id: impl Into<String>, narrative: impl Into<String>, consequences: [Consequence; 3]) -> Self {
        Self {
            id: id.into(),
            narrative: narrative.into(),
            consequences,
            next: [None, None, None],
        }
    }

    /// Choose a path. Returns the consequence and optional next node ID.
    pub fn choose(&self, trit: Trit) -> (&Consequence, Option<&str>) {
        let idx = match trit {
            Trit::Negative => 0,
            Trit::Neutral => 1,
            Trit::Positive => 2,
        };
        (&self.consequences[idx], self.next[idx].as_deref())
    }
}

/// Tracks the moral tendency of choices over a story arc.
#[derive(Debug, Clone, Default)]
pub struct MoralTracker {
    /// Total accumulated moral weight.
    pub total_weight: i64,
    /// Number of choices made.
    pub choices_made: usize,
    /// Distribution of choice types.
    pub choice_counts: [usize; 3], // [negative, neutral, positive]
}

impl MoralTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a choice with its moral weight.
    pub fn record(&mut self, trit: Trit, weight: i32) {
        self.total_weight += weight as i64;
        self.choices_made += 1;
        let idx = match trit {
            Trit::Negative => 0,
            Trit::Neutral => 1,
            Trit::Positive => 2,
        };
        self.choice_counts[idx] += 1;
    }

    /// Average moral weight per choice.
    pub fn average_weight(&self) -> f64 {
        if self.choices_made == 0 {
            0.0
        } else {
            self.total_weight as f64 / self.choices_made as f64
        }
    }

    /// Determine the overall moral alignment.
    pub fn alignment(&self) -> MoralAlignment {
        let avg = self.average_weight();
        if avg > 1.0 {
            MoralAlignment::Virtuous
        } else if avg > 0.3 {
            MoralAlignment::Good
        } else if avg > -0.3 {
            MoralAlignment::Neutral
        } else if avg > -1.0 {
            MoralAlignment::Shady
        } else {
            MoralAlignment::Dark
        }
    }

    /// What fraction of choices were the given trit?
    pub fn fraction(&self, trit: Trit) -> f64 {
        if self.choices_made == 0 {
            return 0.0;
        }
        let idx = match trit {
            Trit::Negative => 0,
            Trit::Neutral => 1,
            Trit::Positive => 2,
        };
        self.choice_counts[idx] as f64 / self.choices_made as f64
    }
}

/// Moral alignment categories based on accumulated choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoralAlignment {
    Dark,     // avg < -1.0
    Shady,    // -1.0 <= avg < -0.3
    Neutral,  // -0.3 <= avg <= 0.3
    Good,     // 0.3 < avg <= 1.0
    Virtuous, // avg > 1.0
}

impl std::fmt::Display for MoralAlignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoralAlignment::Dark => write!(f, "Dark"),
            MoralAlignment::Shady => write!(f, "Shady"),
            MoralAlignment::Neutral => write!(f, "Neutral"),
            MoralAlignment::Good => write!(f, "Good"),
            MoralAlignment::Virtuous => write!(f, "Virtuous"),
        }
    }
}

/// A single step in a story arc — the node visited, the choice made, and the consequence.
#[derive(Debug, Clone)]
pub struct ArcStep {
    pub node_id: String,
    pub narrative: String,
    pub choice: Trit,
    pub consequence_description: String,
    pub moral_weight: i32,
}

/// A path through the story — the complete sequence of choices made.
#[derive(Debug, Clone, Default)]
pub struct StoryArc {
    pub steps: Vec<ArcStep>,
}

impl StoryArc {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_step(&mut self, step: ArcStep) {
        self.steps.push(step);
    }

    /// Total moral weight accumulated along this arc.
    pub fn total_moral_weight(&self) -> i64 {
        self.steps.iter().map(|s| s.moral_weight as i64).sum()
    }

    /// Extract the ternary signature: the sequence of trit choices.
    pub fn ternary_signature(&self) -> Vec<Trit> {
        self.steps.iter().map(|s| s.choice).collect()
    }

    /// Number of steps in this arc.
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Replay this arc and build a moral tracker.
    pub fn moral_tracker(&self) -> MoralTracker {
        let mut tracker = MoralTracker::new();
        for step in &self.steps {
            tracker.record(step.choice, step.moral_weight);
        }
        tracker
    }
}

/// The story engine — holds all nodes and drives the narrative.
#[derive(Debug, Clone)]
pub struct StoryEngine {
    nodes: HashMap<String, StoryNode>,
}

impl StoryEngine {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Add a node to the story.
    pub fn add_node(&mut self, node: StoryNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: &str) -> Option<&StoryNode> {
        self.nodes.get(id)
    }

    /// Play through the story following a sequence of trit choices.
    /// Returns the resulting arc. Stops when a choice leads to a missing next node.
    pub fn play(&self, start_id: &str, choices: &[Trit]) -> Result<StoryArc, StoryError> {
        let mut arc = StoryArc::new();
        let mut current_id = start_id.to_string();

        for &choice in choices {
            let node = self.nodes.get(&current_id).ok_or(StoryError::NodeNotFound(current_id.clone()))?;
            let (consequence, next) = node.choose(choice);

            arc.add_step(ArcStep {
                node_id: current_id.clone(),
                narrative: node.narrative.clone(),
                choice,
                consequence_description: consequence.description.clone(),
                moral_weight: consequence.moral_weight,
            });

            match next {
                Some(next_id) => current_id = next_id.to_string(),
                None => break, // Story ended
            }
        }

        Ok(arc)
    }

    /// Play a story to completion, using a choice function to decide each step.
    /// The choice function receives the current node index (0-based) and the node.
    pub fn play_with<F>(&self, start_id: &str, mut choose: F) -> Result<StoryArc, StoryError>
    where
        F: FnMut(usize, &StoryNode) -> Trit,
    {
        let mut arc = StoryArc::new();
        let mut current_id = start_id.to_string();
        let mut step = 0;

        loop {
            let node = self.nodes.get(&current_id).ok_or(StoryError::NodeNotFound(current_id.clone()))?;
            let choice = choose(step, node);
            let (consequence, next) = node.choose(choice);

            arc.add_step(ArcStep {
                node_id: current_id.clone(),
                narrative: node.narrative.clone(),
                choice,
                consequence_description: consequence.description.clone(),
                moral_weight: consequence.moral_weight,
            });

            step += 1;

            match next {
                Some(next_id) => current_id = next_id.to_string(),
                None => break,
            }

            // Safety limit
            if step > 1000 {
                break;
            }
        }

        Ok(arc)
    }

    /// Number of nodes in the story.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

/// Errors that can occur during story playback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoryError {
    NodeNotFound(String),
    InvalidChoice,
}

impl std::fmt::Display for StoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoryError::NodeNotFound(id) => write!(f, "Node not found: {}", id),
            StoryError::InvalidChoice => write!(f, "Invalid choice"),
        }
    }
}

impl std::error::Error for StoryError {}

/// Pattern element for procedural story generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternTrit {
    N, // Negative
    Z, // Zero/Neutral
    P, // Positive
}

/// Generates stories from ternary patterns. Given a sequence of trit patterns,
/// generates story nodes with appropriate consequences that follow the pattern's
/// moral trajectory.
pub struct StoryGenerator {
    /// Templates for narratives keyed by pattern position type.
    negative_narratives: Vec<&'static str>,
    neutral_narratives: Vec<&'static str>,
    positive_narratives: Vec<&'static str>,
}

impl StoryGenerator {
    pub fn new() -> Self {
        Self {
            negative_narratives: vec![
                "A shadow falls across your path. Something valuable slips away.",
                "The cold wind bites. An ally turns their back.",
                "Darkness gathers. The price of your choices comes due.",
                "A door slams shut behind you. There is no going back.",
                "The ground trembles. You have made a dangerous enemy.",
            ],
            neutral_narratives: vec![
                "The path stretches on, neither ascending nor descending.",
                "A moment of stillness. The world holds its breath.",
                "You pause at a crossroads. Signs point in every direction.",
                "Time passes without incident. The journey continues.",
                "The fog lifts, revealing... more fog. Ambiguity reigns.",
            ],
            positive_narratives: vec![
                "Light breaks through the clouds. A new possibility emerges.",
                "A hand reaches out in friendship. Trust blooms.",
                "The path clears. Your resolve strengthens.",
                "A door opens to reveal a garden. Hope returns.",
                "Music drifts through the air. Something has changed for the better.",
            ],
        }
    }

    /// Generate a story from a pattern of trit values.
    /// The pattern determines the moral trajectory and narrative tone.
    pub fn generate(&self, pattern: &[PatternTrit]) -> StoryEngine {
        let mut engine = StoryEngine::new();
        let n = pattern.len();
        if n == 0 {
            return engine;
        }

        for (i, &pt) in pattern.iter().enumerate() {
            let id = format!("node_{}", i);
            let narrative = self.narrative_for(pt, i);
            let base_weight = match pt {
                PatternTrit::N => -2,
                PatternTrit::Z => 0,
                PatternTrit::P => 2,
            };

            let consequences = [
                Consequence::new(Trit::Negative, format!("Embrace the darkness (node {})", i), base_weight - 1),
                Consequence::new(Trit::Neutral, format!("Hesitate and wait (node {})", i), base_weight),
                Consequence::new(Trit::Positive, format!("Seize the light (node {})", i), base_weight + 1),
            ];

            let next = if i + 1 < n {
                let next_id = Some(format!("node_{}", i + 1));
                [next_id.clone(), next_id.clone(), next_id]
            } else {
                // Last node: only the "matching" choice continues (to an ending), others end
                let ending_id = format!("ending_{}", i);
                match pt {
                    PatternTrit::N => [Some(ending_id.clone()), None, None],
                    PatternTrit::Z => [None, Some(ending_id.clone()), None],
                    PatternTrit::P => [None, None, Some(ending_id.clone())],
                }
            };

            let node = StoryNode::new(id, narrative, consequences, next);
            engine.add_node(node);
        }

        // Add ending node
        if n > 0 {
            let last_pt = pattern.last().unwrap();
            let ending_narrative = match last_pt {
                PatternTrit::N => "The darkness consumes all. But even in the void, something stirs...",
                PatternTrit::Z => "The story dissolves into equilibrium. Neither triumph nor tragedy. Just... balance.",
                PatternTrit::P => "Light prevails. The world reshapes itself around your kindness. A new chapter begins.",
            };
            let ending_id = format!("ending_{}", n - 1);
            let ending_weight = match last_pt {
                PatternTrit::N => -5,
                PatternTrit::Z => 0,
                PatternTrit::P => 5,
            };
            let ending = StoryNode::ending(
                ending_id,
                ending_narrative,
                [
                    Consequence::new(Trit::Negative, "The end.", ending_weight - 1),
                    Consequence::new(Trit::Neutral, "The end.", ending_weight),
                    Consequence::new(Trit::Positive, "The end.", ending_weight + 1),
                ],
            );
            engine.add_node(ending);
        }

        engine
    }

    fn narrative_for(&self, pt: PatternTrit, index: usize) -> String {
        let pool = match pt {
            PatternTrit::N => &self.negative_narratives,
            PatternTrit::Z => &self.neutral_narratives,
            PatternTrit::P => &self.positive_narratives,
        };
        pool[index % pool.len()].to_string()
    }
}

impl Default for StoryGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_story() -> StoryEngine {
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

        let fight = StoryNode::new(
            "fight",
            "The stranger challenges you to a duel.",
            [
                Consequence::new(Trit::Negative, "You fight dirty and win.", -3),
                Consequence::new(Trit::Neutral, "You duel honorably to a draw.", 0),
                Consequence::new(Trit::Positive, "You refuse and offer friendship.", 3),
            ],
            [Some("ending_dark".into()), Some("ending_neutral".into()), Some("ending_good".into())],
        );

        let town = StoryNode::new(
            "town",
            "You arrive at a quiet town. The tavern beckons.",
            [
                Consequence::new(Trit::Negative, "You steal from the tavern.", -2),
                Consequence::new(Trit::Neutral, "You rest and move on.", 0),
                Consequence::new(Trit::Positive, "You share stories with the locals.", 1),
            ],
            [Some("ending_dark".into()), Some("ending_neutral".into()), Some("ending_good".into())],
        );

        let ally = StoryNode::new(
            "ally",
            "The stranger becomes your companion. A village needs help.",
            [
                Consequence::new(Trit::Negative, "You exploit the village.", -4),
                Consequence::new(Trit::Neutral, "You help but demand payment.", 0),
                Consequence::new(Trit::Positive, "You help freely and earn gratitude.", 4),
            ],
            [Some("ending_dark".into()), Some("ending_neutral".into()), Some("ending_good".into())],
        );

        let ending_dark = StoryNode::ending(
            "ending_dark",
            "The shadows close in. Your choices have led you here.",
            [
                Consequence::new(Trit::Negative, "Embrace the dark.", -2),
                Consequence::new(Trit::Neutral, "Accept your fate.", 0),
                Consequence::new(Trit::Positive, "Seek redemption.", 1),
            ],
        );

        let ending_neutral = StoryNode::ending(
            "ending_neutral",
            "The story fades to gray. Balance is maintained.",
            [
                Consequence::new(Trit::Negative, "A tinge of regret.", -1),
                Consequence::new(Trit::Neutral, "Perfect equilibrium.", 0),
                Consequence::new(Trit::Positive, "Quiet satisfaction.", 1),
            ],
        );

        let ending_good = StoryNode::ending(
            "ending_good",
            "Light fills the horizon. Your journey has meaning.",
            [
                Consequence::new(Trit::Negative, "Doubt creeps in.", -1),
                Consequence::new(Trit::Neutral, "Contentment.", 0),
                Consequence::new(Trit::Positive, "Pure joy.", 2),
            ],
        );

        engine.add_node(root);
        engine.add_node(fight);
        engine.add_node(town);
        engine.add_node(ally);
        engine.add_node(ending_dark);
        engine.add_node(ending_neutral);
        engine.add_node(ending_good);
        engine
    }

    #[test]
    fn test_trit_values() {
        assert_eq!(Trit::Negative.as_i8(), -1);
        assert_eq!(Trit::Neutral.as_i8(), 0);
        assert_eq!(Trit::Positive.as_i8(), 1);
        assert_eq!(Trit::from_i8(-1), Some(Trit::Negative));
        assert_eq!(Trit::from_i8(0), Some(Trit::Neutral));
        assert_eq!(Trit::from_i8(1), Some(Trit::Positive));
        assert_eq!(Trit::from_i8(2), None);
    }

    #[test]
    fn test_trit_display() {
        assert_eq!(format!("{}", Trit::Negative), "−1");
        assert_eq!(format!("{}", Trit::Neutral), " 0");
        assert_eq!(format!("{}", Trit::Positive), "+1");
    }

    #[test]
    fn test_node_choose() {
        let engine = make_test_story();
        let start = engine.get_node("start").unwrap();

        let (neg_cons, neg_next) = start.choose(Trit::Negative);
        assert_eq!(neg_cons.moral_weight, -2);
        assert_eq!(neg_next, Some("fight"));

        let (neu_cons, neu_next) = start.choose(Trit::Neutral);
        assert_eq!(neu_cons.moral_weight, 0);
        assert_eq!(neu_next, Some("town"));

        let (pos_cons, pos_next) = start.choose(Trit::Positive);
        assert_eq!(pos_cons.moral_weight, 2);
        assert_eq!(pos_next, Some("ally"));
    }

    #[test]
    fn test_node_ending() {
        let ending = StoryNode::ending(
            "end",
            "The end.",
            [
                Consequence::new(Trit::Negative, "bad", -1),
                Consequence::new(Trit::Neutral, "meh", 0),
                Consequence::new(Trit::Positive, "good", 1),
            ],
        );
        let (_, next) = ending.choose(Trit::Positive);
        assert!(next.is_none());
    }

    #[test]
    fn test_play_all_positive() {
        let engine = make_test_story();
        let arc = engine.play("start", &[Trit::Positive, Trit::Positive]).unwrap();

        assert_eq!(arc.len(), 2);
        assert_eq!(arc.steps[0].node_id, "start");
        assert_eq!(arc.steps[0].choice, Trit::Positive);
        assert_eq!(arc.steps[1].node_id, "ally");
        assert_eq!(arc.steps[1].choice, Trit::Positive);
    }

    #[test]
    fn test_play_all_negative() {
        let engine = make_test_story();
        let arc = engine.play("start", &[Trit::Negative, Trit::Negative]).unwrap();

        assert_eq!(arc.len(), 2);
        assert_eq!(arc.steps[0].node_id, "start");
        assert_eq!(arc.steps[1].node_id, "fight");
    }

    #[test]
    fn test_play_mixed() {
        let engine = make_test_story();
        let arc = engine.play("start", &[Trit::Positive, Trit::Negative]).unwrap();

        assert_eq!(arc.steps[0].node_id, "start");
        assert_eq!(arc.steps[0].choice, Trit::Positive);
        assert_eq!(arc.steps[1].node_id, "ally");
        assert_eq!(arc.steps[1].choice, Trit::Negative);
    }

    #[test]
    fn test_play_missing_node() {
        let engine = make_test_story();
        let result = engine.play("nonexistent", &[Trit::Positive]);
        assert!(result.is_err());
    }

    #[test]
    fn test_play_stops_on_ending() {
        let engine = make_test_story();
        // After the ending node, no more nodes to visit
        let arc = engine.play("start", &[Trit::Positive, Trit::Positive, Trit::Positive]).unwrap();
        // The third choice is on the ending_good node, which has no next, so arc has 3 steps
        assert_eq!(arc.len(), 3);
    }

    #[test]
    fn test_moral_tracker_positive_path() {
        let engine = make_test_story();
        let arc = engine.play("start", &[Trit::Positive, Trit::Positive]).unwrap();
        let tracker = arc.moral_tracker();

        assert_eq!(tracker.choices_made, 2);
        assert_eq!(tracker.total_weight, 6); // 2 + 4
        assert_eq!(tracker.alignment(), MoralAlignment::Virtuous);
    }

    #[test]
    fn test_moral_tracker_negative_path() {
        let engine = make_test_story();
        let arc = engine.play("start", &[Trit::Negative, Trit::Negative]).unwrap();
        let tracker = arc.moral_tracker();

        assert_eq!(tracker.total_weight, -5); // -2 + -3
        assert_eq!(tracker.alignment(), MoralAlignment::Dark);
    }

    #[test]
    fn test_moral_tracker_neutral_path() {
        let engine = make_test_story();
        let arc = engine.play("start", &[Trit::Neutral, Trit::Neutral]).unwrap();
        let tracker = arc.moral_tracker();

        assert_eq!(tracker.total_weight, 0);
        assert_eq!(tracker.alignment(), MoralAlignment::Neutral);
    }

    #[test]
    fn test_moral_tracker_mixed() {
        let engine = make_test_story();
        let arc = engine.play("start", &[Trit::Positive, Trit::Negative]).unwrap();
        let tracker = arc.moral_tracker();

        // +2 then -4
        assert_eq!(tracker.total_weight, -2);
        assert_eq!(tracker.choice_counts[0], 1); // one negative
        assert_eq!(tracker.choice_counts[2], 1); // one positive
    }

    #[test]
    fn test_moral_fraction() {
        let mut tracker = MoralTracker::new();
        tracker.record(Trit::Positive, 1);
        tracker.record(Trit::Positive, 1);
        tracker.record(Trit::Negative, -1);

        assert!((tracker.fraction(Trit::Positive) - 0.6667).abs() < 0.01);
        assert!((tracker.fraction(Trit::Negative) - 0.3333).abs() < 0.01);
        assert!((tracker.fraction(Trit::Neutral) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_alignment_boundaries() {
        let mut tracker = MoralTracker::new();
        tracker.total_weight = 5;
        tracker.choices_made = 5; // avg = 1.0 -> Good (not Virtuous, which needs > 1.0)
        assert_eq!(tracker.alignment(), MoralAlignment::Good);

        tracker.total_weight = 6;
        tracker.choices_made = 5; // avg = 1.2 -> Virtuous
        assert_eq!(tracker.alignment(), MoralAlignment::Virtuous);

        tracker.total_weight = 0;
        tracker.choices_made = 5; // avg = 0 -> Neutral
        assert_eq!(tracker.alignment(), MoralAlignment::Neutral);
    }

    #[test]
    fn test_arc_ternary_signature() {
        let engine = make_test_story();
        let arc = engine.play("start", &[Trit::Positive, Trit::Negative]).unwrap();
        let sig = arc.ternary_signature();

        assert_eq!(sig, vec![Trit::Positive, Trit::Negative]);
    }

    #[test]
    fn test_arc_total_moral_weight() {
        let engine = make_test_story();
        let arc = engine.play("start", &[Trit::Neutral, Trit::Neutral]).unwrap();

        assert_eq!(arc.total_moral_weight(), 0);
    }

    #[test]
    fn test_play_with_closure() {
        let engine = make_test_story();
        let arc = engine.play_with("start", |_step, _node| Trit::Positive).unwrap();

        assert!(arc.len() >= 2);
        assert!(arc.steps.iter().all(|s| s.choice == Trit::Positive));
    }

    #[test]
    fn test_play_with_varying_choices() {
        let engine = make_test_story();
        let choices = [Trit::Negative, Trit::Neutral, Trit::Positive];
        let arc = engine.play_with("start", |step, _| choices[step % 3]).unwrap();

        assert!(arc.len() >= 2);
    }

    #[test]
    fn test_generator_empty_pattern() {
        let gen = StoryGenerator::new();
        let engine = gen.generate(&[]);
        assert_eq!(engine.node_count(), 0);
    }

    #[test]
    fn test_generator_single_node() {
        let gen = StoryGenerator::new();
        let engine = gen.generate(&[PatternTrit::P]);
        assert_eq!(engine.node_count(), 2); // node + ending
        assert!(engine.get_node("node_0").is_some());
        assert!(engine.get_node("ending_0").is_some());
    }

    #[test]
    fn test_generator_multi_node() {
        let gen = StoryGenerator::new();
        let pattern = [PatternTrit::N, PatternTrit::Z, PatternTrit::P];
        let engine = gen.generate(&pattern);

        assert_eq!(engine.node_count(), 4); // 3 nodes + ending

        let arc = engine.play("node_0", &[Trit::Negative, Trit::Neutral, Trit::Positive]).unwrap();
        assert_eq!(arc.len(), 3);
    }

    #[test]
    fn test_generator_moral_trajectory() {
        let gen = StoryGenerator::new();
        let engine = gen.generate(&[PatternTrit::P, PatternTrit::P, PatternTrit::P]);

        // Play all positive choices
        let arc = engine.play("node_0", &[Trit::Positive, Trit::Positive, Trit::Positive]).unwrap();
        let tracker = arc.moral_tracker();
        assert_eq!(tracker.alignment(), MoralAlignment::Virtuous);
    }

    #[test]
    fn test_generator_dark_trajectory() {
        let gen = StoryGenerator::new();
        let engine = gen.generate(&[PatternTrit::N, PatternTrit::N, PatternTrit::N]);

        let arc = engine.play("node_0", &[Trit::Negative, Trit::Negative, Trit::Negative]).unwrap();
        let tracker = arc.moral_tracker();
        assert_eq!(tracker.alignment(), MoralAlignment::Dark);
    }

    #[test]
    fn test_generator_ending_only_on_matching_choice() {
        let gen = StoryGenerator::new();
        let engine = gen.generate(&[PatternTrit::P]);

        // At the last node, only positive leads to ending
        let node = engine.get_node("node_0").unwrap();
        let (_, neg_next) = node.choose(Trit::Negative);
        let (_, neu_next) = node.choose(Trit::Neutral);
        let (_, pos_next) = node.choose(Trit::Positive);

        assert!(neg_next.is_none());
        assert!(neu_next.is_none());
        assert!(pos_next.is_some());
    }

    #[test]
    fn test_consequence_branching_differs() {
        let engine = make_test_story();
        let start = engine.get_node("start").unwrap();

        let (neg, _) = start.choose(Trit::Negative);
        let (neu, _) = start.choose(Trit::Neutral);
        let (pos, _) = start.choose(Trit::Positive);

        // Each consequence has different moral weight
        assert!(neg.moral_weight < neu.moral_weight);
        assert!(neu.moral_weight < pos.moral_weight);
    }

    #[test]
    fn test_story_arc_empty() {
        let arc = StoryArc::new();
        assert!(arc.is_empty());
        assert_eq!(arc.len(), 0);
        assert_eq!(arc.total_moral_weight(), 0);
    }

    #[test]
    fn test_moral_tracker_empty() {
        let tracker = MoralTracker::new();
        assert_eq!(tracker.choices_made, 0);
        assert_eq!(tracker.total_weight, 0);
        assert!((tracker.average_weight() - 0.0).abs() < f64::EPSILON);
        assert_eq!(tracker.alignment(), MoralAlignment::Neutral);
    }

    #[test]
    fn test_moral_alignment_display() {
        assert_eq!(format!("{}", MoralAlignment::Dark), "Dark");
        assert_eq!(format!("{}", MoralAlignment::Virtuous), "Virtuous");
    }

    #[test]
    fn test_story_error() {
        let err = StoryError::NodeNotFound("missing".to_string());
        assert_eq!(format!("{}", err), "Node not found: missing");
    }

    #[test]
    fn test_engine_node_count() {
        let engine = make_test_story();
        assert_eq!(engine.node_count(), 7);
    }
}
