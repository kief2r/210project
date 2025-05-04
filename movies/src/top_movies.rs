// Summary: Provides a function to retrieve the top-rated movies for a given user
// The top movies are selected based on the highest ratings, with ties broken by the most recent timestamp
// This is useful for showing a user’s favorite movies, reflecting both score and recency

use crate::Rating; // Imports the Rating struct from the main crate.

// top_movies
// Returns the top-N favorite movies (with ratings) for a specific user
// Inputs:
// - user_id: u32 --> the ID of the target user
// - ratings: &[Rating] --> a slice of all rating records
// - top_n: usize --> how many top movies to return
// Output:
// - Vec<(u32, f32)> --> a vector of (movie_id, rating) pairs, sorted by rating (desc), then recency (desc)
//
// High-level logic:
// 1. Filter ratings: keep only ratings belonging to the given user
// 2. Collect as (movie_id, rating, timestamp) triples
// 3. Sort the triples:
//     - Primary: rating descending
//     - Secondary (tie-breaker): timestamp descending
// 4. Take the first top_n entries
// 5. Return (movie_id, rating) pairs (drop the timestamp)
pub fn top_movies(user_id: u32, ratings: &[Rating], top_n: usize) -> Vec<(u32, f32)> {
    let mut seen: Vec<(u32, f32, u64)> = ratings
        .iter()
        .filter(|r| r.user_id == user_id)
        .map(|r| (r.movie_id, r.rating, r.timestamp))
        .collect();
    // sort by rating desc, then timestamp desc
    seen.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap()
            .then(b.2.cmp(&a.2))
    });
    // Take the top_n entries and map back to (movie_id, rating) only
    seen
        .into_iter()
        .take(top_n)
        .map(|(mid, rating, _)| (mid, rating))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rating;

    // Test: checks that top_movies correctly identifies the top-rated, most recent movies for user 2
    // We build a fixed dataset where user 2 has multiple 5.0 ratings, the test verifies that the function returns the most recent ones first
  
    #[test]
    fn test_top_movies_user2_real_data() {
        // All of user 2’s ratings:
        let ratings = vec![
            Rating {user_id: 2, movie_id: 318,    rating: 3.0, timestamp: 1445714835},
            Rating {user_id: 2, movie_id: 333,    rating: 4.0, timestamp: 1445715029},
            Rating {user_id: 2, movie_id: 1704,   rating: 4.5, timestamp: 1445715228},
            Rating {user_id: 2, movie_id: 3578,   rating: 4.0, timestamp: 1445714885},
            Rating {user_id: 2, movie_id: 6874,   rating: 4.0, timestamp: 1445714952},
            Rating {user_id: 2, movie_id: 8798,   rating: 3.5, timestamp: 1445714960},
            Rating {user_id: 2, movie_id: 46970,  rating: 4.0, timestamp: 1445715013},
            Rating {user_id: 2, movie_id: 48516,  rating: 4.0, timestamp: 1445715064},
            Rating {user_id: 2, movie_id: 58559,  rating: 4.5, timestamp: 1445715141},
            Rating {user_id: 2, movie_id: 60756,  rating: 5.0, timestamp: 1445714980},
            Rating {user_id: 2, movie_id: 68157,  rating: 4.5, timestamp: 1445715154},
            Rating {user_id: 2, movie_id: 71535,  rating: 3.0, timestamp: 1445714974},
            Rating {user_id: 2, movie_id: 74458,  rating: 4.0, timestamp: 1445714926},
            Rating {user_id: 2, movie_id: 77455,  rating: 3.0, timestamp: 1445714941},
            Rating {user_id: 2, movie_id: 79132,  rating: 4.0, timestamp: 1445714841},
            Rating {user_id: 2, movie_id: 80489,  rating: 4.5, timestamp: 1445715340},
            Rating {user_id: 2, movie_id: 80906,  rating: 5.0, timestamp: 1445715172},
            Rating {user_id: 2, movie_id: 86345,  rating: 4.0, timestamp: 1445715166},
            Rating {user_id: 2, movie_id: 89774,  rating: 5.0, timestamp: 1445715189},
            Rating {user_id: 2, movie_id: 91529,  rating: 3.5, timestamp: 1445714891},
            Rating {user_id: 2, movie_id: 91658,  rating: 2.5, timestamp: 1445714938},
            Rating {user_id: 2, movie_id: 99114,  rating: 3.5, timestamp: 1445714874},
            Rating {user_id: 2, movie_id: 106782, rating: 5.0, timestamp: 1445714966},
            Rating {user_id: 2, movie_id: 109487, rating: 3.0, timestamp: 1445715145},
            Rating {user_id: 2, movie_id: 112552, rating: 4.0, timestamp: 1445714882},
            Rating {user_id: 2, movie_id: 114060, rating: 2.0, timestamp: 1445715276},
            Rating {user_id: 2, movie_id: 115713, rating: 3.5, timestamp: 1445714854},
            Rating {user_id: 2, movie_id: 122882, rating: 5.0, timestamp: 1445715272},
            Rating {user_id: 2, movie_id: 131724, rating: 5.0, timestamp: 1445714851},
        ];

        // When we ask for the top 4, we should get the four 5.0’s with the highest timestamps (descending).
        // This makes it so more recent movie ratings are preferred! 
        let top = top_movies(2, &ratings, 4);
        assert_eq!(top, vec![
            (122882, 5.0),
            ( 89774, 5.0),
            ( 80906, 5.0),
            ( 60756, 5.0),
        ]);
    }
}
