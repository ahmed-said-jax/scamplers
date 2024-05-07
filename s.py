import csv

with open('f.csv') as f:
    reader = csv.DictReader(f)

    for line in reader:
        for k, v in line.items():
            print(f'{k}: {v}')