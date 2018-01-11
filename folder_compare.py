import sys
import os
import glob
import json
import re

def listdirs(*dir):
  return next(os.walk(os.path.join(*dir, '.')))[1]

source_folder = os.path.abspath(os.environ['PHOTO_SOURCE_DIR'])
dest_folder = os.path.abspath(os.environ['PHOTO_DEST_DIR'])

data = []
counts = {}
if os.path.exists('counts.json'):
  with open('counts.json', 'r') as file:
    counts = json.loads(file.read())

r = re.compile(r".*\.(NEF|CR2|DNG)$", re.I)

for year in listdirs(source_folder):
  albums = []
  if not year in counts:
    counts[year] = {}

  for album in listdirs(source_folder, year):
    print(album)

    if not album in counts[year]:
      counts[year][album] = []

    raw_folder = os.path.join(source_folder, year, album)
    raw_files = [os.path.splitext(filename)[0] for filename in os.listdir(raw_folder) if r.match(filename)]
    edit_folder = os.path.join(dest_folder, year, album)

    # Ensure the list of all files ever is updated
    for raw in raw_files:
      if not raw in counts[year][album]:
        counts[year][album].append(raw)


    # Build list of files in edited folder
    edit_files = []
    if(os.path.exists(edit_folder)):
      for name in os.listdir(edit_folder):
        path = os.path.join(edit_folder, name)
        if(os.path.isfile(path)):
          edit_files.append(os.path.splitext(name)[0])
        elif(os.path.isdir(path)):
          for subname in os.listdir(os.path.join(edit_folder, path)):
            subpath = os.path.join(path, subname)
            if(os.path.isfile(subpath)):
              edit_files.append(os.path.splitext(subname)[0])

    # Count edited and deleted files
    edit_count = 0
    deleted_count = 0
    for filename in counts[year][album]:
      if filename not in raw_files:
        deleted_count += 1
      elif filename in edit_files:
        edit_count += 1

    albums.append({
      'album': album,
      'total': len(counts[year][album]),
      'edited': edit_count,
      'deleted': deleted_count
    })
  data.append({
    'year': year,
    'albums': albums
  })

with open('photo_data.js', 'w') as outfile:
  outfile.write('var data=' + json.dumps(data) + ';')

with open('counts.json', 'w') as outfile:
  outfile.write(json.dumps(counts))
