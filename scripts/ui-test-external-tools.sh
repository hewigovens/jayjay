write_external_merge_source() {
  local output="$1"
  local label="$2"
  local primary_seed="$3"
  local secondary_seed="$4"
  cat > "$output" <<SWIFT
import Foundation

struct ConflictSample {
    static func primaryGreeting(for name: String) -> String {
        let prefix = "$label primary"
        let retryLimit = $primary_seed
        let timeoutSeconds = $primary_seed
        let backoffSeconds = $primary_seed
        let endpoint = "$label-primary"
        let locale = "$label"
        let audience = "$label users"
        let cacheNamespace = "$label-primary-cache"
        let userAgent = "$label-primary-agent"
        let tracingLabel = "$label-primary-trace"
        let region = "$label-primary-region"
        let transport = "$label-primary-transport"
        let authentication = "$label-primary-auth"
        let requestPolicy = "$label-primary-policy"
        let responsePolicy = "$label-primary-response"
        let telemetryCategory = "$label-primary-telemetry"
        let fallbackMessage = "$label primary fallback"
        let recoverySuggestion = "$label primary recovery"
        let diagnosticsTag = "$label-primary-diagnostics"
        let punctuation = "!"
        return "\(prefix), \(name)\(punctuation) retries=\(retryLimit) timeout=\(timeoutSeconds) backoff=\(backoffSeconds) endpoint=\(endpoint) locale=\(locale) audience=\(audience) cache=\(cacheNamespace) agent=\(userAgent) trace=\(tracingLabel) region=\(region) transport=\(transport) auth=\(authentication) request=\(requestPolicy) response=\(responsePolicy) telemetry=\(telemetryCategory) fallback=\(fallbackMessage) recovery=\(recoverySuggestion) diagnostics=\(diagnosticsTag)"
    }

    static let stableIdentifiers = [
        "jayjay",
        "external-tool",
        "merge-editor",
        "scroll-fixture"
    ]

    static func secondaryGreeting(for name: String) -> String {
        let prefix = "$label secondary"
        let retryLimit = $secondary_seed
        let timeoutSeconds = $secondary_seed
        let backoffSeconds = $secondary_seed
        let endpoint = "$label-secondary"
        let locale = "$label"
        let audience = "$label users"
        let cacheNamespace = "$label-secondary-cache"
        let userAgent = "$label-secondary-agent"
        let tracingLabel = "$label-secondary-trace"
        let region = "$label-secondary-region"
        let transport = "$label-secondary-transport"
        let authentication = "$label-secondary-auth"
        let requestPolicy = "$label-secondary-policy"
        let responsePolicy = "$label-secondary-response"
        let telemetryCategory = "$label-secondary-telemetry"
        let fallbackMessage = "$label secondary fallback"
        let recoverySuggestion = "$label secondary recovery"
        let diagnosticsTag = "$label-secondary-diagnostics"
        let punctuation = "."
        return "\(prefix), \(name)\(punctuation) retries=\(retryLimit) timeout=\(timeoutSeconds) backoff=\(backoffSeconds) endpoint=\(endpoint) locale=\(locale) audience=\(audience) cache=\(cacheNamespace) agent=\(userAgent) trace=\(tracingLabel) region=\(region) transport=\(transport) auth=\(authentication) request=\(requestPolicy) response=\(responsePolicy) telemetry=\(telemetryCategory) fallback=\(fallbackMessage) recovery=\(recoverySuggestion) diagnostics=\(diagnosticsTag)"
    }
}
SWIFT
}

fixture_external_tools() {
  local root="$fixtures/external-tool"
  mkdir -p \
    "$root/diff-left" "$root/diff-right" \
    "$root/edit-left" "$root/edit-right" \
    "$root/merge"

  printf 'before comparison\n' > "$root/diff-left/file.txt"
  printf 'after comparison\n' > "$root/diff-right/file.txt"
  printf 'before edit\n' > "$root/edit-left/file.txt"
  printf 'after edit\n' > "$root/edit-right/file.txt"
  chmod 0644 "$root/edit-left/file.txt"
  chmod 0755 "$root/edit-right/file.txt"
  write_external_merge_source "$root/merge/left.swift" main 2 3
  write_external_merge_source "$root/merge/base.swift" base 1 1
  write_external_merge_source "$root/merge/right.swift" feature 4 5
  : > "$root/merge/output.swift"
}
