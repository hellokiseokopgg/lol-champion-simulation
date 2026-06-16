import re
import json

with open('opgg_test.html', 'r', encoding='utf-8') as f:
    html = f.read()

chunks = re.findall(r'self\.__next_f\.push\(\[1,"(.*?)\]\)', html)
for chunk in chunks:
    decoded = chunk.replace('\\"', '"').replace('\\\\', '\\')
    if "nodfan" in decoded and "8010" in decoded:
        # Find where 8010 is and print surroundings
        idx = decoded.find('8010')
        print("AROUND 8010:", decoded[max(0, idx-100):min(len(decoded), idx+500)])
