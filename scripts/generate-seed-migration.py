#!/usr/bin/env python3
"""
Generate a SQL migration that seeds judges and attorneys from FJC data files.

Usage:
    python3 scripts/generate-seed-migration.py \
        --judges-csv ~/Downloads/judges.csv \
        --magistrates-json ~/Downloads/brmag-judges.json \
        --output migrations/20260301000072_seed_judges_attorneys.sql

Reads real federal judge data from:
  - FJC Biographical Directory CSV (district/circuit judges)
  - CourtListener magistrate/bankruptcy JSON

Produces a SQL migration with INSERT statements for:
  - Court records for each federal district
  - Active district court judges
  - Active magistrate judges
  - Generated sample attorneys
"""

import argparse
import csv
import json
import hashlib
import sys

# ---------------------------------------------------------------------------
# Court name -> court_id mapping
# ---------------------------------------------------------------------------

STATE_ABBREVS = {
    "Alabama": "al", "Alaska": "ak", "Arizona": "az", "Arkansas": "ar",
    "California": "ca", "Colorado": "co", "Connecticut": "ct", "Delaware": "de",
    "Florida": "fl", "Georgia": "ga", "Hawaii": "hi", "Idaho": "id",
    "Illinois": "il", "Indiana": "in", "Iowa": "ia", "Kansas": "ks",
    "Kentucky": "ky", "Louisiana": "la", "Maine": "me", "Maryland": "md",
    "Massachusetts": "ma", "Michigan": "mi", "Minnesota": "mn",
    "Mississippi": "ms", "Missouri": "mo", "Montana": "mt", "Nebraska": "ne",
    "Nevada": "nv", "New Hampshire": "nh", "New Jersey": "nj",
    "New Mexico": "nm", "New York": "ny", "North Carolina": "nc",
    "North Dakota": "nd", "Ohio": "oh", "Oklahoma": "ok", "Oregon": "or",
    "Pennsylvania": "pa", "Rhode Island": "ri", "South Carolina": "sc",
    "South Dakota": "sd", "Tennessee": "tn", "Texas": "tx", "Utah": "ut",
    "Vermont": "vt", "Virginia": "va", "Washington": "wa",
    "West Virginia": "wv", "Wisconsin": "wi", "Wyoming": "wy",
    "Columbia": "dc", "Puerto Rico": "pr", "Guam": "gu",
    "Virgin Islands": "vi", "Northern Mariana Islands": "nmi",
}

DIRECTION_ABBREVS = {
    "Northern": "n", "Southern": "s", "Eastern": "e", "Western": "w",
    "Central": "c", "Middle": "m",
}


def court_name_to_id(name):
    """Convert FJC court name to a short district code."""
    if not name:
        return None

    cleaned = name.replace("U.S. District Court for the ", "")
    cleaned = cleaned.replace("U.S. District Court - ", "")
    cleaned = cleaned.replace("U.S. Bankruptcy Court - ", "")
    cleaned = cleaned.replace("District of ", "")

    direction = ""
    for d_name, d_abbrev in DIRECTION_ABBREVS.items():
        if d_name in cleaned:
            direction = d_abbrev
            cleaned = cleaned.replace(f"{d_name} District of ", "")
            cleaned = cleaned.replace(f"{d_name} ", "")
            break

    state_code = ""
    for s_name, s_abbrev in STATE_ABBREVS.items():
        if s_name in cleaned:
            state_code = s_abbrev
            break

    if state_code:
        return f"{state_code}{direction}d"
    return None


def court_id_to_name(court_id, district_text):
    """Build a human-readable court name from the district text."""
    return f"U.S. District Court for the {district_text}"


# ---------------------------------------------------------------------------
# Title and status mapping
# ---------------------------------------------------------------------------

def map_title(raw_title):
    """Map FJC appointment title to valid judge title."""
    if not raw_title:
        return "Judge"
    t = raw_title.strip()
    if "Chief" in t:
        return "Chief Judge"
    if "Magistrate" in t:
        return "Magistrate Judge"
    if "Senior" in t:
        return "Senior Judge"
    return "Judge"


def map_status(row):
    """Determine judge status from CSV row."""
    senior_date = (row.get("Senior Status Date (1)") or "").strip()
    termination = (row.get("Termination (1)") or "").strip()
    if termination == "Retirement":
        return "Retired"
    if senior_date:
        return "Senior"
    return "Active"


