use crate::Event;
use anyhow::{ensure, Result};
use log::*;
use std::{
    fs::{self, Permissions},
    io::Write,
    os::unix::fs::{chown, PermissionsExt},
    path::{Path, PathBuf},
};
use tera::{Context, Tera};

pub struct Template {
    context: Context,
    templates: PathBuf,
    generated: PathBuf,

    permissions: u32,
    uid: Option<u32>,
    gid: Option<u32>,
}

impl Template {
    pub fn new(
        templates: &Path,
        generated: &Path,
        route_url: &str,
        events: &Vec<Event>,
        club_id: u64,
        permissions: u32,
        uid: Option<u32>,
        gid: Option<u32>,
    ) -> Result<Self> {
        // check that path exists
        ensure!(
            templates.exists() && templates.is_dir(),
            "{} does not exist or is not a directory",
            templates.display()
        );

        let mut context = Context::new();

        context.insert("club_id", &club_id);
        context.insert("events", events);
        context.insert("routes_url", route_url);
        context.insert(
            "generated_at",
            &chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
        );
        context.insert("total_events", &events.len());

        Ok(Self {
            templates: templates.to_path_buf(),
            generated: generated.to_path_buf(),
            context,
            permissions,
            uid,
            gid,
        })
    }

    pub fn generate(&self) -> Result<()> {
        /*
        to_string_lossy() may or may not replace the string with ? for invalid UTF-8 character,
        hence returning a borrow of the original string, or a owned new string (with character
        replaced), hence a Cow<str>
        */
        let pattern = self.templates.join("**/*").to_string_lossy().to_string();
        let tera = Tera::new(&pattern)?;
        debug!("applying templates in {}", &pattern);

        for template in tera.get_template_names() {
            let rendered = tera.render(template, &self.context)?;

            let template_relative_path = Path::new(template);
            let template_generated_path = self.generated.join(template_relative_path);
            debug!("rendering {}", template_generated_path.display());

            // create directories in generated path if needed
            if let Some(parent) = template_generated_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut file = fs::File::create(&template_generated_path)?;
            file.write_all(rendered.as_bytes())?;
            file.set_permissions(Permissions::from_mode(self.permissions))?;
            drop(file);
            chown(template_generated_path, self.uid, self.gid)?;
        }

        Ok(())
    }
}
