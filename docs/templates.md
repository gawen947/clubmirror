# Writing templates

clubmirror renders a **template directory** into an **output directory**, once per club
(`template_dir` and `output_dir` of each `[[club]]`). Templates use [Tera 2](https://docs.rs/tera)
(Jinja-like). Start from [`templates/default`](../templates/default) and edit it.

## What gets rendered

| In the template directory | In the output directory |
|---|---|
| files whose extension is in `template_extensions` (default: `html htm xml txt md json rss atom ics webmanifest`) | rendered with Tera, same relative path |
| any other file (CSS, JS, images, fonts, ...) | copied verbatim |
| files or directories starting with `_` | never written; usable with `{% extends %}` / `{% include %}` |
| dotfiles | ignored |

Template names are paths relative to the template directory: `{% include "_partials/header.html" %}`.
Tera auto-escapes `.html`, `.htm` and `.xml` files.
Other extensions are not escaped: use `| escape` yourself.

## Variables

| Name | Type | Notes |
|---|---|---|
| `club_id` | number | Strava club id |
| `club_name` | string | `name` of the `[[club]]`, else the name reported by Strava, else `Club <id>` |
| `timezone` | string | IANA timezone of the club, e.g. `Europe/Brussels` |
| `events` | list | the events after the past/future window and sorting, see below |
| `total_events` | number | `events | length` |
| `generated_at` | string | `2026-11-07 12:00:00 UTC` |
| `routes_url` | string | only if `routes_url` is set in `[generated]` |
| `calendars_url` | string | only if `calendars_url` is set |
| `calendar_url` | string | only if `calendars_url` is set: `<calendars_url>/<club id>.ics` |

## An event

| Field | Notes |
|---|---|
| `id`, `title`, `description` | `description` has newlines and may be empty |
| `address` | may be empty |
| `upcoming_occurrences` | list of RFC 3339 UTC timestamps, may be **empty** |
| `route_id` | number or `null` (can exceed 2^53: never parse it with JavaScript) |
| `route` | `null` most of the time; else `name`, `map_urls.{url,retina_url,light_url,dark_url}`, `map.summary_polyline` |
| `skill_levels` | `null`, or a bit field: 1 casual, 2 tempo, 4 race (see Strava's docs; combinations such as 3 are assumed to be sums) |
| `terrain` | `null`, 0 mostly flat, 1 rolling hills, 2 killer climbs |
| `activity_type` | `Ride`, `GravelRide`, `Run`, ... |
| `women_only`, `private`, `joined` | booleans. `private` = members-only event on Strava |
| `start_latlng` | `null` or `{latitude, longitude}` |
| `zone` | timezone of the event |
| `created_at` | timestamp or `null` |
| `club` | `{id, name}` |
| `organizing_athlete` | `id`, `firstname`, `lastname`; the others can be `null`: `username`, `bio`, `city`, `state`, `country`, `sex`, `premium`, `summit`, `profile`, `profile_medium`, ... Strava can return a relative placeholder (`avatar/athlete/medium.png`) instead of a URL when there is no picture |

## Filters, tests and Tera 2 notes

Tera 2 has no built-in `date` or `filter` filter: clubmirror provides them.

- `{{ ts | date(format="%Y-%m-%d %H:%M", timezone="Europe/Brussels") }}`: formats an RFC 3339 timestamp
  (chrono format specifiers, e.g. `%A %d %B`); without `timezone` it is UTC. Day and month names are in English.
- `{{ list | filter(attribute="private", value=false) }}`: keeps the items whose attribute equals the value.

Tera 2 is strict about missing values: reading a field of `null` is an error, and tests take named
arguments.

```jinja
{# event.route is null most of the time: test each step before going deeper #}
{% if event.route and event.route.map_urls is defined and event.route.map_urls.retina_url is defined %} ... {% endif %}

{# named arguments #}
{% if url is starting_with(pat="http") %} ... {% endif %}
```

## Snippets

```jinja
{# date in the club timezone, English names #}
{{ event.upcoming_occurrences[0] | date(format="%A %d %B %Y, %H:%M", timezone=timezone) }}

{# guard against events without dates #}
{% if event.upcoming_occurrences | length > 0 %} ... {% endif %}

{# terrain 0 is falsy: test the type, not the value #}
{% if event.terrain is number %} ... {% endif %}

{# skill_levels bit field #}
{% if event.skill_levels in [2, 3, 6, 7] %}Tempo{% endif %}

{# hide members-only events #}
{% set shown = events | filter(attribute="private", value=false) %}

{# description lines as paragraphs #}
{% for line in event.description | split(pat="\n") %}{% if line | trim %}<p>{{ line | trim }}</p>{% endif %}{% endfor %}
```

## Experimenting locally

No Strava account is needed to work on a template:

```sh
chmod 600 examples/config.toml
cargo run -- -c examples/config.toml -v        # renders into ./public
python3 -m http.server -d public 8000
```

`examples/events.json` holds fake events covering the awkward cases: empty description and address,
no dates, routes with and without a map, null athlete fields, very long titles, HTML in titles,
members-only and women-only events, a non-cycling activity. The dates are fixed: edit them if they
are in the past for you, or set `past_days` in the club to see the window at work.

To work on your own club's data, run clubmirror once for real with `quirk_save_json = "some/dir"` in
`[generated]` (writes `<club id>.json`), then replay it with `quirk_local_json` in `[api]`: no more API calls.
In that mode clubmirror does not need a token and does not download routes.
