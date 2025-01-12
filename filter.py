lines = []

with open("src/headers.txt") as file:
    for line in file:
        lines.append(line)

result = sorted(list(set(lines)))

with open("src/headers.txt", "w") as file:
    for line in result:
        file.write(line)
