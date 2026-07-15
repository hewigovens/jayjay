package telemetry

import (
	"net/http/httptest"
	"strings"
	"testing"
	"time"
)

func TestFromRequestSkipsProbeRequests(t *testing.T) {
	req := httptest.NewRequest("GET", "/ping?probe=1&version=0.3.1", nil)

	if _, ok := FromRequest(req, "gpui"); ok {
		t.Fatal("probe requests must not be logged")
	}
}

func TestFromRequestSkipsNonReleaseVersions(t *testing.T) {
	for _, version := range []string{"", "test", "unknown", "0.3", "0.3.1-dev", "0.3.1.4"} {
		req := httptest.NewRequest("GET", "/ping?version="+version, nil)
		if _, ok := FromRequest(req, "gpui"); ok {
			t.Fatalf("version %q must not be logged", version)
		}
	}
}

func TestFromRequestAcceptsReleaseVersion(t *testing.T) {
	dailyID := strings.Repeat("a", 64)
	monthlyID := strings.Repeat("b", 64)
	req := httptest.NewRequest("GET", "/ping?version=0.3.1&build=42&platform=gpui&os=linux&osver=6.12&arch=x86_64&daily_id="+dailyID+"&monthly_id="+monthlyID, nil)

	event, ok := FromRequest(req, "gpui")
	if !ok {
		t.Fatal("release version should be logged")
	}
	if event.Version != "0.3.1" {
		t.Fatalf("version = %q, want 0.3.1", event.Version)
	}
	if event.Platform != "gpui" || event.OSName != "linux" || event.Arch != "x86_64" {
		t.Fatalf("unexpected event fields: %+v", event)
	}
	if event.Build != "42" || event.OSVersion != "6.12" {
		t.Fatalf("build/OS version missing: %+v", event)
	}
	if event.DailyID != dailyID || event.MonthlyID != monthlyID {
		t.Fatalf("rotating identity missing: %+v", event)
	}
}

func TestFromRequestRejectsPartialOrMalformedIdentity(t *testing.T) {
	validID := strings.Repeat("a", 64)
	for _, query := range []string{
		"daily_id=" + validID,
		"daily_id=not-hex&monthly_id=" + validID,
		"daily_id=" + validID + "&monthly_id=short",
	} {
		req := httptest.NewRequest("GET", "/ping?version=0.3.1&"+query, nil)
		event, ok := FromRequest(req, "gpui")
		if !ok {
			t.Fatal("release event unexpectedly skipped")
		}
		if event.DailyID != "" || event.MonthlyID != "" {
			t.Fatalf("invalid identity accepted: %+v", event)
		}
	}
}

func TestIdentityKeysPreferClientRotationAndFallbackByNetworkPeriod(t *testing.T) {
	now := time.Date(2026, 7, 15, 12, 0, 0, 0, time.UTC)
	dailyID := strings.Repeat("a", 64)
	monthlyID := strings.Repeat("b", 64)

	daily, monthly, kind := IdentityKeys(Event{DailyID: dailyID, MonthlyID: monthlyID}, "203.0.113.1", now, "secret")
	if daily != dailyID || monthly != monthlyID || kind != "client" {
		t.Fatalf("client identity not preserved: %q %q %q", daily, monthly, kind)
	}

	daily, monthly, kind = IdentityKeys(Event{}, "203.0.113.1", now, "secret")
	if daily == monthly || kind != "network" {
		t.Fatalf("network fallback did not rotate by period: %q %q %q", daily, monthly, kind)
	}
	if strings.Contains(daily, "203.0.113.1") || strings.Contains(monthly, "203.0.113.1") {
		t.Fatal("raw IP leaked into stored identity")
	}
}

func TestFromRequestUsesSparkleVersionAlias(t *testing.T) {
	req := httptest.NewRequest("GET", "/appcast.xml?appVersionShort=0.3.1&osVersion=15.5&cputype=16777228", nil)

	event, ok := FromRequest(req, "swiftui")
	if !ok {
		t.Fatal("Sparkle release version should be logged")
	}
	if event.Version != "0.3.1" {
		t.Fatalf("version = %q, want 0.3.1", event.Version)
	}
	if event.Platform != "macos" || event.OSName != "macos" || event.OSVersion != "15.5" || event.Arch != "arm64" {
		t.Fatalf("Sparkle defaults not applied: %+v", event)
	}
	if event.DailyID != "" || event.MonthlyID != "" {
		t.Fatalf("legacy Sparkle payload unexpectedly has client identity: %+v", event)
	}
}

func TestFromRequestAcceptsLegacyGPUIPayload(t *testing.T) {
	req := httptest.NewRequest("GET", "/ping?version=0.3.1&platform=gpui&os=linux&osver=6.12&arch=x86_64", nil)

	event, ok := FromRequest(req, "gpui")
	if !ok {
		t.Fatal("legacy GPUI release should be logged")
	}
	if event.Platform != "gpui" || event.OSName != "linux" || event.OSVersion != "6.12" || event.Arch != "x86_64" {
		t.Fatalf("legacy GPUI fields not preserved: %+v", event)
	}
	day, month, kind := IdentityKeys(event, "203.0.113.1", time.Date(2026, 7, 15, 12, 0, 0, 0, time.UTC), "secret")
	if day == "" || month == "" || day == month || kind != "network" {
		t.Fatalf("legacy GPUI payload did not use network-period fallback: %q %q %q", day, month, kind)
	}
}

func TestIsReleaseVersion(t *testing.T) {
	for _, version := range []string{"0.3.1", "10.20.300"} {
		if !IsReleaseVersion(version) {
			t.Fatalf("%q should be a release version", version)
		}
	}
	for _, version := range []string{"", "test", "unknown", "0.3", "0.3.1-dev", "0.3.1.4"} {
		if IsReleaseVersion(version) {
			t.Fatalf("%q should not be a release version", version)
		}
	}
}
