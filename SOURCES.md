# Data Sources

This document lists all data sources used by the `irl` CLI, their endpoints, licences, and registration requirements.

## Houses of the Oireachtas

- **API:** https://api.oireachtas.ie/v1/
- **Documentation:** https://api.oireachtas.ie/
- **Licence:** Open Data (Oireachtas Open Data Licence)
- **Registration:** None required
- **Endpoints used:**
  - `/v1/members` — TDs and Senators
  - `/v1/legislation` — Bills and Acts
  - `/v1/debates` — Parliamentary debates
  - `/v1/questions` — Parliamentary questions
  - `/v1/divisions` — Voting records
  - `/v1/parties` — Political parties

## Met Éireann

- **API:** https://prodapi.metweb.ie/
- **Licence:** Met Éireann Open Data Licence (attribution required)
- **Registration:** None required
- **Attribution:** Data provided by Met Éireann — https://www.met.ie/
- **Endpoints used:**
  - `/observations/{location}/today` — Hourly observations
  - Weather warnings via met.ie Open Data JSON feeds

## Central Statistics Office (PxStat)

- **API:** https://ws.cso.ie/public/api.restful/
- **Documentation:** https://data.cso.ie/
- **Licence:** Creative Commons Attribution 4.0 (CC BY 4.0)
- **Registration:** None required
- **Endpoints used:**
  - `PxStat.Data.Cube_API.ReadCollection` — Table of contents
  - `PxStat.Data.Cube_API.ReadDataset/{code}/JSON-stat/2.0/en` — Table data (JSON-stat 2.0)

## Transport for Ireland (NTA)

- **API:** https://api.nationaltransport.ie/
- **Documentation:** https://developer.nationaltransport.ie/
- **Licence:** NTA Open Data Licence
- **Registration:** **Required** — register at https://developer.nationaltransport.ie/
- **API Key:** Set via `irl config set transport.api_key <KEY>`
- **Endpoints used:**
  - `/gtfsr/v2/Vehicles?format=json` — Vehicle positions
  - `/gtfsr/v2/TripUpdates?format=json` — Trip updates

## Companies Registration Office

- **API:** https://core.cro.ie/ (under investigation)
- **Open Data Portal:** https://opendata.cro.ie/
- **Licence:** Open Data
- **Registration:** May be required
- **Status:** API endpoints being verified; Cloudflare protection detected

## Property Price Register (PSRA)

- **Website:** https://www.propertypriceregister.ie/
- **Licence:** Open Data
- **Registration:** None required
- **Data Format:** CSV files (Windows-1252 encoding)
- **Notes:** Data requires form-based download from the PSRA website. CSV files are imported into a local SQLite database for querying.

## Environmental Protection Agency

- **Website:** https://airquality.ie/ and https://data.epa.ie/
- **Licence:** Open Data
- **Registration:** None required
- **Status:** Public JSON API endpoints not currently accessible. Web interface available at airquality.ie.

## OPW Water Levels

- **API:** https://waterlevel.ie/
- **Licence:** Open Data
- **Registration:** None required
- **Endpoints used:**
  - `/geojson/` — Station locations (GeoJSON FeatureCollection)
  - Station data endpoints under investigation

## Tailte Éireann (Valuation Office)

- **Website:** https://www.tailte.ie/
- **Licence:** Open Data
- **Registration:** None required
- **Status:** REST API endpoint (opendata.tailte.ie) currently returning 404. Bulk data may be available via data.gov.ie.

## GeoHive / Ordnance Survey Ireland

- **API:** ArcGIS REST Services
- **Documentation:** https://data-osi.opendata.arcgis.com/
- **Licence:** Creative Commons Attribution 4.0 (CC BY 4.0)
- **Registration:** None required
- **Notes:** ArcGIS FeatureServer URLs need to be configured for specific datasets. Standard ArcGIS REST query pattern supported.
