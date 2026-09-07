//! voice_intent.rs — turns a dictated sentence into one [`Intent`].
//!
//! The phone dictates free text; this module decides whether that text asks
//! the app to *do* something. It is deliberately small and local: no network,
//! no model, no state. An AI reply may be added on top later
//! ([`crate::voice_ai`]), but the intent itself must be decided offline, or a
//! missing API key would silently disable "drzemka 5".
//!
//! ## Recognition is a table, not a chain of `if`s
//! [`RULES`] is an ordered list of `(kind, regex)` pairs matched against the
//! lowercased transcript. Order is meaningful — "wracam ze spaceru" contains
//! the walk vocabulary of both directions, so [`Kind::WalkEnd`] is tested
//! before [`Kind::WalkStart`].
//!
//! ## Anything unrecognised is a note, never an error
//! [`Intent::Note`] is the fallback. A dictation the parser does not
//! understand is still worth keeping — it is stored and acknowledged like any
//! other note. There is no "unparseable" outcome for the caller to handle.

use std::sync::OnceLock;

use regex::Regex;

/// Snooze length used when the user said "drzemka" without a number.
pub const DEFAULT_SNOOZE_MINS: u16 = 5;

/// Shortest snooze a transcript can ask for.
pub const MIN_SNOOZE_MINS: u16 = 1;

/// Longest snooze a transcript can ask for. A dictated "snooze 9000" is a
/// misheard number, not a three-day request.
pub const MAX_SNOOZE_MINS: u16 = 180;

/// What the user asked the app to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    /// Silence escalation for N minutes.
    Snooze(u16),
    /// Nothing to do — keep the text.
    Note,
    /// The user is leaving for a walk.
    WalkStart,
    /// The user is back from a walk.
    WalkEnd,
}

impl Intent {
    /// Stable machine label, used in the event log, the `voice_notes` table
    /// and the `VoiceAck` payload.
    ///
    /// A label is deliberately *not* `{:?}` — `Snooze(5)` would then leak the
    /// minute count into a field consumers group by.
    pub fn label(&self) -> &'static str {
        match self {
            Intent::Snooze(_) => "snooze",
            Intent::Note => "note",
            Intent::WalkStart => "walk_start",
            Intent::WalkEnd => "walk_end",
        }
    }
}

/// The intent families the table can recognise. `Note` is absent on purpose:
/// it is the fallback, so no rule can produce it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Snooze,
    WalkEnd,
    WalkStart,
}

/// One recognition rule: what it means, and the pattern that spots it.
struct Rule {
    kind: Kind,
    pattern: &'static str,
}

/// The rule table, in match order. Polish and English share one table rather
/// than branching on `lang`, because the phone's dictation language and the
/// user's language do not reliably agree.
const RULES: &[Rule] = &[
    Rule {
        kind: Kind::Snooze,
        pattern: r"drzemk\w*|odl[oó]z\w*|odł[oó][zż]\w*|przypomnij|ucisz|snooze|remind me",
    },
    Rule {
        kind: Kind::WalkEnd,
        pattern: r"wracam|wr[oó]ci[lł]\w*|koniec spaceru|po spacerze|jestem z powrotem|back from|end(?:ing)? (?:my )?walk|walk (?:done|over|ended|finished)|finished (?:my )?walk|i'?m back",
    },
    Rule {
        kind: Kind::WalkStart,
        pattern: r"id[eę] (?:na )?spacer|id[eę] si[eę] przej|na spacer\b|zaczynam spacer|wychodz[eę]|going (?:out )?for a walk|start(?:ing)? (?:my )?walk|walk start|taking a walk",
    },
];

/// Compiled form of [`RULES`], built once on first use.
///
/// A rule whose pattern does not compile is dropped with a loud log line
/// rather than panicking the HTTP handler that called us — but the count is
/// asserted in the tests, so a dropped rule cannot pass unnoticed.
fn compiled() -> &'static [(Kind, Regex)] {
    static COMPILED: OnceLock<Vec<(Kind, Regex)>> = OnceLock::new();
    COMPILED.get_or_init(|| {
        RULES
            .iter()
            .filter_map(|rule| match Regex::new(rule.pattern) {
                Ok(re) => Some((rule.kind, re)),
                Err(e) => {
                    log::error!("voice_intent: rule {:?} failed to compile: {e}", rule.kind);
                    None
                }
            })
            .collect()
    })
}

/// First run of digits anywhere in the transcript.
fn number() -> &'static Regex {
    static NUMBER: OnceLock<Regex> = OnceLock::new();
    NUMBER.get_or_init(|| Regex::new(r"\d{1,4}").expect("literal digit pattern compiles"))
}

/// Classifies one dictated transcript.
///
/// Matching runs on the lowercased text; the caller keeps the original for
/// storage. Unknown text — including empty text — is [`Intent::Note`].
pub fn parse(transcript: &str) -> Intent {
    let text = transcript.trim().to_lowercase();
    if text.is_empty() {
        return Intent::Note;
    }

    for (kind, re) in compiled() {
        if !re.is_match(&text) {
            continue;
        }
        return match kind {
            Kind::Snooze => Intent::Snooze(snooze_minutes(&text)),
            Kind::WalkEnd => Intent::WalkEnd,
            Kind::WalkStart => Intent::WalkStart,
        };
    }
    Intent::Note
}

/// Reads the requested snooze length out of `text`, clamped to a sane range.
///
/// No number at all means the user said only "drzemka" — a request without a
/// length, which gets [`DEFAULT_SNOOZE_MINS`] rather than being downgraded to
/// a note.
fn snooze_minutes(text: &str) -> u16 {
    number()
        .find(text)
        .and_then(|m| m.as_str().parse::<u32>().ok())
        .map(|n| u16::try_from(n.clamp(u32::from(MIN_SNOOZE_MINS), u32::from(MAX_SNOOZE_MINS)))
            .unwrap_or(MAX_SNOOZE_MINS))
        .unwrap_or(DEFAULT_SNOOZE_MINS)
}

/// Longest transcript accepted by the route, mirrored here because the
/// truncation for the event log is this module's concern.
pub const LOG_TRANSCRIPT_CHARS: usize = 80;

/// Shortens a transcript to [`LOG_TRANSCRIPT_CHARS`] characters for the event
/// log. Counts characters, not bytes — a Polish transcript truncated on a byte
/// boundary would not be valid UTF-8.
pub fn truncate_for_log(transcript: &str) -> String {
    let single_line = transcript.replace(['\n', '\r'], " ");
    let trimmed = single_line.trim();
    if trimmed.chars().count() <= LOG_TRANSCRIPT_CHARS {
        return trimmed.to_string();
    }
    trimmed.chars().take(LOG_TRANSCRIPT_CHARS).collect()
}

/// Number of rules the table declares — asserted by the tests so a rule that
/// fails to compile cannot quietly shrink the parser.
pub const RULE_COUNT: usize = RULES.len();

/// Number of rules that actually compiled.
pub fn compiled_rule_count() -> usize {
    compiled().len()
}

// Tests live in voice_intent_tests.rs.
