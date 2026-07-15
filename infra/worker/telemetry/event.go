package telemetry

import (
	"crypto/sha256"
	"encoding/hex"
	"net/http"
	"strconv"
	"strings"
	"time"
)

type Event struct {
	Channel   string
	Platform  string
	OSName    string
	OSVersion string
	Arch      string
	Version   string
	Build     string
	Model     string
	DailyID   string
	MonthlyID string
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
	dailyID := periodID(q.Get("daily_id"))
	monthlyID := periodID(q.Get("monthly_id"))
	if dailyID == "" || monthlyID == "" {
		dailyID = ""
		monthlyID = ""
	}

	return Event{
		Channel:   channel,
		Platform:  platform,
		OSName:    osName,
		OSVersion: osVersion,
		Arch:      arch,
		Version:   version,
		Build:     strings.TrimSpace(q.Get("build")),
		Model:     q.Get("model"),
		DailyID:   dailyID,
		MonthlyID: monthlyID,
	}, true
}

func periodID(value string) string {
	value = strings.ToLower(strings.TrimSpace(value))
	if len(value) != 64 {
		return ""
	}
	for _, char := range []byte(value) {
		if !((char >= '0' && char <= '9') || (char >= 'a' && char <= 'f')) {
			return ""
		}
	}
	return value
}

func IdentityKeys(event Event, ip string, now time.Time, secret string) (string, string, string) {
	if event.DailyID != "" && event.MonthlyID != "" {
		return event.DailyID, event.MonthlyID, "client"
	}
	day := strconv.FormatInt(now.Unix()/86400, 10)
	month := now.Format("2006-01")
	return anonymousPeriodKey(ip, day, secret), anonymousPeriodKey(ip, month, secret), "network"
}

func anonymousPeriodKey(ip, period, secret string) string {
	h := sha256.Sum256([]byte(ip + "|" + period + "|" + secret))
	return hex.EncodeToString(h[:12])
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
