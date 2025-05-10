//! Utility function to stall an iterator

/// The [stalling] function wraps an iterator into one that "stalls" at random,
/// returning either `Some(t)` (where `t` is the value yielded by the underlying iterator)
/// or `None`.  The probability of a stall is a parameter `prob`.  
pub fn stalling<S>(mut s: S, prob: f64) -> impl Iterator<Item = Option<<S as Iterator>::Item>>
where
    S: Iterator,
{
    assert!(
        prob < 1.0,
        "Stalling with probability >= 1.0 is not supported, and probably not what you mean"
    );
    std::iter::from_fn(move || {
        Some(if rand::random::<f64>() < prob {
            // Stall the generator and return None
            None
        } else {
            s.next()
        })
    })
}
