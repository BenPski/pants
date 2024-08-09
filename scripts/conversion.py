#!/bin/env python
# Convert the exported password csv to the json to be imported into pants

import csv
import json
import argparse

def read_csv(filename):
    data = {}
    field_headers = {}
    rows = []
    with open(filename, 'r') as csvfile:
        csvreader = csv.reader(csvfile)

        fields = next(csvreader)
        for (i, field) in enumerate(fields):
            field_headers[field] = i

        for row in csvreader:
            rows.append(row)
    for row in rows:
        # 'name' for chromium based, 'url' for firefox based
        if 'name' in field_headers:
            pos = field_headers['name']
        else:
            pos = field_headers['url']
        name = row[pos]
        ty = "UsernamePassword"
        username = row[field_headers['username']]
        password = row[field_headers['password']]
        data[name] = {"ty": ty, "data": [("Username", username), ("Password", password)]}
    return data


def main():
    parser = argparse.ArgumentParser(description="Convert the exported csv from a browser to json for importing into pants")
    parser.add_argument("filename", help="csv to read from")
    args = parser.parse_args()

    data = read_csv(args.filename)

    out = json.dumps(data)
    print(out)


if __name__ == "__main__":
    main()
