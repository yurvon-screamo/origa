import re

with open('grammar.json', 'r', encoding='utf-8') as f:
    lines = f.readlines()

print('Analyzing JSON structure issues...')
print(f'Total lines: {len(lines)}')

# Find lines that appear to be continuations of JSON strings
string_continuations = []
in_string_context = False

for i, line in enumerate(lines, 1):
    stripped = line.strip()
    
    # Check if this line looks like it's continuing a JSON string
    # (starts with content that should be inside a string, without opening quote)
    if i > 1:
        prev_line = lines[i-2].strip()
        # Previous line ended without closing quote and this line continues
        if prev_line and not prev_line.endswith('"') and not prev_line.endswith(',') and not prev_line.endswith(':'):
            # Current line starts with text that doesn't look like JSON
            if stripped and not stripped.startswith(('{', '[', ']', '}', ',')):
                string_continuations.append(i)

print(f'\nFound {len(string_continuations)} potential string continuation issues:')
for i in string_continuations[:20]:
    print(f'  Line {i}: {lines[i-1][:80].rstrip()}')
