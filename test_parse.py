import re

with open('opgg_test.html', 'r', encoding='utf-8') as f:
    html = f.read()

chunks = re.findall(r'self\.__next_f\.push\(\[1,"(.*?)\]\)', html)
for chunk in chunks:
    decoded = chunk.replace('\\"', '"').replace('\\\\', '\\')
    if "nodfan" in decoded:
        if "rune" in decoded.lower():
            print("Found rune data!")
        if "skill" in decoded.lower() or "spell" in decoded.lower():
            print("Found skill data!")
        
        # Let's see if 8010 (Conqueror) is there
        if "8010" in decoded:
            print("Found 8010 (Conqueror)!")
        