def parse_date(date_str):
    """Parse date string to ISO 8601 or None."""
    if not date_str or not date_str.strip():
        return None
    d = date_str.strip()
    from datetime import datetime
    for fmt in ("%Y-%m-%d", "%m/%d/%Y", "%Y"):
        try:
            return datetime.strptime(d, fmt).strftime("%Y-%m-%dT00:00:00Z")
        except ValueError:
            continue
    return None


def sql_str(value):
    """Escape a string for SQL."""
    if value is None:
        return "NULL"
    s = str(value).replace("'", "''")
    return f"'{s}'"


def sql_date(value):
    """Format a date for SQL."""
    if value is None:
        return "NULL"
    return f"'{value}'"


def deterministic_uuid(seed_string):
    """Generate a deterministic UUID from a seed string."""
    h = hashlib.sha256(seed_string.encode()).hexdigest()
    return f"{h[:8]}-{h[8:12]}-4{h[13:16]}-8{h[17:20]}-{h[20:32]}"


# ---------------------------------------------------------------------------
# Parse CSV judges
# ---------------------------------------------------------------------------

def parse_csv_judges(csv_path):
    """Parse FJC biographical directory CSV."""
    judges = []
    with open(csv_path, "r", encoding="utf-8-sig") as f:
        reader = csv.DictReader(f)
        for row in reader:
            death_year = (row.get("Death Year") or "").strip()
            if death_year:
                continue

            court_type = (row.get("Court Type (1)") or "").strip()
            if court_type != "U.S. District Court":
                continue

            court_name = (row.get("Court Name (1)") or "").strip()
            court_id = court_name_to_id(court_name)
            if not court_id:
                continue

            first = (row.get("First Name") or "").strip()
            middle = (row.get("Middle Name") or "").strip()
            last = (row.get("Last Name") or "").strip()
            suffix = (row.get("Suffix") or "").strip()

            name_parts = [first]
            if middle and middle != " ":
                name_parts.append(middle)
            name_parts.append(last)
            if suffix and suffix != " ":
                name_parts.append(suffix)
            name = " ".join(name_parts)

            title = map_title(row.get("Appointment Title (1)"))
            status = map_status(row)
            commission = parse_date(row.get("Commission Date (1)"))
            senior_date = parse_date(row.get("Senior Status Date (1)"))

            # Check if currently serving as Chief Judge
            chief_begin = (row.get("Service as Chief Judge, Begin (1)") or "").strip()
            chief_end = (row.get("Service as Chief Judge, End (1)") or "").strip()
            if chief_begin and not chief_end:
                title = "Chief Judge"

            district_text = court_name.replace("U.S. District Court for the ", "")

            judges.append({
                "court_id": court_id,
                "name": name,
                "title": title,
                "district": district_text,
                "appointed_date": commission,
                "status": status,
                "senior_status_date": senior_date,
                "seed_key": f"csv-{row.get('jid', '')}",
                "court_full_name": court_name,
            })

    return judges


# ---------------------------------------------------------------------------
# Parse JSON magistrate judges
# ---------------------------------------------------------------------------

def parse_json_magistrates(json_path):
    """Parse CourtListener magistrate/bankruptcy JSON."""
    judges = []
    with open(json_path, "r") as f:
        data = json.load(f)

    for entry in data:
        death = (entry.get("death_date") or "").strip()
        if death:
            continue

        positions = entry.get("positions", [])
        current = None
        for pos in positions:
            end = (pos.get("date_end") or "").strip()
            title_raw = (pos.get("title") or "").strip()
            if not end and "Magistrate" in title_raw:
                current = pos
                break

        if not current:
            for pos in positions:
                end = (pos.get("date_end") or "").strip()
                title_raw = (pos.get("title") or "").strip()
                if not end and "Bankruptcy" not in title_raw:
                    current = pos
                    break

        if not current:
            continue

        institution = (current.get("institution") or "").strip()
        court_id = court_name_to_id(institution)
        if not court_id:
            continue

        first = (entry.get("name_first") or "").strip()
        middle = (entry.get("name_middle") or "").strip()
        last = (entry.get("name_last") or "").strip()
        suffix = (entry.get("name_suffix") or "").strip()

        name_parts = [first]
        if middle:
            name_parts.append(middle)
        name_parts.append(last)
        if suffix:
            name_parts.append(suffix)
        name = " ".join(name_parts)

        title = map_title(current.get("title"))
        start_date = parse_date(current.get("date_start"))

        district_text = institution.replace("U.S. District Court - ", "")
        district_text = district_text.replace("U.S. District Court for the ", "")

        court_full = f"U.S. District Court for the {district_text}"

        judges.append({
            "court_id": court_id,
            "name": name,
            "title": title,
            "district": district_text,
            "appointed_date": start_date,
            "status": "Active",
            "senior_status_date": None,
            "seed_key": f"mag-{entry.get('judge_id', '')}",
            "court_full_name": court_full,
        })

    return judges


