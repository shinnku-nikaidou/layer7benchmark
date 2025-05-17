# remove_duplicate_ips.py

input_file = 'ip.txt'
output_file = 'ip_unique.txt'

with open(input_file, 'r', encoding='utf-8') as f:
    lines = [line.rstrip('\n') for line in f]

unique_lines = list(dict.fromkeys(lines))

with open(output_file, 'w', encoding='utf-8') as f:
    for line in unique_lines:
        f.write(line + '\n')
