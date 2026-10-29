// Independent cross-check of the name order reference (ADR 0036-0039) through ICU4C (a different code base than ICU4X).
// The output format is the same as in research/ordering/icu4x-order: `<set> | <variant> | name1 < name2 < ...`.
// Names that are equal under the collator are not distinguished in the order (ADR 0036); in the output they are joined by " = ", their order is by
// code points (only for output reproducibility).
#include <unicode/coll.h>
#include <unicode/unistr.h>
#include <unicode/uversion.h>
#include <unicode/uclean.h>
#include <unicode/uchar.h>
#include <algorithm>
#include <iostream>
#include <string>
#include <vector>
using namespace icu;
using namespace std;

static u32string cps(const UnicodeString &s) {
  u32string r;
  for (int32_t i = 0; i < s.length();) { UChar32 c = s.char32At(i); r.push_back(c); i += U16_LENGTH(c); }
  return r;
}

int main() {
  vector<pair<string, vector<string>>> sets = {
    {"script", {"Ann", "Яков", "Бобр", "Zed", "Анна"}},
    {"yo-tail", {"Ежи", "Ёж", "Еж", "Её", "Жук"}},
    {"yo-pair", {"Ёлка", "Елка", "ЕЛКА"}},
    {"case", {"Антон", "АННА", "Анна", "анна"}},
    {"case-yo", {"Её", "Еж", "Ёж", "ёж", "Ежи"}},
    {"punct", {"AnnZed", "Anna", "Ann-Zed", "Ann Zed"}},
    {"punct2", {"Ann-Smith", "Ann Smith", "AnnSmith"}},
    {"numeric", {"Ann 10", "Ann 2"}},
    {"numeric2", {"Ann 2", "Ann 10", "Ann 1"}},
    {"invisible", {"A­nn", "Ann"}},
    {"smith", {"Bob Smith", "Ann Smith"}},
  };
  struct V { const char *name; UColAttributeValue strength; UColAttributeValue alt; UColAttributeValue caseFirst; UColAttributeValue numeric; };
  vector<V> variants = {
    {"tertiary+nonignorable (default)", UCOL_TERTIARY, UCOL_NON_IGNORABLE, UCOL_DEFAULT, UCOL_DEFAULT},
    {"secondary", UCOL_SECONDARY, UCOL_NON_IGNORABLE, UCOL_DEFAULT, UCOL_DEFAULT},
    {"primary", UCOL_PRIMARY, UCOL_NON_IGNORABLE, UCOL_DEFAULT, UCOL_DEFAULT},
    {"tertiary+shifted", UCOL_TERTIARY, UCOL_SHIFTED, UCOL_DEFAULT, UCOL_DEFAULT},
    {"tertiary+case_first upper", UCOL_TERTIARY, UCOL_NON_IGNORABLE, UCOL_UPPER_FIRST, UCOL_DEFAULT},
    {"tertiary+numeric", UCOL_TERTIARY, UCOL_NON_IGNORABLE, UCOL_DEFAULT, UCOL_ON},
  };
  UErrorCode st = U_ZERO_ERROR;
  for (auto &v : variants) {
    Collator *c = Collator::createInstance(Locale("ru"), st);
    c->setAttribute(UCOL_STRENGTH, v.strength, st);
    c->setAttribute(UCOL_ALTERNATE_HANDLING, v.alt, st);
    c->setAttribute(UCOL_CASE_FIRST, v.caseFirst, st);
    c->setAttribute(UCOL_NUMERIC_COLLATION, v.numeric, st);
    if (U_FAILURE(st)) { cerr << "icu error " << u_errorName(st) << endl; return 1; }
    for (auto &s : sets) {
      vector<UnicodeString> u;
      for (auto &n : s.second) u.push_back(UnicodeString::fromUTF8(n));
      sort(u.begin(), u.end(), [&](const UnicodeString &a, const UnicodeString &b) {
        UCollationResult r = c->compare(a, b, st);
        if (r == UCOL_EQUAL) return cps(a) < cps(b);
        return r == UCOL_LESS;
      });
      cout << s.first << " | " << v.name << " | ";
      for (size_t i = 0; i < u.size(); i++) {
        string o; u[i].toUTF8String(o);
        const char *sep = " < ";
        if (i && c->compare(u[i - 1], u[i], st) == UCOL_EQUAL) sep = " = ";
        cout << (i ? sep : "") << o;
      }
      cout << "\n";
    }
    delete c;
  }
  UVersionInfo uv; u_getUnicodeVersion(uv); char ub[U_MAX_VERSION_STRING_LENGTH]; u_versionToString(uv, ub);
  cerr << "ICU4C " << U_ICU_VERSION << ", Unicode " << ub << endl;
  return 0;
}
