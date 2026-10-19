import Foundation
import JayJayCore

extension [DiffSpan] {
    func localizingFileSizes(locale: Locale) -> [DiffSpan] {
        guard first?.text.hasPrefix("<") == true, last?.text.hasSuffix(" bytes)>") == true else { return self }
        return localizeBinaryFileSpans(spans: self, locale: locale.identifier(.bcp47))
    }
}
