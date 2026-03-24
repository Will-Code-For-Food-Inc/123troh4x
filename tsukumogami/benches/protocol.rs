use criterion::{black_box, criterion_group, criterion_main, Criterion};
use protocol::{Op, Request, Response};

fn bench_op_serialize(c: &mut Criterion) {
    let ops = vec![
        Op::Build { target: Some("rom.gba".into()), jobs: Some(4) },
        Op::Clean,
        Op::Check { tool: "gcc-arm-none-eabi".into() },
        Op::Disassemble { file: "rom.elf".into(), offset: Some(0x8000), length: Some(0x100) },
        Op::HexDump { file: "rom.bin".into(), offset: Some(0x10), length: Some(64) },
        Op::Grep { pattern: "TODO".into(), path: Some("src/".into()), recursive: Some(true) },
        Op::GitStatus,
        Op::GitDiff { file: Some("main.c".into()) },
        Op::ListOps,
    ];

    c.bench_function("op_serialize_all_variants", |b| {
        b.iter(|| {
            for op in &ops {
                let _ = serde_json::to_string(black_box(op)).unwrap();
            }
        })
    });
}

fn bench_op_deserialize(c: &mut Criterion) {
    let jsons = vec![
        r#"{"op":"build","target":"rom.gba","jobs":4}"#,
        r#"{"op":"clean"}"#,
        r#"{"op":"check","tool":"gcc-arm-none-eabi"}"#,
        r#"{"op":"disassemble","file":"rom.elf","offset":32768,"length":256}"#,
        r#"{"op":"hex_dump","file":"rom.bin","offset":16,"length":64}"#,
        r#"{"op":"grep","pattern":"TODO","path":"src/","recursive":true}"#,
        r#"{"op":"git_status"}"#,
        r#"{"op":"git_diff","file":"main.c"}"#,
        r#"{"op":"list_ops"}"#,
    ];

    c.bench_function("op_deserialize_all_variants", |b| {
        b.iter(|| {
            for json in &jsons {
                let _ = serde_json::from_str::<Op>(black_box(json)).unwrap();
            }
        })
    });
}

fn bench_request_roundtrip(c: &mut Criterion) {
    let req = Request {
        id: "bench-id-1234".into(),
        workdir: Some("/dshax/vendor/pokeplatinum".into()),
        op: Op::Build { target: Some("all".into()), jobs: Some(8) },
    };

    c.bench_function("request_roundtrip", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&req)).unwrap();
            let _: Request = serde_json::from_str(black_box(&json)).unwrap();
        })
    });
}

fn bench_response_serialize(c: &mut Criterion) {
    let resp = Response::success(
        "bench-id",
        "arm-none-eabi-gcc (GNU Arm Embedded Toolchain 10.3-2021.10) 10.3.1 20210824\n".into(),
        "".into(),
        0,
    );

    c.bench_function("response_serialize", |b| {
        b.iter(|| {
            let _ = serde_json::to_string(black_box(&resp)).unwrap();
        })
    });
}

criterion_group!(benches, bench_op_serialize, bench_op_deserialize, bench_request_roundtrip, bench_response_serialize);
criterion_main!(benches);
