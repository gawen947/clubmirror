//! Template filters that Tera 2 does not provide anymore: `date` and `filter`.

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use std::fmt::Write;
use tera::{Error, Kwargs, State, Tera, TeraResult, Value};

/**
 * `{{ ts | date(format="%Y-%m-%d %H:%M", timezone="Europe/Brussels") }}`
 * Formats an RFC 3339 timestamp with an optional timezone (UTC by default).
 */
fn date(val: &str, kwargs: Kwargs, _: &State) -> TeraResult<String> {
    let format = kwargs.get::<&str>("format")?.unwrap_or("%Y-%m-%d");
    let timezone = kwargs.get::<&str>("timezone")?;

    let parsed = DateTime::parse_from_rfc3339(val)
        .map_err(|e| Error::message(format!("date: cannot parse `{val}`: {e}")))?
        .with_timezone(&Utc);

    let mut out = String::new();
    let written = match timezone {
        Some(name) => {
            let tz: Tz = name
                .parse()
                .map_err(|_| Error::message(format!("date: unknown timezone `{name}`")))?;
            write!(out, "{}", parsed.with_timezone(&tz).format(format))
        }
        None => write!(out, "{}", parsed.format(format)),
    };
    written.map_err(|_| Error::message(format!("date: invalid format `{format}`")))?;

    Ok(out)
}

/**
 * `{{ events | filter(attribute="private", value=false) }}`
 * Keeps the items of a list whose `attribute` equals `value`.
 */
fn filter(val: &[Value], kwargs: Kwargs, _: &State) -> TeraResult<Vec<Value>> {
    let attribute = kwargs.must_get::<&str>("attribute")?;
    let wanted = kwargs.must_get::<Value>("value")?;

    Ok(val
        .iter()
        .filter(|item| item.get_from_path(attribute) == Some(&wanted))
        .cloned()
        .collect())
}

/// Register the clubmirror filters.
pub fn register(tera: &mut Tera) {
    tera.register_filter("date", date);
    tera.register_filter("filter", filter);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tera::Context;

    fn render(template: &str) -> String {
        let mut tera = Tera::default();
        register(&mut tera);
        let mut context = Context::new();
        context.insert("ts", "2026-06-06T08:00:00Z");
        context.insert(
            "items",
            &serde_json::json!([{"n": 1, "private": false}, {"n": 2, "private": true}, {"n": 3, "private": false}]),
        );
        tera.render_str(template, &context, false).unwrap()
    }

    #[test]
    fn date_uses_the_timezone() {
        assert_eq!(render(r#"{{ ts | date(format="%H:%M") }}"#), "08:00");
        assert_eq!(
            render(r#"{{ ts | date(format="%a %d %b %H:%M", timezone="Europe/Brussels") }}"#),
            "Sat 06 Jun 10:00"
        );
    }

    #[test]
    fn filter_keeps_matching_items() {
        assert_eq!(
            render(
                r#"{% for i in items | filter(attribute="private", value=false) %}{{ i.n }}{% endfor %}"#
            ),
            "13"
        );
    }
}
