//! Screen break nudge — random encouraging message when standing too long at screen.

use rand::seq::IndexedRandom;

/// Select a random message from the nudge pool.
/// Returns None if pool is empty.
pub fn pick_nudge_message(messages: &[String]) -> Option<String> {
    let mut rng = rand::rng();
    messages.choose(&mut rng).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_from_empty_pool_returns_none() {
        assert!(pick_nudge_message(&[]).is_none());
    }

    #[test]
    fn pick_from_pool_returns_some() {
        let pool = vec!["msg1".to_string(), "msg2".to_string()];
        let result = pick_nudge_message(&pool);
        assert!(result.is_some());
        assert!(pool.contains(&result.unwrap()));
    }
}
