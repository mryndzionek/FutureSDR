use anyhow::{Context, Result};
use std::iter::repeat_with;
use std::time;

use futuresdr::blocks::CopyBuilder;
use futuresdr::blocks::VectorSink;
use futuresdr::blocks::VectorSinkBuilder;
use futuresdr::blocks::VectorSourceBuilder;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;

fn main() -> Result<()> {
    let mut fg = Flowgraph::new();

    let n_items = 20_000;
    let n_copy = 1000;

    let orig: Vec<f32> = repeat_with(rand::random::<f32>).take(n_items).collect();

    let src = fg.add_block(VectorSourceBuilder::new(orig.clone()).build());
    let snk = fg.add_block(
        VectorSinkBuilder::<f32>::new()
            .init_capacity(n_items)
            .build(),
    );

    let mut prev = 0;
    for i in 0..n_copy {
        let t = fg.add_block(CopyBuilder::new(4).build());

        if i == 0 {
            fg.connect_stream(src, "out", t, "in")?;
        } else {
            fg.connect_stream(prev, "out", t, "in")?;
        }
        prev = t;
    }

    fg.connect_stream(prev, "out", snk, "in")?;

    let now = time::Instant::now();
    fg = Runtime::new().run(fg)?;
    let elapsed = now.elapsed();

    let snk = fg
        .block_async::<VectorSink<f32>>(snk)
        .context("block not found")?;
    let v = snk.items();

    assert_eq!(v.len(), n_items);
    for i in 0..v.len() {
        assert!((orig[i] - v[i]).abs() < f32::EPSILON);
    }

    println!("flowgraph took {:?}", elapsed);

    Ok(())
}
