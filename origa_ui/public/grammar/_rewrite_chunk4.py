import json

d = json.load(open('grammar.json', 'r', encoding='utf-8'))
rules = d['grammar'][120:160]

def rewrite_rule(rule):
    """Rewrite md_description for both RU and EN according to template."""
    title_ru = rule['content']['Russian']['title']
    title_en = rule['English']['title'] if 'English' in rule['content'] else rule['content']['English']['title']
    short_ru = rule['content']['Russian']['short_description']
    short_en = rule['content']['English']['short_description']

    # Map of rule titles to rewritten descriptions
    rewrites = get_rewrites(title_ru)
    
    if title_ru in rewrites:
        rule['content']['Russian']['md_description'] = rewrites[title_ru]['ru']
        rule['content']['English']['md_description'] = rewrites[title_ru]['en']
    
    return rule

def get_rewrites(title):
    return {}

# Apply rewrites
for i, rule in enumerate(rules):
    rules[i] = rewrite_rule(rule)

json.dump(rules, open('chunk_4.json', 'w', encoding='utf-8'), ensure_ascii=False, indent=2)
print('Done')
