//! Tests for the dictated-intent parser (E021-T06).
//!
//! The centre of gravity is one table of real phrases in both languages. A
//! parser like this fails by *drifting* — a pattern loosened for one phrase
//! starts swallowing another — so the table asserts the full mapping at once
//! rather than one phrase per test.

#![cfg(test)]

use crate::voice_intent::{
    compiled_rule_count, parse, truncate_for_log, Intent, DEFAULT_SNOOZE_MINS,
    LOG_TRANSCRIPT_CHARS, MAX_SNOOZE_MINS, RULE_COUNT,
};

/// The twelve-phrase acceptance table: six Polish, six English, covering all
/// four intents and both snooze forms (with and without a number).
const TABLE: &[(&str, Intent)] = &[
    // ── Polish ──────────────────────────────────────────────────────────────
    ("drzemka 5", Intent::Snooze(5)),
    ("przypomnij mi za 10 minut", Intent::Snooze(10)),
    ("odłóż na 15 minut", Intent::Snooze(15)),
    ("idę na spacer", Intent::WalkStart),
    ("wracam ze spaceru", Intent::WalkEnd),
    ("kupić mleko po pracy", Intent::Note),
    // ── English ─────────────────────────────────────────────────────────────
    ("snooze 20", Intent::Snooze(20)),
    ("remind me in 30 minutes", Intent::Snooze(30)),
    ("snooze", Intent::Snooze(DEFAULT_SNOOZE_MINS)),
    ("going for a walk", Intent::WalkStart),
    ("back from my walk", Intent::WalkEnd),
    ("call the dentist tomorrow", Intent::Note),
];

#[test]
fn e021_t06_twelve_phrase_table_maps_to_the_expected_intents() {
    assert_eq!(TABLE.len(), 12, "the acceptance table must stay at 12 phrases");
    for (phrase, expected) in TABLE {
        assert_eq!(
            parse(phrase),
            *expected,
            "phrase {phrase:?} did not parse as {expected:?}"
        );
    }
}

/// Every rule in the table must compile. A pattern that fails to compile is
/// dropped at runtime, which would silently shrink the parser to a
/// note-only classifier — the classic "silence looks like success".
#[test]
fn e021_t06_every_rule_compiles() {
    assert!(RULE_COUNT > 0, "the rule table must not be empty");
    assert_eq!(
        compiled_rule_count(),
        RULE_COUNT,
        "a rule pattern failed to compile and was dropped"
    );
}

#[test]
fn e021_t06_diacritic_free_dictation_still_parses() {
    // Android keyboards routinely return Polish without diacritics.
    assert_eq!(parse("ide na spacer"), Intent::WalkStart);
    assert_eq!(parse("odloz na 15 minut"), Intent::Snooze(15));
    assert_eq!(parse("wrocilem ze spaceru"), Intent::WalkEnd);
}

#[test]
fn e021_t06_walk_end_wins_over_walk_start_when_both_vocabularies_appear() {
    // "wracam ze spaceru" carries the word "spacer" too — order in the rule
    // table is what keeps this from being read as leaving.
    assert_eq!(parse("wracam ze spaceru na obiad"), Intent::WalkEnd);
    assert_eq!(parse("koniec spaceru"), Intent::WalkEnd);
}

#[test]
fn e021_t06_snooze_length_is_clamped_not_rejected() {
    assert_eq!(parse("snooze 9000"), Intent::Snooze(MAX_SNOOZE_MINS));
    assert_eq!(parse("drzemka 0"), Intent::Snooze(1));
    assert_eq!(parse("drzemka 180"), Intent::Snooze(180));
}

#[test]
fn e021_t06_empty_and_whitespace_transcripts_are_notes() {
    assert_eq!(parse(""), Intent::Note);
    assert_eq!(parse("   \n\t "), Intent::Note);
}

#[test]
fn e021_t06_case_and_punctuation_do_not_change_the_intent() {
    assert_eq!(parse("DRZEMKA 5!"), Intent::Snooze(5));
    assert_eq!(parse("  Snooze 5.  "), Intent::Snooze(5));
}

#[test]
fn e021_t06_labels_are_stable_and_carry_no_minutes() {
    assert_eq!(Intent::Snooze(5).label(), "snooze");
    assert_eq!(Intent::Snooze(60).label(), "snooze");
    assert_eq!(Intent::Note.label(), "note");
    assert_eq!(Intent::WalkStart.label(), "walk_start");
    assert_eq!(Intent::WalkEnd.label(), "walk_end");
}

#[test]
fn e021_t06_log_truncation_counts_characters_not_bytes() {
    let long = "ą".repeat(LOG_TRANSCRIPT_CHARS + 40);
    let truncated = truncate_for_log(&long);
    assert_eq!(truncated.chars().count(), LOG_TRANSCRIPT_CHARS);

    let short = "drzemka 5";
    assert_eq!(truncate_for_log(short), short);
}

#[test]
fn e021_t06_log_truncation_flattens_newlines() {
    assert_eq!(truncate_for_log("first\nsecond"), "first second");
}
