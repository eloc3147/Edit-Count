function genRow(a) {
  'use strict';
  let name = a.album,
      edited = a.edited,
      deleted = a.deleted,
      total = a.total;
  return $('<tr\>').append(
    $('<td\>', {class: 'name-row', text: name}),
    $('<td\>', {class: 'prog-row', text: [edited, deleted, total].join('/')}),
    $('<td\>', {class: 'bar-row'}).append(
      $('<progress\>', {class: 'edited', max: total, value: edited}),
      $('<progress\>', {class: 'deleted', max: total, value: deleted})
    )
  );
}

$(document).ready(function() {
  'use strict';
  let ip = $('#table_ip'),
      complete = $('#table_complete'),
      totaledited = $('#totaledited'),
      totaldeleted = $('#totaldeleted'),
      label = $('#label'),
      edited = 0,
      deleted = 0,
      total = 0;
  for(let i=0; i < data.length; i++) {
    for(let j=0; j < data[i].albums.length; j++) {
      let album = data[i].albums[j];
      if(album.edited == album.total - album.deleted) {
        complete.append(genRow(album));
      } else {
        ip.append(genRow(album));
      }
      edited += album.edited;
      deleted += album.deleted;
      total += album.total;
    }
  }

  totaledited.attr('max', total);
  totaledited.attr('value', edited);
  totaldeleted.attr('max', total);
  totaldeleted.attr('value', deleted);
  label.text([edited, deleted, total].join('/'));
});
