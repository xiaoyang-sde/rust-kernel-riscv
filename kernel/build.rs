use std::fs::{read_dir, File};
use std::io::{Result, Write};

static TARGET_PATH: &str = "../kernel-lib/target/riscv64gc-unknown-none-bin/debug/";

fn main() {
    println!("cargo:rerun-if-changed={}", TARGET_PATH);
    insert_bin_data().unwrap();
}

fn insert_bin_data() -> Result<()> {
    let mut linkage_file = File::create("src/asm/linkage.asm").unwrap();
    let mut bin_vec: Vec<_> = read_dir("../kernel-lib/src/bin")
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    bin_vec.sort();

    writeln!(
        linkage_file,
        r#".align 3
    .section .data
    .global _bin_num
_bin_num:
    .quad {}"#,
        bin_vec.len()
    )?;

    for i in 0..bin_vec.len() {
        writeln!(linkage_file, r#"    .quad bin_{}_start"#, i)?;
    }
    writeln!(
        linkage_file,
        r#"    .quad bin_{}_end"#,
        bin_vec.len() - 1
    )?;

    for (i, bin) in bin_vec.iter().enumerate() {
        println!("bin_{}: {}", i, bin);
        writeln!(
            linkage_file,
            r#"
    .section .data
    .global bin_{0}_start
    .global bin_{0}_end
bin_{0}_start:
    .incbin "{2}{1}.bin"
bin_{0}_end:"#,
            i, bin, TARGET_PATH
        )?;
    }
    Ok(())
}
