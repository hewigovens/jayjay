/// Fuzzy-rank `candidates` against `query`; returns matching indices, best first.
#[uniffi::export]
fn fuzzy_rank(query: String, candidates: Vec<String>) -> Vec<u32> {
    jayjay_core::fuzzy::rank(&query, &candidates)
}
