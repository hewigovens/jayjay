// JayJay service worker (Cloudflare Worker, standard Go -> WASM).
//
//	GET /ping         — SwiftUI/GPUI: log anonymous app version/build + OS + arch.
//	GET /appcast.xml  — compatibility proxy for older SwiftUI releases. The
//	                    EdDSA signature is still verified in-app.
//
// Privacy: no IP or personal data is stored. New clients send day/month-scoped
// rotating hashes derived from an on-device secret. Older clients fall back to
// salted network-period hashes; the raw IP never leaves this function.
package main

import (
	"database/sql"
	"io"
	"net/http"
	"time"

	"github.com/hewigovens/jayjay/infra/worker/telemetry"
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
	event, ok := telemetry.FromRequest(r, channel)
	if !ok {
		return
	}
	now := time.Now().UTC()
	day := now.Unix() / 86400
	dailyKey, monthlyKey, identityKind := telemetry.IdentityKeys(
		event,
		r.Header.Get("CF-Connecting-IP"),
		now,
		cloudflare.Getenv("HASH_SECRET"),
	)

	// Fire-and-forget: a logging failure must never break the update check.
	_, _ = db.ExecContext(r.Context(),
		`INSERT INTO pings (day, unique_key, monthly_key, identity_kind, channel, platform, os, os_version, arch, version, build, model)
		 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
		day, dailyKey, monthlyKey, identityKind, event.Channel, event.Platform, event.OSName, event.OSVersion, event.Arch, event.Version, event.Build, event.Model)
}
