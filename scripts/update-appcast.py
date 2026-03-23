#!/usr/bin/env python3
"""Generate Sparkle appcast.xml from release artifacts."""
import sys
import os

if len(sys.argv) < 6:
    print("Usage: update-appcast.py <version> <build_number> <app_name> <zip_path> <appcast_path> [signature]")
    sys.exit(1)

version = sys.argv[1]
build_number = sys.argv[2]
app_name = sys.argv[3]
zip_path = sys.argv[4]
appcast_path = sys.argv[5]
signature = sys.argv[6] if len(sys.argv) > 6 else "PENDING"

file_size = os.path.getsize(zip_path)

from datetime import datetime, timezone
pub_date = datetime.now(timezone.utc).strftime("%a, %d %b %Y %H:%M:%S %z")

appcast = f"""<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0" xmlns:sparkle="http://www.andymatuschak.org/xml-namespaces/sparkle" xmlns:dc="http://purl.org/dc/elements/1.1/">
    <channel>
        <title>JayJay</title>
        <link>https://raw.githubusercontent.com/hewigovens/jayjay/main/docs/appcast.xml</link>
        <description>JayJay updates</description>
        <language>en</language>
        <item>
            <title>Version {version}</title>
            <sparkle:version>{build_number}</sparkle:version>
            <sparkle:shortVersionString>{version}</sparkle:shortVersionString>
            <pubDate>{pub_date}</pubDate>
            <enclosure url="https://github.com/hewigovens/jayjay/releases/download/v{version}/{app_name}-{version}.zip"
                       sparkle:edSignature="{signature}"
                       length="{file_size}"
                       type="application/octet-stream"/>
        </item>
    </channel>
</rss>
"""

with open(appcast_path, "w") as f:
    f.write(appcast)

print(f"Updated {appcast_path}")
print(f"  Version: {version}")
print(f"  Size: {file_size}")
print(f"  Signature: {signature}")
