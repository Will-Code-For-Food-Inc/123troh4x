use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tsukumogami::ops;

fn bench_build_cmd(c: &mut Criterion) {
    c.bench_function("build_cmd_all_args", |b| {
        b.iter(|| {
            let _ = ops::build_cmd(black_box(Some("all")), black_box(Some(8)));
        })
    });
}

fn bench_disassemble_cmd(c: &mut Criterion) {
    c.bench_function("disassemble_cmd_full", |b| {
        b.iter(|| {
            let _ = ops::disassemble_cmd(
                black_box("rom.elf"),
                black_box(Some(0x8000)),
                black_box(Some(0x100)),
            );
        })
    });
}

fn bench_grep_cmd(c: &mut Criterion) {
    c.bench_function("grep_cmd_full", |b| {
        b.iter(|| {
            let _ = ops::grep_cmd(
                black_box("TODO"),
                black_box(Some("src/")),
                black_box(true),
            );
        })
    });
}

fn bench_hexdump_cmd(c: &mut Criterion) {
    c.bench_function("hexdump_cmd_full", |b| {
        b.iter(|| {
            let _ = ops::hexdump_cmd(
                black_box("rom.bin"),
                black_box(Some(0x100)),
                black_box(Some(256)),
            );
        })
    });
}

criterion_group!(benches, bench_build_cmd, bench_disassemble_cmd, bench_grep_cmd, bench_hexdump_cmd);
criterion_main!(benches);