# ---------------------------------------------------------------------------
# Generate attorneys
# ---------------------------------------------------------------------------

ATTORNEY_DATA = [
    ("Sarah", "Mitchell", "Johnson", "Mitchell & Associates"),
    ("Robert", "James", "Chen", "Chen Law Group"),
    ("Maria", "Elena", "Garcia", "Garcia & Partners LLP"),
    ("David", "Allen", "Park", "Park Legal Services"),
    ("Jennifer", "Lynn", "O'Brien", "O'Brien & Associates"),
    ("Michael", "Thomas", "Williams", None),
    ("Lisa", "Marie", "Rodriguez", "Rodriguez Criminal Defense"),
    ("James", "Patrick", "Murphy", "Murphy Law Firm"),
    ("Amanda", "Rose", "Thompson", "Thompson & Associates"),
    ("Christopher", "Lee", "Kim", None),
    ("Rachel", "Ann", "Patel", "Patel Law Group"),
    ("William", "Edward", "Davis", "Davis & Whitmore LLP"),
    ("Michelle", "Lee", "Nguyen", "Nguyen Legal Defense"),
    ("Daniel", "Mark", "Anderson", "Anderson & Partners"),
    ("Stephanie", "Grace", "Brown", None),
    ("Kevin", "John", "Wilson", "Wilson Criminal Law"),
    ("Laura", "Beth", "Taylor", "Taylor & Associates"),
    ("Marcus", "James", "Jackson", "Jackson Legal Group"),
    ("Patricia", "Ann", "White", "White & Associates LLP"),
    ("Brian", "Michael", "Harris", None),
    ("Christina", "Marie", "Martin", "Martin Law Offices"),
    ("Andrew", "Thomas", "Lewis", "Lewis & Walker"),
    ("Nicole", "Elizabeth", "Clark", "Clark Criminal Defense"),
    ("Steven", "Robert", "Robinson", None),
    ("Karen", "Sue", "Martinez", "Martinez & Associates"),
    ("Gregory", "Paul", "Hall", "Hall Law Group"),
    ("Angela", "Dawn", "Young", "Young & Associates"),
    ("Jason", "Lee", "Allen", "Allen Legal Services"),
    ("Catherine", "Anne", "King", "King & Associates"),
    ("Thomas", "William", "Scott", None),
    ("Rebecca", "Jo", "Adams", "Adams Law Firm"),
    ("Eric", "Charles", "Baker", "Baker & Nelson LLP"),
    ("Diana", "Lynn", "Gonzalez", "Gonzalez Defense Group"),
    ("Richard", "Joseph", "Nelson", None),
    ("Susan", "Kay", "Carter", "Carter Legal Services"),
    ("Jeffrey", "Dean", "Mitchell", "Mitchell & Stern"),
    ("Heather", "Marie", "Perez", None),
    ("Timothy", "James", "Roberts", "Roberts Law Group"),
    ("Sandra", "Lee", "Turner", "Turner & Associates"),
    ("Ryan", "Patrick", "Phillips", "Phillips Criminal Law"),
    ("Emily", "Anne", "Campbell", None),
    ("Douglas", "Wayne", "Parker", "Parker & Associates"),
    ("Amy", "Nicole", "Evans", "Evans Law Firm"),
    ("Mark", "Allen", "Edwards", "Edwards & Associates"),
    ("Donna", "Sue", "Collins", None),
    ("Scott", "David", "Stewart", "Stewart Legal Group"),
    ("Pamela", "Jean", "Sanchez", "Sanchez Defense"),
    ("Adam", "Michael", "Morris", None),
    ("Christine", "Marie", "Rogers", "Rogers & Associates"),
    ("Joseph", "Francis", "Reed", "Reed Law Offices"),
]

