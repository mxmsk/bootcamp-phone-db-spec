#!/usr/bin/env python3
"""Analysis of the output of `DIFF_LIST=1 uax29 diff 2>difflist.tsv` by character age (DerivedAge-18.0.0) and Word_Break class.
Usage: python3 diffage.py difflist.tsv DerivedAge.txt"""
import collections, sys
age = {}
for l in open(sys.argv[2], encoding='utf8'):
    l = l.split('#')[0].strip()
    if not l:
        continue
    r, a = [x.strip() for x in l.split(';')]
    x = r.split('..')
    for c in range(int(x[0], 16), int(x[-1], 16) + 1):
        age[c] = a
new = 0
old = collections.Counter()
ex = {}
for l in open(sys.argv[1]):
    cp, sc, wb = l.rstrip('\n').split('\t')
    c = int(cp[2:], 16)
    if age.get(c) == '18.0':
        new += 1
    else:
        k = (sc.strip('"'), wb)
        old[k] += 1
        ex.setdefault(k, cp)
print('mismatches on chars added in Unicode 18.0 (DerivedAge 18.0):', new)
print('mismatches on chars older than 18.0 (script, Word_Break from icu_properties; WordBreak(0)=Other, (9)=Extend, (1)=ALetter):')
for k, v in sorted(old.items(), key=lambda x: -x[1]):
    print(f'  {k[0]:14} {k[1]:14} {v:7}  first {ex[k]}')
