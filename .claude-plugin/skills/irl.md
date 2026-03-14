---
name: irl
description: Query Irish public sector open data — weather, parliament, statistics, transport, companies, property prices, water levels, and more. Use when the user asks about Irish government data, weather in Ireland, TDs, legislation, CSO statistics, bus times, company lookups, or house prices.
user_invocable: true
---

# Irish Public Sector Data Query

You have access to the `irl` CLI tool which provides structured access to Irish public sector open data APIs. When the user asks a question about Irish data, you should:

1. Map their query to the correct `irl` command
2. Execute it via Bash
3. Format the results as a readable markdown response

## Step 1: Check irl is available

Before your first command, verify the binary exists:

```bash
which irl 2>/dev/null || echo "NOT_FOUND"
```

If NOT_FOUND, tell the user:

> The `irl` CLI is not installed. Install it with:
> ```
> cd /path/to/irl-cli && cargo install --path .
> ```

Then stop — do not attempt to run irl commands.

## Step 2: Route the query to the correct command

Map the user's natural language query to an `irl` command using this routing table:

### Weather (Met Éireann)
| Query pattern | Command |
|---|---|
| weather/forecast for a location | `irl met forecast --location <location>` |
| weather forecast limited to N hours | `irl met forecast --location <location> --hours <N>` |
| weather warnings | `irl met warnings` |
| weather stations / what locations available | `irl met stations` |

Location names: dublin, cork, galway, limerick, waterford, belfast, killarney, sligo, athlone, letterkenny, wexford, dundalk, drogheda, kilkenny, ennis, tralee, carlow, tullamore, derry, newry.

### Parliament (Oireachtas)
| Query pattern | Command |
|---|---|
| TDs, senators, members of parliament | `irl oireachtas members` |
| members filtered by party | `irl oireachtas members --party "<party>"` |
| members filtered by constituency | `irl oireachtas members --constituency "<constituency>"` |
| legislation, bills, acts | `irl oireachtas legislation` |
| search legislation by topic | `irl oireachtas legislation --search "<topic>"` |
| legislation by status and year | `irl oireachtas legislation --status <status> --year <year>` |
| parliamentary debates | `irl oireachtas debates` |
| debates on a specific date | `irl oireachtas debates --date <YYYY-MM-DD>` |
| parliamentary questions | `irl oireachtas questions` |
| questions by a specific member | `irl oireachtas questions --member "<name>"` |
| votes, divisions | `irl oireachtas divisions` |

### Statistics (CSO PxStat)
| Query pattern | Command |
|---|---|
| search for statistical tables | `irl cso tables --search "<topic>"` |
| list all CSO tables | `irl cso tables` |
| info about a specific table | `irl cso info <TABLE_CODE>` |
| query table data | `irl cso query <TABLE_CODE>` |
| query with dimension filter | `irl cso query <TABLE_CODE> --dimension "<Key>=<Value>"` |
| query last N time periods | `irl cso query <TABLE_CODE> --last <N>` |

### Transport (NTA)
| Query pattern | Command |
|---|---|
| bus/train departures from a stop | `irl transport departures --stop <STOP_ID>` |
| departures filtered by route | `irl transport departures --stop <STOP_ID> --route <ROUTE>` |
| vehicle positions for a route | `irl transport vehicles --route <ROUTE>` |
| search for stops | `irl transport stops --search "<name>"` |
| list routes | `irl transport routes` |

**Note:** Transport requires an API key. If the user gets an "API key missing" error, tell them:
```
Register at https://developer.nationaltransport.ie/ then run:
irl config set transport.api_key YOUR_KEY
```

### Companies (CRO)
| Query pattern | Command |
|---|---|
| search companies by name | `irl cro search "<company name>"` |
| search active companies | `irl cro search --status active "<name>"` |
| company details by number | `irl cro company <NUMBER>` |
| company filings | `irl cro filings <NUMBER>` |

### Property Prices (PSRA)
| Query pattern | Command |
|---|---|
| search property sales | `irl property search --county <COUNTY>` |
| search by county and year | `irl property search --county <COUNTY> --year <YEAR>` |
| search by price range | `irl property search --min <MIN> --max <MAX>` |
| search by address | `irl property search --address "<address>"` |
| property price statistics | `irl property stats --county <COUNTY>` |
| compare counties | `irl property stats --compare <County1>,<County2>,<County3>` |
| download/update property data | `irl property update` |

**Note:** Property data must be downloaded first with `irl property update`.

### Environment (EPA)
| Query pattern | Command |
|---|---|
| air quality | `irl epa air-quality` |
| air quality at a station | `irl epa air-quality --station "<station>"` |
| water quality by catchment | `irl epa water-quality --catchment "<name>"` |

### Water Levels (OPW)
| Query pattern | Command |
|---|---|
| water monitoring stations | `irl water stations` |
| stations in a county | `irl water stations --county <COUNTY>` |
| search stations by name | `irl water search "<name>"` |
| water level at a station | `irl water level <STATION_ID>` |

### Valuations (Tailte Éireann)
| Query pattern | Command |
|---|---|
| search valuations by address | `irl tailte search --address "<address>"` |
| property valuation details | `irl tailte property <NUMBER>` |

### Geographic Data (GeoHive/OSi)
| Query pattern | Command |
|---|---|
| county boundaries | `irl geo boundaries --type county` |
| electoral districts | `irl geo boundaries --type electoral-district` |
| available datasets | `irl geo datasets` |

### General
| Query pattern | Command |
|---|---|
| what data sources are available | `irl help` |
| show irl configuration | `irl config show` |
| set up config | `irl config init` |

If the query doesn't clearly match any module, show the user the available modules and ask them to clarify.

## Step 3: Execute the command

Run the command with `--format json -q` flags appended for machine-readable output:

```bash
irl <subcommand> <args> --format json -q 2>&1
```

Always append `--format json -q` unless the user explicitly asks for raw/table output.

If the command fails or returns an error, capture stderr and present the error clearly to the user with a suggested fix.

## Step 4: Format the results

After receiving the JSON output:

1. Parse the JSON (it will be an array of objects or a single object)
2. Write a **1-2 sentence natural language summary** of what the data shows
3. Render the data as a **markdown table** with clear column headers
4. If results have more than 20 rows, show the first 20 and note: "*Showing 20 of N results. Use `--limit` to see more.*"
5. For empty results, say: "No results found for this query."
6. Preserve Irish characters (fadas: á, é, í, ó, ú) correctly in all output

### Example output format:

> **Weather in Dublin** — Current observations from Dublin Airport show 9°C with fair conditions and westerly winds.
>
> | Time | Temp | Weather | Wind | Humidity |
> |------|------|---------|------|----------|
> | 14:00 | 9°C | Fair | 15 km/h W | 56% |
> | 13:00 | 9°C | Fair | 20 km/h W | 67% |
> | ... | ... | ... | ... | ... |

## Handling Errors

- **"API key missing"** → Tell the user how to register and configure: `irl config set <service>.api_key <KEY>`
- **"API returned error"** → Show the error message and suggest trying again or checking the service status
- **"No results found"** → Suggest broadening the search criteria
- **Network errors** → Suggest checking internet connection or trying with `--no-cache`
- **Non-JSON output** → The command may have printed an error message. Show it directly.

## Variables

USER_QUERY: $ARGUMENTS
