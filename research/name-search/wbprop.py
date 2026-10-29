#!/usr/bin/env python3
"""Word_Break property of selected characters per WordBreakProperty-18.0.0.txt and the set of code points in WordBreakTest-18.0.0.txt.
Run: python3 wbprop.py WordBreakProperty.txt WordBreakTest.txt"""
import sys

m = {}
for l in open(sys.argv[1], encoding='utf8'):
    l = l.split('#')[0].strip()
    if not l:
        continue
    r, p = [x.strip() for x in l.split(';')]
    a = r.split('..')
    for c in range(int(a[0], 16), int(a[-1], 16) + 1):
        m[c] = p
for name, c in [('李', 0x674E), ('雷', 0x96F7), ('こ', 0x3053), ('世', 0x4E16), ('Thai ส', 0xE2A), ('Thai U+0E31', 0xE31),
                ('コ', 0x30B3), ('Hebrew ש', 0x5E9)]:
    print(name, m.get(c, 'Other'))
cps = set()
for l in open(sys.argv[2], encoding='utf8'):
    for tok in l.split('#')[0].split():
        if tok not in ('÷', '×'):
            cps.add(int(tok, 16))
print('code points >= U+0E00 and < U+F900 in WordBreakTest (CJK, Thai, kana):',
      sorted(hex(c) for c in cps if 0x0E00 <= c < 0xF900 and c not in range(0x2000, 0x3000)))
