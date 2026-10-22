#!/usr/bin/env python3
"""Validate a Windows payload and write a byte-reproducible ZIP from those files."""

import argparse
import hashlib
from pathlib import Path
import re
import struct
import zipfile


# These ship with supported Windows versions. Other imports must be beside the EXEs.
SYSTEM_DLLS = set("""
advapi32 bcrypt bcryptprimitives cabinet cfgmgr32 comctl32 comdlg32 combase crypt32
cryptbase cryptnet cryptui d2d1 d3d11 d3d12 d3dcompiler_47 dcomp dbghelp dnsapi dwmapi
dwrite dxgi dxguid gdi32 gdiplus hid imm32 iphlpapi kernel32 kernelbase msimg32 msvcrt
ncrypt netapi32 normaliz ntdll ole32 oleacc oleaut32 opengl32 powrprof propsys psapi
rpcrt4 secur32 setupapi shell32 shcore shlwapi user32 userenv usp10 uxtheme version
windowscodecs winhttp wininet winmm winspool wintrust wldap32 ws2_32 wtsapi32 ucrtbase
uiautomationcore urlmon runtimeobject icu icuuc icuin
""".split())

EXECUTABLE_SUBSYSTEMS = {"jayjay.exe": 3, "jayjay-gpui.exe": 2}  # console CLI, GUI app
REQUIRED_FILES = [*EXECUTABLE_SUBSYSTEMS, "LICENSE.txt", "CORE-LICENSE.txt", "README.txt",
                  "THIRD-PARTY-GUI.html", "THIRD-PARTY-CLI.html", "OCTICONS-LICENSE.txt",
                  "SETI-LICENSE.txt"]


def pe_info(path):
    data = path.read_bytes()
    if data[:2] != b"MZ":
        raise ValueError(f"not a PE executable: {path}")
    pe = struct.unpack_from("<I", data, 0x3C)[0]
    if data[pe:pe + 4] != b"PE\0\0":
        raise ValueError(f"invalid PE signature: {path}")
    machine, count = struct.unpack_from("<HH", data, pe + 4)
    optional_size = struct.unpack_from("<H", data, pe + 20)[0]
    optional = pe + 24
    if machine != 0x8664 or struct.unpack_from("<H", data, optional)[0] != 0x20B:
        raise ValueError(f"expected Windows x64 PE32+: {path}")
    subsystem = struct.unpack_from("<H", data, optional + 68)[0]
    sections = [struct.unpack_from("<IIII", data, optional + optional_size + i * 40 + 8)
                for i in range(count)]

    def offset(rva):
        for size, base, raw_size, raw in sections:
            if base <= rva < base + max(size, raw_size):
                return raw + rva - base
        raise ValueError(f"unmapped PE address {rva:x}: {path}")

    imports = set()
    for index, stride, name_offset in [(1, 20, 12), (13, 32, 4)]:
        rva = struct.unpack_from("<I", data, optional + 112 + index * 8)[0]
        if not rva:
            continue
        cursor = offset(rva)
        while any(data[cursor:cursor + stride]):
            name_rva = struct.unpack_from("<I", data, cursor + name_offset)[0]
            start = offset(name_rva)
            imports.add(data[start:data.index(b"\0", start)].decode("ascii").lower())
            cursor += stride
    return subsystem, imports


def validate(payload):
    for name in REQUIRED_FILES:
        if not (payload / name).is_file() or not (payload / name).stat().st_size:
            raise ValueError(f"missing or empty payload file: {name}")
    bundled = {path.name.lower() for path in payload.iterdir()}
    for path in sorted(payload.iterdir()):
        if path.suffix.lower() not in (".exe", ".dll"):
            continue
        subsystem, imports = pe_info(path)
        expected = EXECUTABLE_SUBSYSTEMS.get(path.name)
        if expected and subsystem != expected:
            raise ValueError(f"wrong console/GUI subsystem for {path.name}: {subsystem}")
        for dll in imports:
            stem = dll.removesuffix(".dll")
            if stem in SYSTEM_DLLS or dll.startswith(("api-ms-win-", "ext-ms-win-")):
                continue
            if dll not in bundled:
                raise ValueError(f"{path.name} requires unbundled runtime {dll}")


def package(payload, version, output):
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9.+-]*", version):
        raise ValueError("version must be a filename-safe version identifier")
    validate(payload)
    files = sorted((path for path in payload.rglob("*") if path.is_file() and path.name != "SHA256SUMS"),
                   key=lambda path: path.relative_to(payload).as_posix())
    sums = "".join(f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.relative_to(payload).as_posix()}\n"
                   for path in files)
    (payload / "SHA256SUMS").write_bytes(sums.encode("utf-8"))
    output.mkdir(parents=True, exist_ok=True)
    archive = output / f"JayJay-{version}-windows-x64.zip"
    with zipfile.ZipFile(archive, "w") as zip_file:
        for path in sorted([*files, payload / "SHA256SUMS"], key=lambda path: path.relative_to(payload).as_posix()):
            info = zipfile.ZipInfo(path.relative_to(payload).as_posix(), (1980, 1, 1, 0, 0, 0))
            info.create_system = 3
            info.external_attr = 0o100644 << 16
            # ZipInfo entries default to stored.
            zip_file.writestr(info, path.read_bytes(), compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
    checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_suffix(".zip.sha256").write_bytes(f"{checksum}  {archive.name}\n".encode("ascii"))
    print(archive)
    return archive


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("payload", type=Path)
    parser.add_argument("version")
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    package(args.payload, args.version, args.output)
