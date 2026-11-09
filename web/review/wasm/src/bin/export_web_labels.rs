use std::{env, error::Error, fs, path::PathBuf};

use jayjay_i18n::Language;

fn main() -> Result<(), Box<dyn Error>> {
    let output = PathBuf::from(env::args().nth(1).ok_or("expected output directory")?);
    fs::create_dir_all(&output)?;
    for tag in Language::all()
        .map(|language| language.tag)
        .chain(["en-XA"])
    {
        fs::write(
            output.join(format!("{tag}.json")),
            jj_diff_web::ui_labels(tag),
        )?;
    }
    Ok(())
}
