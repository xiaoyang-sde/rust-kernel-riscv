use std::fs::{read_dir, File};
use std::io::{Result, Write};

static TARGET_PATH: &str = "../kernel-lib/target/riscv64gc-unknown-none-elf/debug/";

fn main() {
    println!("cargo:rerun-if-changed={TARGET_PATH}");
    insert_bin_data().unwrap();
}

fn insert_bin_data() -> Result<()> {
    let mut linkage_file = File::create("src/asm/linkage.asm").unwrap();
    let mut bin_vec: Vec<_> = read_dir("../kernel-lib/src/bin")
        .unwrap()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    bin_vec.sort();

    writeln!(
        linkage_file,
        r#"    .align 3
    .section .data
    .global _bin_count
    .global _bin_address
    .global _bin_name

_bin_count:
    .quad {}

_bin_address:"#,
        bin_vec.len()
    )?;

    for i in 0..bin_vec.len() {
        writeln!(
            linkage_file,
            r#"    .quad bin_{i}_start
    .quad bin_{i}_end"#
        )?;
    }

    writeln!(
        linkage_file,
        r#"
_bin_name:"#
    )?;

    for bin in bin_vec.iter() {
        writeln!(linkage_file, r#"    .string "{}""#, bin)?;
    }

    for (i, bin) in bin_vec.iter().enumerate() {
        writeln!(
            linkage_file,
            r#"
    .section .data
    .global bin_{i}_start
    .global bin_{i}_end
    .align 3
bin_{i}_start:
    .incbin "{TARGET_PATH}{bin}"
bin_{i}_end:"#
        )?;
    }
    Ok(())
}
