import re

with open('opgg_test.html', 'r', encoding='utf-8') as f:
    html = f.read()

if "9111" in html:
    print("Found Triumph (9111)!")
else:
    print("Triumph not found")
