//! Direct replay assertions use the same closed review ledger as the CLI.
use silver_corpus::{
    Difference, ReviewedLayoutDivergence, read_manifest, verify_reviewed_layout_stream,
};
use std::{path::Path, sync::OnceLock};

pub fn reviewed_result(
    id: &str,
    result: Result<(), Difference>,
    bytes: &[u8],
) -> Result<(), Difference> {
    static REVIEWS: OnceLock<Vec<ReviewedLayoutDivergence>> = OnceLock::new();
    let reviews = REVIEWS.get_or_init(|| {
        let root = Path::new(
            option_env!("BAZEL_CARGO_MANIFEST_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR")),
        )
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
        read_manifest(&root.join("silver-corpus.toml"))
            .expect("read Silver review ledger")
            .corpus
            .reviewed_layout_divergences
    });
    let Some(review) = reviews.iter().find(|review| review.id == id) else {
        return result;
    };
    let difference = result.expect_err("reviewed CSS layout difference must remain present");
    verify_reviewed_layout_stream(review, &difference, bytes)
        .unwrap_or_else(|error| panic!("{id}: {error:#}"));
    Ok(())
}
