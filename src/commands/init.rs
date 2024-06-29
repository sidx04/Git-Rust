use std::fs;

pub(crate) fn invoke() -> anyhow::Result<()> {
    /*
    Folder structure:
     - .git/
       - objects/
       - refs/
       - HEAD (should contain "ref: refs/heads/main\n" for a new repository)
    */

    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
    println!("Initialized git directory");

    Ok(())
}
