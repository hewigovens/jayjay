#!/usr/bin/env python3
"""Update Sparkle appcast.xml — prepends a new entry, or replaces an existing one for the same version (idempotent).

SwiftUI release notes (HTML body inside <description><![CDATA[...]]>) are read
from releases/<version>.html. Missing or empty notes abort appcast generation.
"""
import sys
import os
import re
from datetime import datetime, timezone

if len(sys.argv) < 6:
    print("Usage: update-appcast.py <version> <build_number> <app_name> <zip_path> <appcast_path> [signature] [channel]")
    sys.exit(1)

version = sys.argv[1]
build_number = sys.argv[2]
app_name = sys.argv[3]
zip_path = sys.argv[4]
appcast_path = sys.argv[5]
signature = sys.argv[6] if len(sys.argv) > 6 else "PENDING"
channel = sys.argv[7] if len(sys.argv) > 7 else "stable"

# macOS deployment target — gate Sparkle so clients on older macOS aren't offered
# an update they can't launch. Keep in sync with shell/mac/project.yml.
MINIMUM_SYSTEM_VERSION = "26.0"


def insert_after_first(pattern: str, content: str, snippet: str, anchor_name: str) -> str:
    updated, count = re.subn(pattern, lambda match: match.group(0) + snippet, content, count=1)
    if count != 1:
        print(f"ERROR: could not locate {anchor_name} in {appcast_path}", file=sys.stderr)
        sys.exit(1)
    return updated


def normalize_item_indentation(content: str) -> str:
    return re.sub(r"^[ \t]*<item>[ \t]*$", "        <item>", content, flags=re.MULTILINE)

file_size = os.path.getsize(zip_path)
pub_date = datetime.now(timezone.utc).strftime("%a, %d %b %Y %H:%M:%S %z")

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
notes_path = os.path.join(repo_root, "releases", f"{version}.html")
if not os.path.isfile(notes_path):
    print(f"ERROR: SwiftUI release notes not found at {notes_path}", file=sys.stderr)
    sys.exit(1)

with open(notes_path, "r") as f:
    notes_html = f.read().strip()
if not notes_html:
    print(f"ERROR: SwiftUI release notes are empty at {notes_path}", file=sys.stderr)
    sys.exit(1)
if "]]>" in notes_html:
    print(f"ERROR: SwiftUI release notes contain an invalid CDATA terminator: {notes_path}", file=sys.stderr)
    sys.exit(1)

indented = "\n".join("                " + line for line in notes_html.splitlines())
description_block = f"""            <description><![CDATA[
{indented}
            ]]></description>
"""

channel_line = "            <sparkle:channel>beta</sparkle:channel>\n" if channel == "beta" else ""
short_version = version.split("-")[0]

new_item = f"""        <item>
            <title>Version {version}</title>
{channel_line}            <sparkle:version>{build_number}</sparkle:version>
            <sparkle:shortVersionString>{short_version}</sparkle:shortVersionString>
            <sparkle:minimumSystemVersion>{MINIMUM_SYSTEM_VERSION}</sparkle:minimumSystemVersion>
            <pubDate>{pub_date}</pubDate>
{description_block}            <enclosure url="https://github.com/hewigovens/jayjay/releases/download/v{version}/{app_name}-{version}.zip"
                       sparkle:edSignature="{signature}"
                       length="{file_size}"
                       type="application/octet-stream"/>
        </item>
"""

if os.path.exists(appcast_path) and os.path.getsize(appcast_path) > 0:
    with open(appcast_path, "r") as f:
        content = f.read()

    # Idempotent: drop any existing entry for this version before inserting.
    pattern = re.compile(
        r"^[ \t]*<item>\s*<title>Version " + re.escape(version) + r"</title>.*?</item>[ \t]*(?:\n|$)",
        re.MULTILINE | re.DOTALL,
    )
    content = pattern.sub("", content)

    # Sparkle only offers strictly greater sparkle:version values; a reused build number would strand earlier installs (beta testers most of all) with no upgrade path.
    existing_builds = [int(b) for b in re.findall(r"<sparkle:version>(\d+)</sparkle:version>", content)]
    if existing_builds and int(build_number) <= max(existing_builds):
        print(
            f"ERROR: build {build_number} must exceed every published sparkle:version (highest is {max(existing_builds)})",
            file=sys.stderr,
        )
        sys.exit(1)

    # Prepend the new entry right after the channel header without consuming the next item's indentation.
    if "</language>" in content:
        content = insert_after_first(r"</language>[ \t]*\n", content, new_item, "</language>")
    else:
        content = insert_after_first(r"<channel>[ \t]*\n", content, new_item, "<channel>")

    content = normalize_item_indentation(content)
    with open(appcast_path, "w") as f:
        f.write(content)
else:
    # Initial creation.
    content = f"""<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0" xmlns:sparkle="http://www.andymatuschak.org/xml-namespaces/sparkle" xmlns:dc="http://purl.org/dc/elements/1.1/">
    <channel>
        <title>JayJay</title>
        <link>https://raw.githubusercontent.com/hewigovens/jayjay/main/docs/appcast.xml</link>
        <description>JayJay updates</description>
        <language>en</language>
{new_item}    </channel>
</rss>
"""
    content = normalize_item_indentation(content)
    with open(appcast_path, "w") as f:
        f.write(content)

print(f"Updated {appcast_path}")
print(f"  Version: {version}")
print(f"  Size: {file_size}")
print(f"  Signature: {signature}")
