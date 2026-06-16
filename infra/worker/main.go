// JayJay service worker (Cloudflare Worker, standard Go -> WASM).
//
//	GET /appcast.xml  — macOS/Sparkle: log the profiling query params Sparkle
//	                    appends, then proxy the real appcast (APPCAST_ORIGIN).
//	                    The EdDSA signature is verified in-app, so the proxy
//	                    cannot tamper with updates.
//	GET /ping         — GPUI (Linux/Windows): log app version + OS + arch.
//
// Privacy: no IP or personal data is stored. The daily-unique key is a salted
// SHA-256 of (IP + day + HASH_SECRET); the raw IP never leaves this function.
package main

import (
	"crypto/sha256"
	"database/sql"
	"encoding/hex"
	"io"
	"net/http"
	"strconv"
	"time"

	"github.com/syumai/workers"
	"github.com/syumai/workers/cloudflare"
	_ "github.com/syumai/workers/cloudflare/d1" // registers the "d1" sql driver
	"github.com/syumai/workers/cloudflare/fetch"
)

func main() {
	db, err := sql.Open("d1", "DB")
	if err != nil {
		panic(err)
	}
	http.HandleFunc("/ping", func(w http.ResponseWriter, r *http.Request) {
		logEvent(db, r, "gpui")
		w.Write([]byte("ok"))
	})
	appcastHandler := func(w http.ResponseWriter, r *http.Request) {
		logEvent(db, r, "swiftui")
		proxyAppcast(w, r)
	}
	http.HandleFunc("/appcast.xml", appcastHandler)
	http.HandleFunc("/", appcastHandler)
	workers.Serve(nil) // use http.DefaultServeMux
}

func proxyAppcast(w http.ResponseWriter, r *http.Request) {
	req, err := fetch.NewRequest(r.Context(), http.MethodGet, cloudflare.Getenv("APPCAST_ORIGIN"), nil)
	if err != nil {
		w.WriteHeader(http.StatusInternalServerError)
		return
	}
	res, err := fetch.NewClient().Do(req, nil)
	if err != nil {
		w.WriteHeader(http.StatusBadGateway)
		return
	}
	defer res.Body.Close()
	w.Header().Set("content-type", "application/xml; charset=utf-8")
	io.Copy(w, res.Body)
}

func logEvent(db *sql.DB, r *http.Request, channel string) {
	day := time.Now().Unix() / 86400
	unique := dailyUnique(r.Header.Get("CF-Connecting-IP"), day, cloudflare.Getenv("HASH_SECRET"))

	q := r.URL.Query()
	version := firstNonEmpty(q.Get("version"), q.Get("appVersionShort"), q.Get("appVersion"))
	osName := orDefault(q.Get("os"), appcastDefault(channel, "macos"))
	osVersion := firstNonEmpty(q.Get("osver"), q.Get("osVersion"))
	arch := q.Get("arch")
	if arch == "" {
		arch = archFromCPUType(q.Get("cputype"))
	}
	platform := orDefault(q.Get("platform"), appcastDefault(channel, "macos"))

	// Fire-and-forget: a logging failure must never break the update check.
	_, _ = db.ExecContext(r.Context(),
		`INSERT INTO pings (day, unique_key, channel, platform, os, os_version, arch, version, model)
		 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)`,
		day, unique, channel, platform, osName, osVersion, arch, version, q.Get("model"))
}

func dailyUnique(ip string, day int64, secret string) string {
	h := sha256.Sum256([]byte(ip + "|" + strconv.FormatInt(day, 10) + "|" + secret))
	return hex.EncodeToString(h[:12])
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
