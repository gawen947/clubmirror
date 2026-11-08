# clubmirror

[![Crates.io](https://img.shields.io/crates/v/clubmirror)](https://crates.io/crates/clubmirror)
[![Tests](https://github.com/gawen947/clubmirror/actions/workflows/tests.yml/badge.svg)](https://github.com/gawen947/clubmirror/actions)
[![License](https://img.shields.io/crates/l/clubmirror)](https://github.com/gawen947/clubmirror/blob/main/LICENSE)

**A static website generator that mirrors Strava club events, schedules, calendar feeds, and routes.**

Strava is a great place to organize group rides and runs, but the events are only visible to people logged in to Strava. clubmirror reads the group events of your club through the Strava API and generates a plain static website from templates: an events page, an `.ics` calendar to subscribe to, and downloadable GPX/TCX routes. 

It is meant to be run as a cron script that fetches and renders the data from [Tera](https://docs.rs/tera) templates as a static website and then exits. So there is no daemon to run. You can even run it and test it locally. You can manage any number of clubs in the configuration, each with its own templates, output, timezone, and time window.

> **Disclaimer.** clubmirror is an unofficial tool: it is not affiliated with, endorsed by or sponsored by Strava. You are responsible for making sure your use complies with the current [Strava API Agreement](https://www.strava.com/legal/api) and [Brand Guidelines](https://developers.strava.com/guidelines). Use at your own risk.

## Contents

- [Try it](#try-it)
- [Connect it to Strava](#connect-it-to-strava)
- [Configuration](#configuration)
- [Running it](#running-it)
- [Templates](#templates)
- [Calendar and routes](#calendar-and-routes)
- [Limitations](#limitations)
- [Roadmap](#roadmap)

## Try it

If you haven't already, install the Rust toolchain with [rustup](https://rustup.rs).

No Strava account nor network is needed in order to test. We included a fake sample of events [example/events.json](examples/events.json). The configuration file [examples/config.toml](examples/config.toml) is made to use this sample instead to generate the static website.

```sh
cargo install clubmirror
cd clubmirror
chmod 600 examples/config.toml
cargo run -- -c examples/config.toml -v
python3 -m http.server -d public 8000   # open http://localhost:8000
```

You get the events page and a calendar in `public/`. Edit `templates/default/` and run again to see your changes.

## Connect it to Strava

clubmirror reads the club events with the refresh token of an athlete who is **a member of the club**.

1. Create an API application at <https://www.strava.com/settings/api>. For *Authorization Callback Domain*, `localhost` is fine.
2. On that same page, Strava shows your application's **Client ID** and **Client Secret**, and, for your own account, **Your Access Token** and **Your Refresh Token** (scope `read`). Copy the client ID, the client secret, and the **refresh token** into the `[api]` section of the config file. You don't need the access token; clubmirror gets its own.
3. The club id is the number in the club's Strava URL, `https://www.strava.com/clubs/<id>`. You can find it with any of the internal links on the club page (Club Leaderboard, Recent Activity, ...).

If Strava rotates the refresh token, the new one is saved in `cache_path` and used from then on.
### Private routes and events

The `read` scope covers public events and routes. For private routes or private events you need a refresh token with the `read_all` scope, which the settings page doesn't give you:

1. Open this URL in your browser, logged in as the athlete (replace the client id):

   ```
   https://www.strava.com/oauth/authorize?client_id=YOUR_CLIENT_ID&response_type=code&redirect_uri=http://localhost&approval_prompt=force&scope=read,read_all
   ```

   Approve. The browser is redirected to a `http://localhost/?...&code=XXXX` page that won't load: copy the `code` value from the address bar.
2. Exchange the code for tokens, and use the `refresh_token` of the answer in the config file:

   ```sh
   curl -X POST https://www.strava.com/oauth/token \
     -d client_id=YOUR_CLIENT_ID -d client_secret=YOUR_CLIENT_SECRET \
     -d code=XXXX -d grant_type=authorization_code
   ```

Strava's developer program and its rules change from time to time: check its [developer documentation](https://developers.strava.com) for the current requirements, rate limits, and terms.

## Configuration

The tool is configured with one TOML file, `(/usr/local)/etc/clubmirror/config.toml` or whatever `--config` points to. Since it contains secrets, clubmirror refuses to start unless it is owned by the user running it and with permission `0o600`. 

Start from [config.toml.sample](config.toml.sample); every option is documented and can also be found in [src/config.rs](src/config.rs).

Generated files are written with the mode (default `0o644`) and the `user`/`group` of section `[generated]`. Obviously changing the owner requires the right privileges.

## Running it

Running it is as simple as just starting the command:

```sh
# -v for more logs, -q for none
clubmirror
```

It processes every club once, then exits with `0` when everything went fine, `1` if the configuration could not be loaded, and `2` if at least one club failed. Run it from cron or a periodic, for example, every hour:

```cron
0 * * * * clubmirror -q
```

Each run costs one API request per club + two per event with a route whose cached files have expired: mind Strava's rate limits. However, the default limits are well sufficient to handle a handful of clubs with your account.

## Templates

The default theme in [templates/default](templates/default) is a single page: a list of events, a detail view for each one (description, dates, organizer, level and terrain, route with map and downloads), and a calendar link. Dates are rendered in the club timezone and localized in the browser. Feel free to adapt it to your needs.

Files with a template extension (`html`, `xml`, `txt`, `md`, ...) are rendered with Tera; everything else is copied as is. This default list can be adjusted in the configuration file. The list of variables that can be used in the templates are documented in [docs/templates.md](docs/templates.md).

## Calendar and routes

**Calendar.** With `calendars_path` set, clubmirror writes `<club id>.ics`, with one entry per occurrence of each event: title, description, location, and the Strava page of the event as URL. The calendar can be presented with the variable `calendar_url` given to the templates. Its name is the club name, and events are assumed to last 2 hours.

**Routes.** With `routes_path` set, the GPX and TCX files of the routes attached to the events are downloaded and refreshed when older than `routes_ttl` seconds. They can be linked in the templates with `routes_url`.

## Limitations

- The calendar is built from all the events returned by the API, before the past/future window, and includes members-only events (`private`). The default theme hides members-only events; check `event.private` in your own templates.
- Events are assumed to last 2 hours in the calendar.
- `quirk_*` options (replaying and saving events, sorting) are temporary placeholders.

## Roadmap

- Replace the hardcoded quirks by scripted, per-club quirks with [Rhai](https://rhai.rs)).
- Proper debug options (record and replay the API answer) instead of `quirk_*`.
- Event filtering options, for example, leaving members-only events out of the calendar.
- Configurable event duration for the calendar.
- Processing clubs in parallel.
