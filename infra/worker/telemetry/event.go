package telemetry

import (
	"net/http"
	"strings"
)

type Event struct {
	Channel   string
	Platform  string
	OSName    string
	OSVersion string
	Arch      string
	Version   string
	Model     string
}

func FromRequest(r *http.Request, channel string) (Event, bool) {
	q := r.URL.Query()
	version := strings.TrimSpace(firstNonEmpty(q.Get("version"), q.Get("appVersionShort"), q.Get("appVersion")))
	if ShouldSkip(q.Get("probe"), version) {
		return Event{}, false
	}
	osName := orDefault(q.Get("os"), appcastDefault(channel, "macos"))
	osVersion := firstNonEmpty(q.Get("osver"), q.Get("osVersion"))
	arch := q.Get("arch")
	if arch == "" {
		arch = archFromCPUType(q.Get("cputype"))
	}
	platform := orDefault(q.Get("platform"), appcastDefault(channel, "macos"))

	return Event{
		Channel:   channel,
		Platform:  platform,
		OSName:    osName,
		OSVersion: osVersion,
		Arch:      arch,
		Version:   version,
		Model:     q.Get("model"),
	}, true
}

func ShouldSkip(probe, version string) bool {
	probe = strings.TrimSpace(strings.ToLower(probe))
	if probe == "1" || probe == "true" {
		return true
	}
	return !IsReleaseVersion(version)
}

func IsReleaseVersion(version string) bool {
	parts := strings.Split(strings.TrimSpace(version), ".")
	if len(parts) != 3 {
		return false
	}
	for _, part := range parts {
		if part == "" {
			return false
		}
		for _, b := range []byte(part) {
			if b < '0' || b > '9' {
				return false
			}
		}
	}
	return true
}

// Sparkle sends a mach-o cputype; map the common ones, pass anything else through.
func archFromCPUType(cputype string) string {
	switch cputype {
	case "16777223":
		return "x86_64"
	case "16777228":
		return "arm64"
	case "7":
		return "x86"
	default:
		return cputype
	}
}

func firstNonEmpty(vals ...string) string {
	for _, v := range vals {
		if v != "" {
			return v
		}
	}
	return ""
}

func orDefault(v, fallback string) string {
	if v == "" {
		return fallback
	}
	return v
}

func appcastDefault(channel, value string) string {
	if channel == "swiftui" {
		return value
	}
	return ""
}
