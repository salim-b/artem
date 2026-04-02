use clap_complete::{
    generate_to,
    shells::{Bash, Elvish, Fish, PowerShell, Zsh},
    Generator,
};
use std::ffi::OsString;
use std::{env, fs, path};

use std::io::Error;

include!("src/cli.rs");
//from https://docs.rs/clap_complete/3.0.6/clap_complete/generator/fn.generate_to.html
fn main() -> Result<(), Error> {
    println!("cargo:rerun-if-changed=src/cli.rs");

    let out_dir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(dir) => dir,
    };

    let mut cmd = build_cli();
    //this is only generated when the git ref changes???
    let bash_path = generate_shell_completion(&mut cmd, &out_dir, Bash)?;
    let powershell_path = generate_shell_completion(&mut cmd, &out_dir, PowerShell)?;
    let zsh_path = generate_shell_completion(&mut cmd, &out_dir, Zsh)?;
    let fish_path = generate_shell_completion(&mut cmd, &out_dir, Fish)?;
    let elvish_path = generate_shell_completion(&mut cmd, &out_dir, Elvish)?;

    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    let man_page_path = path::PathBuf::from(&out_dir).join("artem.1");

    std::fs::write(&man_page_path, buffer)?;

    println!("cargo:warning=man page is generated: {:?}", man_page_path);

    // When LIBRARY_PREFIX is set (e.g. during conda/pixi builds), copy the
    // generated completions and man page to the standard share directories
    // under that prefix so they are available in `pixi shell` etc.
    if let Ok(prefix) = env::var("LIBRARY_PREFIX") {
        let prefix = path::PathBuf::from(prefix);

        let copies: &[(&path::Path, &[&str])] = &[
            (&bash_path, &["share", "bash-completion", "completions"]),
            (&fish_path, &["share", "fish", "vendor_completions.d"]),
            (&zsh_path, &["share", "zsh", "vendor-completions"]),
            (&powershell_path, &["share", "powershell", "completions"]),
            (&elvish_path, &["share", "elvish", "completions"]),
            (&man_page_path, &["share", "man", "man1"]),
        ];

        for (src, dest_parts) in copies {
            let dest_dir: path::PathBuf = dest_parts.iter().fold(prefix.clone(), |p, s| p.join(s));
            fs::create_dir_all(&dest_dir)?;
            let file_name = src.file_name().expect("source path has a file name");
            let dest = dest_dir.join(file_name);
            fs::copy(src, &dest)?;
            println!(
                "cargo:warning=copied {:?} to {:?}",
                file_name, dest
            );
        }
    }

    Ok(())
}

fn generate_shell_completion<T>(
    cmd: &mut Command,
    out_dir: &OsString,
    shell: T,
) -> Result<PathBuf, Error>
where
    T: Generator,
{
    //generate shell completions
    let path = generate_to(
        shell, cmd,     // We need to specify what generator to use
        "artem", // We need to specify the bin name manually
        out_dir, // We need to specify where to write to
    )?;
    println!("cargo:warning=completion file is generated: {:?}", &path);
    Ok(path)
}