ATTORNEY_STATUSES = [
    "Active", "Active", "Active", "Active", "Active",
    "Active", "Active", "Active", "Active", "Active",
    "Active", "Active", "Active", "Active", "Active",
    "Active", "Active", "Active", "Active", "Active",
    "Active", "Active", "Active", "Active", "Active",
    "Active", "Active", "Inactive", "Active", "Active",
    "Active", "Active", "Active", "Active", "Active",
    "Active", "Active", "Active", "Active", "Active",
    "Active", "Suspended", "Active", "Active", "Active",
    "Active", "Active", "Active", "Active", "Retired",
]

ATTORNEY_CITIES = [
    ("New York", "NY", "10001"), ("Los Angeles", "CA", "90012"),
    ("Chicago", "IL", "60602"), ("Houston", "TX", "77002"),
    ("Miami", "FL", "33132"), ("Boston", "MA", "02108"),
    ("Philadelphia", "PA", "19107"), ("Washington", "DC", "20001"),
    ("Newark", "NJ", "07102"), ("Atlanta", "GA", "30303"),
]


def generate_attorneys(court_ids):
    """Generate sample attorneys distributed across known courts."""
    # Use courts that actually exist from the judge data
    courts_list = sorted(court_ids)[:20] if len(court_ids) > 20 else sorted(court_ids)
    if not courts_list:
        courts_list = ["nysd", "cacd", "ilnd"]

    attorneys = []
    for i, (first, middle, last, firm) in enumerate(ATTORNEY_DATA):
        city_info = ATTORNEY_CITIES[i % len(ATTORNEY_CITIES)]
        court_id = courts_list[i % len(courts_list)]
        state = city_info[1]
        bar_num = f"{state}{10000 + i}"
        status = ATTORNEY_STATUSES[i]

        fax = f"(555) {200 + i:03d}-{2000 + i * 7:04d}" if firm else None
        languages = "ARRAY['English'"
        if i % 4 == 0:
            languages += ",'Spanish'"
        if i % 7 == 0:
            languages += ",'Mandarin'"
        languages += "]::text[]"

        attorneys.append({
            "court_id": court_id,
            "bar_number": bar_num,
            "first_name": first,
            "middle_name": middle,
            "last_name": last,
            "firm_name": firm,
            "email": f"{first.lower()}.{last.lower()}@example.com",
            "phone": f"(555) {100 + i:03d}-{1000 + i * 7:04d}",
            "fax": fax,
            "status": status,
            "street1": f"{100 + i * 10} Federal Plaza",
            "city": city_info[0],
            "state": state,
            "zip_code": city_info[2],
            "country": "US",
            "cja_panel_member": i % 5 == 0,
            "cases_handled": (i * 7 + 3) % 200,
            "languages_sql": languages,
            "seed_key": f"atty-{i}",
        })

    return attorneys


# ---------------------------------------------------------------------------
# SQL generation
# ---------------------------------------------------------------------------

