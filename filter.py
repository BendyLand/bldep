lines = []

with open("src/headers.txt") as file:
    for line in file:
        lines.append(line)

lines.sort()
result = []
for line in lines:
    if line not in result:
        result.append(line)

with open("src/headers.txt", "w") as file:
    for line in lines:
        file.write(line)
