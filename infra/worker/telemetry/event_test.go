package telemetry

import (
	"net/http/httptest"
	"testing"
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
	req := httptest.NewRequest("GET", "/ping?version=0.3.1&platform=gpui&os=linux&arch=x86_64", nil)

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
}

func TestFromRequestUsesSparkleVersionAlias(t *testing.T) {
	req := httptest.NewRequest("GET", "/appcast.xml?appVersionShort=0.3.1", nil)

	event, ok := FromRequest(req, "swiftui")
	if !ok {
		t.Fatal("Sparkle release version should be logged")
	}
	if event.Version != "0.3.1" {
		t.Fatalf("version = %q, want 0.3.1", event.Version)
	}
	if event.Platform != "macos" || event.OSName != "macos" {
		t.Fatalf("Sparkle defaults not applied: %+v", event)
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