def generate_sql(judges, attorneys, courts_map):
    """Generate the complete SQL migration."""
    lines = []
    lines.append("-- Seed data: Federal judges (from FJC data) and sample attorneys")
    lines.append("-- Generated by: scripts/generate-seed-migration.py")
    lines.append("-- Sources: FJC Biographical Directory CSV, CourtListener magistrate JSON")
    lines.append("")
    lines.append("-- Guard: skip if judge data already seeded")
    lines.append("DO $$ BEGIN")
    lines.append("    IF (SELECT COUNT(*) FROM judges) > 10 THEN")
    lines.append("        RAISE NOTICE 'Judges table already seeded (>10 rows), skipping.';")
    lines.append("        RETURN;")
    lines.append("    END IF;")
    lines.append("")

    # Insert courts first
    lines.append(f"    -- Create {len(courts_map)} federal court districts")
    for court_id in sorted(courts_map.keys()):
        court_name = courts_map[court_id]
        lines.append(f"    INSERT INTO courts (id, name, court_type) VALUES ({sql_str(court_id)}, {sql_str(court_name)}, 'district') ON CONFLICT (id) DO NOTHING;")
    lines.append("")

    # Insert judges
    lines.append(f"    -- Insert {len(judges)} judges")
    for j in judges:
        uuid = deterministic_uuid(j["seed_key"])
        h = int(hashlib.md5(j["seed_key"].encode()).hexdigest()[:4], 16)
        max_cl = 100 + (h % 300)
        cur_cl = h % max_cl if j["status"] == "Active" else 0

        lines.append(
            f"    INSERT INTO judges (id, court_id, name, title, district, appointed_date, status, senior_status_date, courtroom, current_caseload, max_caseload, specializations)"
            f" VALUES ({sql_str(uuid)}, {sql_str(j['court_id'])}, {sql_str(j['name'])}, {sql_str(j['title'])}, {sql_str(j['district'])}, {sql_date(j['appointed_date'])}, {sql_str(j['status'])}, {sql_date(j['senior_status_date'])}, NULL, {cur_cl}, {max_cl}, ARRAY[]::text[]);"
        )
    lines.append("")

    # Insert attorneys
    lines.append(f"    -- Insert {len(attorneys)} attorneys")
    for a in attorneys:
        uuid = deterministic_uuid(a["seed_key"])
        firm_val = sql_str(a["firm_name"]) if a["firm_name"] else "NULL"
        fax_val = sql_str(a["fax"]) if a["fax"] else "NULL"

        lines.append(
            f"    INSERT INTO attorneys (id, court_id, bar_number, first_name, middle_name, last_name, firm_name, email, phone, fax, address_street1, address_city, address_state, address_zip, address_country, status, cja_panel_member, cases_handled, languages_spoken)"
            f" VALUES ({sql_str(uuid)}, {sql_str(a['court_id'])}, {sql_str(a['bar_number'])}, {sql_str(a['first_name'])}, {sql_str(a['middle_name'])}, {sql_str(a['last_name'])}, {firm_val}, {sql_str(a['email'])}, {sql_str(a['phone'])}, {fax_val}, {sql_str(a['street1'])}, {sql_str(a['city'])}, {sql_str(a['state'])}, {sql_str(a['zip_code'])}, {sql_str(a['country'])}, {sql_str(a['status'])}, {'TRUE' if a['cja_panel_member'] else 'FALSE'}, {a['cases_handled']}, {a['languages_sql']});"
        )
    lines.append("")

    lines.append("END $$;")
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="Generate seed data migration from FJC judge files")
    parser.add_argument("--judges-csv", required=True, help="Path to FJC judges CSV")
    parser.add_argument("--magistrates-json", required=True, help="Path to magistrate judges JSON")
    parser.add_argument("--output", required=True, help="Output SQL migration file path")
    args = parser.parse_args()

    print(f"Parsing CSV judges from {args.judges_csv}...")
    csv_judges = parse_csv_judges(args.judges_csv)
    print(f"  Found {len(csv_judges)} active district court judges")

    print(f"Parsing magistrate judges from {args.magistrates_json}...")
    mag_judges = parse_json_magistrates(args.magistrates_json)
    print(f"  Found {len(mag_judges)} active magistrate judges")

    all_judges = csv_judges + mag_judges
    print(f"Total judges: {len(all_judges)}")

    # Deduplicate by name + court_id
    seen = set()
    unique_judges = []
    for j in all_judges:
        key = (j["name"].lower(), j["court_id"])
        if key not in seen:
            seen.add(key)
            unique_judges.append(j)
    print(f"After dedup: {len(unique_judges)} unique judges")

    # Build courts map from judge data
    courts_map = {}
    for j in unique_judges:
        if j["court_id"] not in courts_map:
            courts_map[j["court_id"]] = j.get("court_full_name", f"U.S. District Court ({j['court_id']})")

    print(f"Courts: {len(courts_map)} districts")

    print("Generating attorneys...")
    attorneys = generate_attorneys(set(courts_map.keys()))
    print(f"  Generated {len(attorneys)} attorneys")

    sql = generate_sql(unique_judges, attorneys, courts_map)

    with open(args.output, "w") as f:
        f.write(sql)

    size_kb = len(sql.encode()) / 1024
    print(f"\nMigration written to {args.output}")
    print(f"  {len(unique_judges)} judges + {len(attorneys)} attorneys across {len(courts_map)} courts")
    print(f"  File size: {size_kb:.0f} KB")


if __name__ == "__main__":
    main()
