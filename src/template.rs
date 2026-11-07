use anyhow::{anyhow, ensure, Context as _, Result};
use log::*;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tera::{Context, Tera};

use crate::utils::Perms;

/// Recursively list the files below `dir` as paths relative to `rel`, ignoring dotfiles.
fn list_files(dir: &Path, rel: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }
        let rel_path = rel.join(&name);
        if entry.path().is_dir() {
            list_files(&entry.path(), &rel_path, out)?;
        } else {
            out.push(rel_path);
        }
    }
    Ok(())
}

fn has_extension(path: &Path, extensions: &[String]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)))
        .unwrap_or(false)
}

/// Partials (any path component starting with `_`) are never written to the output.
fn is_partial(path: &Path) -> bool {
    path.components()
        .any(|c| c.as_os_str().to_string_lossy().starts_with('_'))
}

/// Render the template directory into the output directory: files with one of the given
/// extensions go through Tera, everything else is copied verbatim.
pub fn render_site(
    template_dir: &Path,
    output_dir: &Path,
    context: &Context,
    extensions: &[String],
    perms: &Perms,
) -> Result<()> {
    ensure!(
        template_dir.is_dir(),
        "{} does not exist or is not a directory",
        template_dir.display()
    );

    let extensions: Vec<String> = extensions
        .iter()
        .map(|e| e.trim_start_matches('.').to_owned())
        .collect();

    let mut files = Vec::new();
    list_files(template_dir, Path::new(""), &mut files)?;
    files.sort();

    // register every template (partials included) so extends/include can resolve them
    let mut tera = Tera::default();
    let mut to_register = Vec::new();
    for rel in files.iter().filter(|f| has_extension(f, &extensions)) {
        let name = rel
            .to_str()
            .ok_or_else(|| anyhow!("{} is not valid UTF-8", rel.display()))?;
        to_register.push((template_dir.join(rel), Some(name)));
    }
    tera.add_template_files(to_register)?;

    for rel in files.iter().filter(|f| !is_partial(f)) {
        let destination = output_dir.join(rel);
        if has_extension(rel, &extensions) {
            let name = rel.to_string_lossy();
            debug!("rendering {}", destination.display());
            let rendered = tera
                .render(&name, context)
                .with_context(|| format!("cannot render {}", name))?;
            perms.write(&destination, rendered)?;
        } else {
            debug!("copying {}", destination.display());
            perms.write(&destination, fs::read(template_dir.join(rel))?)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_templates_and_copies_the_rest() {
        let root = std::env::temp_dir().join(format!("clubmirror-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let (tpl, out) = (root.join("tpl"), root.join("out"));
        fs::create_dir_all(tpl.join("css")).unwrap();
        fs::create_dir_all(tpl.join("_partials")).unwrap();
        fs::write(tpl.join("_base.html"), "<b>{% block c %}{% endblock %}</b>").unwrap();
        fs::write(
            tpl.join("index.html"),
            "{% extends \"_base.html\" %}{% block c %}{{ club_name }}{% endblock %}",
        )
        .unwrap();
        fs::write(tpl.join("_partials/x.html"), "hidden").unwrap();
        fs::write(
            tpl.join("css/site.css"),
            "a { color: red } {{ not_a_template }}",
        )
        .unwrap();
        fs::write(tpl.join(".hidden"), "x").unwrap();

        let mut context = Context::new();
        context.insert("club_name", "Test");
        let perms = Perms {
            mode: 0o644,
            uid: None,
            gid: None,
        };
        let extensions = vec!["html".to_string()];
        render_site(&tpl, &out, &context, &extensions, &perms).unwrap();

        assert_eq!(
            fs::read_to_string(out.join("index.html")).unwrap(),
            "<b>Test</b>"
        );
        assert_eq!(
            fs::read_to_string(out.join("css/site.css")).unwrap(),
            "a { color: red } {{ not_a_template }}"
        );
        assert!(!out.join("_base.html").exists());
        assert!(!out.join("_partials").exists());
        assert!(!out.join(".hidden").exists());
        fs::remove_dir_all(&root).unwrap();
    }
}
