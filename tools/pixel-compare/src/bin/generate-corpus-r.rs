use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

const GENERATED_MARKER: &str = "# BEGIN GENERATED UPSTREAM GM ENTRIES";

fn main() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".to_owned()));
    let manifest_path = root.join("corpus-r.toml");
    let existing = fs::read_to_string(&manifest_path)?;
    let prefix = existing
        .split_once(GENERATED_MARKER)
        .map(|(prefix, _)| prefix.trim_end())
        .unwrap_or(existing.trim_end());
    let mut preserved = preserved_generated_blocks(&existing)?;
    let stream_dir = root.join("fixtures/renderer/streams/gm");
    let mut names = fs::read_dir(&stream_dir)?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension()?.to_str()? == "rive-stream")
                .then(|| path.file_stem()?.to_str().map(str::to_owned))?
        })
        .collect::<Vec<_>>();
    names.sort();

    let mut output = format!("{prefix}\n\n{GENERATED_MARKER}\n");
    for name in &names {
        for mode in ["clockwise-atomic", "msaa"] {
            let gate = if mode == "msaa" {
                "gated = \"reference-harness: C++ Metal does not implement MSAA flush\"\n"
            } else {
                "gated = \"algorithm-core\"\n"
            };
            let id = format!("gm-{name}-{mode}");
            let generated = format!(
                r#"
[[entry]]
id = "{id}"
kind = "upstream-gm-stream"
stream = "fixtures/renderer/streams/gm/{name}.rive-stream"
reference = "fixtures/renderer/reference/metal/gm/{name}-{mode}.png"
mode = "{mode}"
backend = "metal"
status = "gated"
max_channel_delta = 2
max_different_pixels = 32
{gate}
"#
            );
            push_preserved_or_generated(&mut output, &mut preserved, &id, generated);
        }
    }
    let riv_stream_dir = root.join("fixtures/renderer/streams/riv");
    if riv_stream_dir.exists() {
        let mut riv_streams = fs::read_dir(&riv_stream_dir)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("rive-stream"))
            .collect::<Vec<_>>();
        riv_streams.sort();
        for stream_path in riv_streams {
            let name = stream_path
                .file_stem()
                .and_then(|value| value.to_str())
                .ok_or("bad .riv stream name")?;
            let stream =
                nuxie_render_stream::RenderStream::parse(&fs::read_to_string(&stream_path)?)?;
            for frame in 0..stream.frames.len() {
                for mode in ["clockwise-atomic", "msaa"] {
                    let gate = if mode == "msaa" {
                        "reference-harness: C++ Metal does not implement MSAA flush"
                    } else {
                        "algorithm-core"
                    };
                    let id = format!("riv-{name}-frame-{frame}-{mode}");
                    let generated = format!(
                        r#"
[[entry]]
id = "{id}"
kind = "riv-stream"
stream = "fixtures/renderer/streams/riv/{name}.rive-stream"
reference = "fixtures/renderer/reference/metal/riv/{name}-frame-{frame}-{mode}.png"
mode = "{mode}"
backend = "metal"
frame = {frame}
status = "gated"
max_channel_delta = 2
max_different_pixels = 32
gated = "{gate}"
"#
                    );
                    push_preserved_or_generated(&mut output, &mut preserved, &id, generated);
                }
            }
        }
    }
    fs::write(&manifest_path, output)?;
    println!(
        "generated {} GM entries from {} streams",
        names.len() * 2,
        names.len()
    );
    Ok(())
}

fn preserved_generated_blocks(existing: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let Some((_, generated)) = existing.split_once(GENERATED_MARKER) else {
        return Ok(HashMap::new());
    };
    let mut blocks = HashMap::new();
    for body in generated.split("\n[[entry]]\n").skip(1) {
        let block = format!("[[entry]]\n{body}");
        let value: toml::Value = toml::from_str(&block)?;
        let id = value
            .get("entry")
            .and_then(toml::Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(|entry| entry.get("id"))
            .and_then(toml::Value::as_str)
            .ok_or("generated corpus entry has no string id")?;
        blocks.insert(id.to_owned(), block);
    }
    Ok(blocks)
}

fn push_preserved_or_generated(
    output: &mut String,
    preserved: &mut HashMap<String, String>,
    id: &str,
    generated: String,
) {
    if let Some(block) = preserved.remove(id) {
        output.push('\n');
        output.push_str(&block);
    } else {
        output.push_str(&generated);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_existing_generated_entry_block_by_id() {
        let existing = format!(
            "prefix\n\n{GENERATED_MARKER}\n\n[[entry]]\nid = \"gm-a-clockwise-atomic\"\nstatus = \"exact\"\nmax_channel_delta = 7\n"
        );
        let mut preserved = preserved_generated_blocks(&existing).unwrap();
        let mut output = format!("prefix\n\n{GENERATED_MARKER}\n");

        push_preserved_or_generated(
            &mut output,
            &mut preserved,
            "gm-a-clockwise-atomic",
            "generated default must not appear".to_owned(),
        );

        assert!(output.contains("status = \"exact\""));
        assert!(output.contains("max_channel_delta = 7"));
        assert!(!output.contains("generated default must not appear"));
        assert_eq!(output, existing);
    }
}
